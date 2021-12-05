use std::sync::Arc;

use auth::Keys;
use color_eyre::Help;
use eyre::Context;
use server::{Server, ServerHook};
use tokio::{net::TcpListener, spawn, sync::mpsc};
use tracing::{info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{config::Config, net::Connection};

mod auth;
mod config;
pub mod net;
mod nom;
pub mod server;
pub mod varint;

#[tokio::main]
#[instrument]
async fn main() -> eyre::Result<()> {
    setup()?;

    info!("hieronymus v2");

    let (tx, rx) = mpsc::channel(100);

    let keys = Keys::new()?;
    let config = Arc::new(Config::read_from_default_path()?);
    let server = Server::new(rx, config.clone()).await?;
    let hook = ServerHook(tx);

    let listener = TcpListener::bind("127.0.0.1:25565")
        .await
        .wrap_err("Failed to listen on address; is the port occupied?")
        .suggestion("Please use a different address to listen on")?;

    spawn(listener_thread(listener, hook, keys, config));

    server.server_loop().await?;

    Ok(())
}

fn setup() -> eyre::Result<()> {
    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("`info` is not a valid EnvFilter... what?");

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::install()?;
    Ok(())
}

async fn listener_thread(
    listener: TcpListener,
    tx: ServerHook,
    keys: Keys,
    config: Arc<Config>,
) -> eyre::Result<()> {
    while let Ok((socket, _addr)) = listener.accept().await {
        let conn = Connection::new(socket, tx.clone(), keys.clone(), config.clone());
        spawn(async move { conn.connection_loop().await.unwrap() });
    }
    Ok(())
}
