mod handshake;
mod login;
mod status;

use std::sync::Arc;

use aes::{Aes128, cipher::AsyncStreamCipher};
use cfb8::Cfb8;
use eyre::bail;
use nom::{multi::length_data, HexDisplay, IResult};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug, info, instrument, trace, warn};

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
                info!("Connection reset");
                return Ok(());
            }

            use ::nom::Err;
            match self.read_packet(&buf[..read]).await {
                Ok(_) => {}
                Err(Err::Error(e) | Err::Failure(e)) => {
                    debug!("TODO");
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

    #[instrument(skip_all)]
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
        RequestBuilder::new(0x1a)
            .var_blob(reason)
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
pub struct RequestBuilder {
    data: Vec<u8>,
}

impl RequestBuilder {
    pub fn new(packet_id: u32) -> Self {
        Self {
            data: varint::serialize_to_bytes(packet_id as u32),
        }
    }

    #[instrument]
    pub fn u8<'builder>(&'builder mut self, v: u8) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn u16<'builder>(&'builder mut self, v: u16) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn u32<'builder>(&'builder mut self, v: u32) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn u64<'builder>(&'builder mut self, v: u64) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn u128<'builder>(&'builder mut self, v: u128) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn i8<'builder>(&'builder mut self, v: i8) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn i16<'builder>(&'builder mut self, v: i16) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn i32<'builder>(&'builder mut self, v: i32) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn i64<'builder>(&'builder mut self, v: i64) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument]
    pub fn i128<'builder>(&'builder mut self, v: i128) -> &'builder mut Self {
        trace!(?v);
        self.raw_blob(v.to_be_bytes())
    }
    #[instrument(skip_all)]
    pub fn varint<'builder, V: VarInt>(&'builder mut self, v: V) -> &'builder mut Self {
        trace!(?v);
        varint::serialize_and_append(v, &mut self.data);
        self
    }
    #[instrument(skip_all)]
    pub fn raw_blob<'builder, B: AsRef<[u8]>>(&'builder mut self, b: B) -> &'builder mut Self {
        let b = b.as_ref();
        trace!(?b);
        self.data.extend_from_slice(b);
        self
    }
    #[instrument(skip_all)]
    pub fn var_blob<'builder, B: AsRef<[u8]>>(&'builder mut self, b: B) -> &'builder mut Self {
        let b = b.as_ref();
        trace!(?b);
        self.varint(b.len() as u32).raw_blob(b)
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
