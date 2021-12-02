use std::sync::Arc;

use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, PublicKeyParts};

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

#[derive(Debug)]
pub struct AuthSession {
    pub username: String,
    pub verify_token: [u8; 8],
}
impl AuthSession {
    pub fn new(username: String) -> Self {
        AuthSession {
            username,
            verify_token: rand::random(),
        }
    }
}
