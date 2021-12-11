use aes::cipher::NewCipher;
use nom::IResult;
use nom_derive::Nom;
use tracing::{debug, instrument, trace};
use uuid::Uuid;

use crate::{
    match_id_and_forward,
    net::{
        auth::{AuthSession, SERVER_ID},
        AesCipher,
    },
    nom::{var_bytes, var_str_with_max_length},
    server::Player,
};

use super::{auth, BoxedPacket, Connection, ConnectionState, Packet, ResponseBuilder};
use async_trait::async_trait;

pub fn read_packet(input: &[u8]) -> IResult<&[u8], BoxedPacket<'_>> {
    match_id_and_forward! {
        input;
        0 => LoginStart,
        1 => EncryptionResponse
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
        if conn.config.online_mode {
            let auth_session = conn
                .auth_session
                .insert(AuthSession::new(self.username.into()));
            let pub_key = conn.keys.pub_key_der.as_ref();
            let verify_token = &auth_session.verify_token;
            trace!(?auth_session, ?pub_key, ?verify_token);

            ResponseBuilder::new(1)
                .var_data(SERVER_ID)
                .var_data(pub_key)
                .var_data(verify_token)
                .send(conn)
                .await?;
        } else {
            let player = Player {
                uuid: Uuid::new_v4(),
                username: self.username.to_string(),
            };
            conn.login_success(player, None, None).await?;
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
        let shared_secret = conn
            .keys
            .priv_key
            .decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, self.shared_secret)?;
        let verify_token = conn
            .keys
            .priv_key
            .decrypt(rsa::PaddingScheme::PKCS1v15Encrypt, self.verify_token)?;
        trace!(?shared_secret, ?verify_token);

        let auth_response = auth::authenticate(conn, &shared_secret, &verify_token).await?;

        // Success! 🎉
        let player = Player {
            username: auth_response.name,
            uuid: auth_response.id,
        };
        let encrypt_cipher = AesCipher::new_from_slices(&shared_secret, &shared_secret)?;
        let decrypt_cipher = AesCipher::new_from_slices(&shared_secret, &shared_secret)?;
        conn.login_success(player, Some(encrypt_cipher), Some(decrypt_cipher)).await?;
        Ok(())
    }
}

impl Connection {
    #[instrument(skip(self, encrypt_cipher, decrypt_cipher))]
    async fn login_success(
        &mut self,
        player: Player,
        encrypt_cipher: Option<AesCipher>,
        decrypt_cipher: Option<AesCipher>,
    ) -> eyre::Result<()> {
        debug!("Login successful: transitioning into Play state");
        self.encrypt_cipher = encrypt_cipher;
        self.decrypt_cipher = decrypt_cipher;
        self.state = ConnectionState::Play;

        // Login success
        ResponseBuilder::new(2)
            .add(player.uuid)
            .add(&player.username)
            .send(self)
            .await?;

        self.join_game(player).await?;
        Ok(())
    }
}
