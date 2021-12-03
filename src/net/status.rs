use nom::IResult;
use nom_derive::Nom;
use tracing::{instrument, trace};

use crate::match_id_and_forward;

use super::{BoxedPacket, Connection, Packet, RequestBuilder};
use async_trait::async_trait;

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
    match_id_and_forward! {
        input;
        0 => Status,
        1 => Ping
    }
}

#[derive(Debug, Nom)]
struct Status;
#[async_trait]
impl Packet for Status {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        let status = conn.server.get_server_status().await?;
        trace!(?status);

        RequestBuilder::new(0).var_blob(&status).send(conn).await?;
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct Ping(u64);
#[async_trait]
impl Packet for Ping {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        RequestBuilder::new(1).u64(self.0).send(conn).await?;

        Ok(())
    }
}
