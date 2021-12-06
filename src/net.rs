mod handshake;
mod login;
mod play;
mod status;

use std::sync::Arc;

use aes::{cipher::AsyncStreamCipher, Aes128};
use cfb8::Cfb8;
use eyre::bail;
use nom::{multi::length_data, HexDisplay, IResult};
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

use crate::{
    auth::{AuthSession, Keys},
    config::Config,
    server::ServerHook,
    varint::{self, varint, VarInt},
};
use async_trait::async_trait;

#[async_trait]
pub trait Packet: std::fmt::Debug {
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()>;
}

type BoxedPacket<'a> = Box<dyn Packet + Send + Sync + 'a>;
type AesCipher = Cfb8<Aes128>;
pub struct Connection {
    socket: TcpStream,
    server: ServerHook,
    state: ConnectionState,
    config: Arc<Config>,

    keys: Keys,
    auth_session: Option<AuthSession>,
    cipher: Option<AesCipher>,
}

impl Connection {
    pub const fn new(
        socket: TcpStream,
        server: ServerHook,
        keys: Keys,
        config: Arc<Config>,
    ) -> Self {
        Self {
            socket,
            server,
            state: ConnectionState::Handshake,
            config,

            keys,
            auth_session: None,
            cipher: None,
        }
    }

    #[instrument(skip_all)]
    pub async fn connection_loop(mut self) -> eyre::Result<()> {
        let mut buf = vec![0u8; 1024];
        loop {
            let read = self.socket.read(&mut buf).await?;
            if read == 0 {
                debug!("Connection reset");
                return Ok(());
            }

            use ::nom::Err;
            match self.read_packet(&buf[..read]).await {
                Ok(_) => {}
                Err(Err::Error(e) | Err::Failure(e)) => {
                    bail!("Parsing error: {:?}", e);
                }
                Err(Err::Incomplete(n)) => {
                    debug!(?n, "needed more data!");
                    // ignore
                    continue;
                }
            }
        }
    }

    #[instrument(skip(self, input))]
    pub async fn read_packet<'data>(&mut self, mut input: &'data [u8]) -> IResult<&'data [u8], ()> {
        loop {
            trace!(?input);
            let (i, data) = length_data(varint::<u32>)(input)?;
            input = i;
            trace!(?input, ?data);
            let (rem, packet) = match self.state {
                ConnectionState::Handshake => handshake::read_packet(data),
                ConnectionState::Status => status::read_packet(data),
                ConnectionState::Login => login::read_packet(data),
                ConnectionState::Play => todo!(),
            }?;
            trace!(?rem, ?packet);
            assert!(rem.is_empty());

            debug!(?packet, "Got packet");
            //todo
            packet.handle(self).await.unwrap();

            if input.is_empty() {
                return Ok((input, ()));
            }
        }
    }

    pub async fn kick(&mut self, reason: &str) -> eyre::Result<()> {
        let packet_id = match self.state {
            ConnectionState::Login => 0x00,
            ConnectionState::Play => 0x1a,
            _ => bail!("kick packets cannot be issued in state {:?}", self.state),
        };
        ResponseBuilder::new(packet_id)
            .var_data(reason)
            .send(self)
            .await?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
}

#[derive(Debug)]
pub struct ResponseBuilder {
    data: Vec<u8>,
}

impl ResponseBuilder {
    pub fn new(packet_id: u32) -> Self {
        Self {
            data: varint::serialize_to_bytes(packet_id as u32),
        }
    }
    #[instrument(skip_all)]
    pub fn add<'builder, T: ToResponseField>(&'builder mut self, t: T) -> &'builder mut Self {
        t.to_request_field(self);
        self
    }
    #[instrument(skip_all)]
    pub fn add_many<'builder, T: ToResponseField>(
        &'builder mut self,
        ts: &[T],
    ) -> &'builder mut Self {
        self.varint(ts.len() as u32);
        for t in ts {
            t.to_request_field(self);
        }
        self
    }
    #[instrument(skip_all)]
    pub fn varint<'builder, V: VarInt>(&'builder mut self, v: V) -> &'builder mut Self {
        trace!(?v);
        varint::serialize_and_append(v, &mut self.data);
        self
    }
    #[instrument(skip_all)]
    pub fn raw_data<'builder, B: AsRef<[u8]>>(&'builder mut self, b: B) -> &'builder mut Self {
        let b = b.as_ref();
        trace!(?b);
        self.data.extend_from_slice(b);
        self
    }
    #[instrument(skip_all)]
    pub fn var_data<'builder, B: AsRef<[u8]>>(&'builder mut self, b: B) -> &'builder mut Self {
        let b = b.as_ref();
        trace!(?b);
        self.varint(b.len() as u32).raw_data(b)
    }
    #[instrument(skip_all)]
    pub fn nbt<'builder, T: Serialize>(&'builder mut self, t: T) -> eyre::Result<&'builder mut Self> {
        let mut buf = vec![];
        nbt::to_writer(&mut buf, &t, None)?;
        trace!(?buf);
        Ok(self.var_data(buf))
    }
    #[instrument(skip_all)]
    pub fn gzipped_nbt<'builder, T: Serialize>(&'builder mut self, t: T) -> eyre::Result<&'builder mut Self> {
        let mut buf = vec![];
        nbt::to_gzip_writer(&mut buf, &t, None)?;
        trace!(?buf);
        Ok(self.var_data(buf))
    }
    #[instrument(skip_all)]
    pub fn zlibbed_nbt<'builder, T: Serialize>(&'builder mut self, t: T) -> eyre::Result<&'builder mut Self> {
        let mut buf = vec![];
        nbt::to_zlib_writer(&mut buf, &t, None)?;
        trace!(?buf);
        Ok(self.var_data(buf))
    }
    #[instrument(skip_all)]
    pub fn json<'builder, T: Serialize>(&'builder mut self, t: T) -> eyre::Result<&'builder mut Self> {
        let buf = serde_json::to_string(&t)?;
        trace!(?buf);
        Ok(self.var_data(buf))
    }

    #[instrument(skip_all)]
    pub async fn send(&mut self, conn: &mut Connection) -> eyre::Result<()> {
        let mut header = varint::serialize_to_bytes(self.data.len() as u32);
        let data = &mut self.data;
        trace!(?header);
        trace!("\n{}", data.to_hex(16));

        if let Some(cipher) = &mut conn.cipher {
            cipher.encrypt(&mut header);
            cipher.encrypt(data);
        }
        conn.socket.write(&header).await?;
        conn.socket.write(data).await?;
        Ok(())
    }
}

pub trait ToResponseField {
    fn to_request_field(&self, builder: &mut ResponseBuilder);
}
impl ToResponseField for &str {
    fn to_request_field(&self, builder: &mut ResponseBuilder) {
        builder.var_data(self);
    }
}
impl ToResponseField for &String {
    fn to_request_field(&self, builder: &mut ResponseBuilder) {
        self.as_str().to_request_field(builder)
    }
}
impl ToResponseField for bool {
    fn to_request_field(&self, builder: &mut ResponseBuilder) {
        builder.add(if *self { 1u8 } else { 0u8 });
    }
}
impl ToResponseField for Uuid {
    fn to_request_field(&self, builder: &mut ResponseBuilder) {
        builder.add(self.as_u128());
    }
}
macro_rules! into_request_field_primitive_impls {
    ($($ty:ty),+) => {
        $(
            impl ToResponseField for $ty {
                #[tracing::instrument]
                #[inline]
                fn to_request_field(&self, builder: &mut ResponseBuilder) {
                    builder.raw_data(&self.to_be_bytes());
                }
            }
        )+
    };
}
into_request_field_primitive_impls!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
