#![feature(never_type)]

use std::net::SocketAddr;

use eyre::bail;
use server::{Server, ServerRequest};
use tokio::{io::{AsyncReadExt, BufReader}, net::{TcpListener, TcpStream, ToSocketAddrs}, spawn, sync::mpsc::{self, Sender}};
use tracing::{debug, info, instrument};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::net::Connection;

pub mod net;
pub mod nom;
pub mod server;
pub mod varint;

#[tokio::main]
#[instrument]
async fn main() -> eyre::Result<()> {
    setup()?;

    info!("hieronymus v2");

    let server = Server {};

    let (tx, mut rx) = mpsc::channel(100);

    spawn(listener_thread("127.0.0.1:25565", tx));

    loop {
        while let Some(req) = rx.recv().await {
            match req {

            }
        }
    }
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

async fn listener_thread(addr: impl ToSocketAddrs, tx: Sender<ServerRequest>) -> eyre::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to listen on address");

    while let Ok((socket, addr)) = listener.accept().await {
        let conn = Connection::new(socket, addr, tx.clone());
        spawn(async move { conn.connection_loop().await.unwrap() });
    }
    Ok(())
}