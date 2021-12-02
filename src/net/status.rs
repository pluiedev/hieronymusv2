use nom::IResult;
use nom_derive::Nom;
use tokio::io::AsyncWriteExt;
use tracing::{instrument, trace};

use crate::{match_id_and_forward, varint};

use super::{BoxedPacket, Connection, Packet};
use async_trait::async_trait;

pub fn read_packet<'data>(input: &'data [u8]) -> IResult<&'data [u8], BoxedPacket<'data>> {
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
        let len = status.len() as u32;
        trace!(?status, ?len);

        let mut resp = vec![0u8];
        varint::serialize_and_append(len, &mut resp)?;
        resp.extend_from_slice(status.as_bytes());

        let length_bytes = varint::serialize_to_bytes(resp.len() as u32)?;
        conn.socket.write(&length_bytes).await?;
        trace!(?length_bytes);
        conn.socket.write(&resp).await?;
        trace!(?resp);
        Ok(())
    }
}

#[derive(Debug, Nom)]
struct Ping(u64);
#[async_trait]
impl Packet for Ping {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        let mut resp = vec![1u8];
        resp.extend_from_slice(&self.0.to_be_bytes());

        let length_bytes = varint::serialize_to_bytes(resp.len() as u32)?;
        conn.socket.write(&length_bytes).await?;
        trace!(?length_bytes);
        conn.socket.write(&resp).await?;
        trace!(?resp);
        Ok(())
    }
}
