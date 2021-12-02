use server::{Server, ServerHook};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    spawn,
    sync::mpsc,
};
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::net::Connection;

pub mod net;
pub mod server;
pub mod varint;
mod nom;

#[tokio::main]
#[instrument]
async fn main() -> eyre::Result<()> {
    setup()?;

    info!("hieronymus v2");

    let (tx, rx) = mpsc::channel(100);
    let server = Server::new(rx, 69);
    let hook = ServerHook(tx);

    spawn(listener_thread("127.0.0.1:25565", hook));

    server.event_loop().await?;

    Ok(())
}

fn setup() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish()
        .with(ErrorLayer::default());

    tracing::subscriber::set_global_default(subscriber)?;

    color_eyre::install()?;
    Ok(())
}

async fn listener_thread(addr: impl ToSocketAddrs, tx: ServerHook) -> eyre::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to listen on address");

    while let Ok((socket, _addr)) = listener.accept().await {
        let conn = Connection::new(socket, tx.clone());
        spawn(async move { conn.connection_loop().await.unwrap() });
    }
    Ok(())
}
