mod handshake;
mod login;
mod play;
mod status;

use std::net::SocketAddr;

use eyre::bail;
use nom::{
    multi::{length_data, many0_count},
    IResult,
};
use tokio::{io::AsyncReadExt, net::TcpStream, sync::mpsc::Sender};
use tracing::{debug, instrument};

use crate::{varint::varint, ServerRequest};

trait PacketHandler<Packet> {
    fn handle(&mut self, packet: Packet);
}

pub struct Connection {
    socket: TcpStream,
    addr: SocketAddr,
    server: Sender<ServerRequest>,
    state: ConnectionState,
}

#[derive(Clone, Copy, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
}

impl Connection {
    pub const fn new(socket: TcpStream, addr: SocketAddr, server: Sender<ServerRequest>) -> Self {
        Self {
            socket,
            addr,
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
                bail!("End of stream");
            }
            debug!(data = tracing::field::debug(&buf[..read]));

            use ::nom::Err;
            match self.read_packet(&buf[..read]) {
                Ok((_, read_packets)) => {
                    debug!(read_packets);
                }
                Err(Err::Error(e) | Err::Failure(e)) => {
                    bail!("Parsing error: {:?}", e);
                }
                Err(Err::Incomplete(_)) => {
                    // ignore
                    continue;
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub fn read_packet<'data>(&mut self, input: &'data [u8]) -> IResult<&'data [u8], usize> {
        many0_count(|input| {
            let (input, data) = length_data(varint::<u32>)(input)?;
            match self.state {
                ConnectionState::Handshake => handshake::read_packet(self)(data),
                ConnectionState::Status => status::read_packet(self)(data),
                ConnectionState::Login => login::read_packet(self)(data),
                ConnectionState::Play => todo!(),
            }?;
            Ok((input, ()))
        })(input)
    }
}