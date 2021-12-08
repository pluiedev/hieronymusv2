use std::sync::Arc;

use rand::rngs::OsRng;
use rsa::{PublicKeyParts, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use thiserror::Error;
use tracing::trace;
use uuid::Uuid;

use crate::net::Connection;

#[derive(Clone)]
pub struct Keys {
    pub priv_key: Arc<RsaPrivateKey>,
    pub pub_key_der: Arc<Vec<u8>>,
}

impl Keys {
    pub fn new() -> rsa::errors::Result<Self> {
        let priv_key = Arc::new(RsaPrivateKey::new(&mut OsRng, 1024)?);
        let pub_key_der = Arc::new(rsa_der::public_key_to_der(
            &priv_key.n().to_bytes_be(),
            &priv_key.e().to_bytes_be(),
        ));
        Ok(Self {
            priv_key,
            pub_key_der,
        })
    }
}

pub type VerifyToken = [u8; 8];

#[derive(Debug)]
pub struct AuthSession {
    pub username: String,
    pub verify_token: VerifyToken,
}
impl AuthSession {
    pub fn new(username: String) -> Self {
        AuthSession {
            username,
            verify_token: rand::random(),
        }
    }
}

pub const SERVER_ID: &[u8] = b"hiero|rejectnormalcy";

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Not in auth session! This is a bug!")]
    NotInAuthSession,
    #[error("Mismatched verify token – client is either malicious or hilariously non-compliant!")]
    MismatchedVerifyToken,
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

pub async fn authenticate(
    conn: &mut Connection,
    shared_secret: &[u8],
    verify_token: &[u8],
) -> Result<AuthResponse, AuthenticationError> {
    let auth_session = conn
        .auth_session
        .as_ref()
        .ok_or(AuthenticationError::NotInAuthSession)?;

    if !verify_token.starts_with(&auth_session.verify_token) {
        return Err(AuthenticationError::MismatchedVerifyToken);
    }

    let mut sha1 = Sha1::new();
    sha1.update(SERVER_ID);
    sha1.update(shared_secret);
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
    Ok(auth_response)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub id: Uuid,
    pub name: String,
}

/// Minecraft's crappy hash algorithm that isn't found *anywhere*.
/// Mojang, are you ok?
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

    use super::minecraft_style_crappy_hash;

    #[test]
    fn test_crappy_hash() {
        tracing_subscriber::fmt::init();
        // shamelessly stolen from wiki.vg
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
