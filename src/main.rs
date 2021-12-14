use std::sync::Arc;

use color_eyre::Help;
use eyre::Context;
use log::LevelFilter;
use net::auth::Keys;
use server::{Server, ServerHook};
use tokio::{net::TcpListener, spawn, sync::mpsc};
use tracing::{info, instrument};

use crate::{
    config::Config,
    net::Connection,
    tui::{ControlFlow, Tui},
};

mod config;
mod data;
pub mod net;
mod nom;
pub mod server;
mod tui;
pub mod varint;

#[tokio::main]
#[instrument]
async fn main() -> eyre::Result<()> {
    setup()?;
    spawn(server_main());

    let mut tui = Tui::new()?;

    info!("hieronymus v2");

    loop {
        match tui.tick()? {
            ControlFlow::Halt => break,
            ControlFlow::Continue => continue,
        }
    }
    tui.cleanup()?;

    Ok(())
}

fn setup() -> eyre::Result<()> {
    dotenv::dotenv().ok();
    let log = dotenv::var("RUST_LOG").ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or(LevelFilter::Info);
        
    tui_logger::init_logger(LevelFilter::Trace)?;
    tui_logger::set_default_level(log);
    tui_logger::set_log_file("hieronymus.log")?;

    color_eyre::install()?;
    Ok(())
}

#[instrument]
async fn server_main() -> eyre::Result<()> {
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

#[instrument(skip_all)]
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
