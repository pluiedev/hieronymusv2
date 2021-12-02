mod handshake;
mod login;
mod status;

use eyre::bail;
use nom::{multi::length_data, IResult};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::{debug, instrument, trace, warn};

use crate::{server::ServerHook, varint::varint};
use async_trait::async_trait;

#[async_trait]
pub trait Packet: std::fmt::Debug {
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()>;
}

type BoxedPacket<'a> = Box<dyn Packet + Send + Sync + 'a>;

pub struct Connection {
    socket: TcpStream,
    server: ServerHook,
    state: ConnectionState,
}

impl Connection {
    pub const fn new(socket: TcpStream, server: ServerHook) -> Self {
        Self {
            socket,
            server,
            state: ConnectionState::Handshake,
        }
    }

    #[instrument(skip_all)]
    pub async fn connection_loop(mut self) -> eyre::Result<()> {
        let mut buf = vec![0u8; 1024];
        loop {
            let read = self.socket.read(&mut buf).await?;
            if read == 0 {
                warn!("End of stream");
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
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
}
