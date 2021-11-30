use nom::{combinator::map, number::streaming::be_u16, sequence::tuple, IResult};
use tracing::{debug, instrument};
use tracing_subscriber::field::debug;

use crate::{match_id_and_forward, net::PacketHandler, nom::{connection_state, var_str}};

use super::{Connection, ConnectionState};

pub fn read_packet<'data>(
    conn: &mut Connection,
) -> impl FnMut(&'data [u8]) -> IResult<&'data [u8], ()> + '_ {
    match_id_and_forward! {
        0 => map(
            tuple((varint, var_str, be_u16, connection_state)),
            |(protocol_version, server_address, server_port, next_state)| {
                conn.handle(Handshake {
                    protocol_version,
                    server_address,
                    server_port,
                    next_state,
                })
            },
        )
    }
}

#[derive(Debug)]
pub struct Handshake<'a> {
    protocol_version: u32,
    server_address: &'a str,
    server_port: u16,
    next_state: ConnectionState,
}

impl PacketHandler<Handshake<'_>> for Connection {
    #[instrument(skip_all)]
    fn handle(&mut self, packet: Handshake<'_>) {
        debug!(?packet, before = ?self.state, after = ?packet.next_state);
        self.state = packet.next_state;
    }
}
