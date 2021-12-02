use block_modes::BlockMode;
use eyre::bail;
use nom::IResult;
use nom_derive::Nom;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    match_id_and_forward,
    net::AesCipher,
    nom::{maybe, var_bytes, var_str_with_max_length},
};

use super::{BoxedPacket, Connection, Packet, RequestBuilder};
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
        let auth_session = conn
            .auth_session
            .insert(AuthSession::new(self.username.into()));
        let pub_key = conn.keys.pub_key_der.as_ref();
        let verify_token = &auth_session.verify_token;
        trace!(?auth_session, ?pub_key, ?verify_token);

        if conn.config.is_online {
            RequestBuilder::new(1)
                .var_blob("hiero|rejectnormalcy")
                .var_blob(pub_key)
                .var_blob(verify_token)
                .send(conn)
                .await?;
        } else {
            let player_uuid = Uuid::new_v4();
            RequestBuilder::new(2)
                .u128(player_uuid.as_u128())
                .var_blob(&auth_session.username)
                .send(conn)
                .await?;

            // prematurely kick
            conn.kick(
                r#"{"text":"well... i haven't implemented like, the game yet lol. come back later XD"}"#
            ).await?;
        }

        Ok(())
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
        let auth_session = conn.auth_session.as_ref().unwrap();

        let shared_secret = conn
            .keys
            .priv_key
            .decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, self.shared_secret)?;
        let verify_token = conn
            .keys
            .priv_key
            .decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, self.verify_token)?;
        trace!(?shared_secret, ?verify_token);

        if !verify_token.starts_with(&auth_session.verify_token) {
            bail!("Mismatched verify token – client is either malicious or hilariously non-compliant!")
        }

        conn.cipher = Some(AesCipher::new_from_slices(&shared_secret, &shared_secret)?);
        let player_uuid = Uuid::new_v4();

        RequestBuilder::new(2)
            .u128(player_uuid.as_u128())
            .var_blob(&auth_session.username)
            .send(conn)
            .await?;
        // prematurely kick
        conn.kick(
            r#"{"text":"well... i haven't implemented like, the game yet lol. come back later XD"}"#
        ).await?;
        Ok(())
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
