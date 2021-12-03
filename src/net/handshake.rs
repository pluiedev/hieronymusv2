use async_trait::async_trait;
use nom::IResult;
use nom_derive::Nom;
use tracing::{debug, instrument};

use crate::{
    match_id_and_forward,
    nom::{connection_state, var_str},
    varint::varint,
};

use super::{BoxedPacket, Connection, ConnectionState, Packet};

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
    match_id_and_forward! {
        input;
        0 => Handshake
    }
}

#[derive(Debug, Nom)]
struct Handshake<'a> {
    #[nom(Parse = "varint")]
    _protocol_version: u32,
    #[nom(Parse = "var_str")]
    _server_address: &'a str,
    _server_port: u16,
    #[nom(Parse = "connection_state")]
    next_state: ConnectionState,
}

#[async_trait]
impl Packet for Handshake<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        debug!(current = ?conn.state, next = ?self.next_state, "handshake - advancing to next state");
        conn.state = self.next_state;
        Ok(())
    }
}
