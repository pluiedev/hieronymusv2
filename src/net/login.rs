use aes::cipher::NewCipher;
use eyre::bail;
use nom::IResult;
use nom_derive::Nom;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use tracing::{debug, instrument, trace};
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    match_id_and_forward,
    net::AesCipher,
    nom::{maybe, var_bytes, var_str_with_max_length},
};

use super::{BoxedPacket, Connection, ConnectionState, Packet, RequestBuilder};
use async_trait::async_trait;

const SERVER_ID: &[u8] = b"hiero|rejectnormalcy";

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
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
                .var_blob(SERVER_ID)
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
            debug!("Login successful: transitioning into Play state");
            conn.state = ConnectionState::Play;

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
    #[instrument(skip(self, conn))]
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

        let mut sha1 = Sha1::new();
        sha1.update(SERVER_ID);
        sha1.update(&shared_secret);
        sha1.update(&conn.keys.pub_key_der);
        let hash = sha1.digest();
        let hash = minecraft_style_crappy_hash(&hash.bytes());
        trace!(?hash);

        let url = format!(
            "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}",
            auth_session.username, hash
        );
        trace!(?url);
        let auth_response: AuthResponse = reqwest::get(url).await?.json().await?;
        trace!(?auth_response);

        debug!("Login successful: transitioning into Play state");

        conn.state = ConnectionState::Play;
        conn.cipher = Some(AesCipher::new_from_slices(&shared_secret, &shared_secret)?);

        RequestBuilder::new(2)
            .u128(auth_response.id.as_u128())
            .var_blob(&auth_response.name)
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

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    id: Uuid,
    name: String,
}

fn minecraft_style_crappy_hash(input: &[u8]) -> String {
    if input[0] & 0x80 == 0x80 {
        // negative
        let mut input = input.to_vec();

        let mut carry = true;
        // two's complement in-place.
        // yes, I know this is cursed.
        // Minecraft is cursed, and we are cursed.
        for i in input.iter_mut().rev() {
            *i = !*i;
            if carry {
                carry = *i == 0xff;
                *i += 1;
            }
        }

        format!("-{}", hex::encode(input).trim_start_matches('0'))
    } else {
        hex::encode(input).trim_start_matches('0').to_string()
    }
}

#[cfg(test)]
mod tests {
    use sha1::Sha1;

    use crate::net::login::minecraft_style_crappy_hash;

    #[test]
    fn test_crappy_hash() {
        tracing_subscriber::fmt::init();
        test(b"Notch", "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48");
        test(b"jeb_", "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1");
        test(b"simon", "88e16a1019277b15d58faf0541e11910eb756f6");
    }

    fn test(input: &[u8], expected: &str) {
        let mut sha1 = Sha1::new();
        sha1.update(input);
        assert_eq!(
            minecraft_style_crappy_hash(&sha1.digest().bytes()),
            expected
        );
    }
}
