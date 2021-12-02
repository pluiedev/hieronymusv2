use nom::IResult;
use nom_derive::Nom;
use tracing::instrument;

use crate::{
    match_id_and_forward,
    nom::{maybe, var_bytes, var_str_with_max_length},
};

use super::{BoxedPacket, Connection, Packet};
use async_trait::async_trait;

pub fn read_packet<'data>(input: &'data [u8]) -> IResult<&'data [u8], BoxedPacket<'data>> {
    match_id_and_forward! {
        input;
        0 => LoginStart,
        1 => EncryptionResponse,
        2 => LoginPluginResponse
    }
}

#[derive(Debug, Nom)]
struct LoginStart<'a> {
    #[nom(Parse = "var_str_with_max_length(16u32)")]
    username: &'a str,
}
#[async_trait]
impl Packet for LoginStart<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        todo!()
    }
}

#[derive(Debug, Nom)]
struct EncryptionResponse<'a> {
    #[nom(Parse = "var_bytes")]
    shared_secret: &'a [u8],
    #[nom(Parse = "var_bytes")]
    verify_token: &'a [u8],
}
#[async_trait]
impl Packet for EncryptionResponse<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        todo!()
    }
}

#[derive(Debug, Nom)]
struct LoginPluginResponse<'a> {
    message_id: u32,
    #[nom(Parse = "maybe(var_bytes)")]
    data: Option<&'a [u8]>,
}
#[async_trait]
impl Packet for LoginPluginResponse<'_> {
    #[instrument(skip(conn))]
    async fn handle(&self, conn: &mut Connection) -> eyre::Result<()> {
        todo!()
    }
}
