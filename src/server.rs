use std::sync::Arc;

use eyre::eyre;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{instrument, trace, debug};
use uuid::Uuid;

use crate::config::Config;
pub struct Server {
    rx: mpsc::Receiver<ServerEvent>,
    config: Arc<Config>,
    version: Version,
    players: Vec<Player>,
}

impl Server {
    pub fn new(rx: mpsc::Receiver<ServerEvent>, config: Arc<Config>) -> Self {
        Server {
            rx,
            config,
            version: Version::CURRENT,
            players: vec![],
        }
    }

    #[instrument(skip(self))]
    pub async fn event_loop(mut self) -> eyre::Result<()> {
        loop {
            while let Some(ServerEvent(req)) = self.rx.recv().await {
                match req {
                    Inner::GetServerStatus { tx } => {
                        let json = serde_json::json!({
                            "version": {
                                "name": self.version.name,
                                "protocol": self.version.protocol_version,
                            },
                            "players": {
                                "max": self.config.max_players,
                                "online": self.players.len(),
                                "sample": self.players.iter().take(5).collect::<Vec<_>>()
                            }
                        });
                        let json = serde_json::to_string(&json)?;
                        trace!(?json);
                        tx.send(json)
                            .map_err(|_| eyre!("failed to send status data"))?;
                    },
                    Inner::JoinGame(player) => {
                        debug!(?player, "Player joined");
                        self.players.push(player);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Version {
    pub name: &'static str,
    pub protocol_version: u32,
}
impl Version {
    const CURRENT: Self = Self {
        name: "1.17.1",
        protocol_version: 756,
    };
}
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.protocol_version == other.protocol_version
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.protocol_version.partial_cmp(&other.protocol_version)
    }
}
impl Eq for Version {}
impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.protocol_version.cmp(&other.protocol_version)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub username: String,
    pub uuid: Uuid,
}

#[derive(Clone)]
pub struct ServerHook(pub mpsc::Sender<ServerEvent>);

impl ServerHook {
    pub async fn get_server_status(&self) -> eyre::Result<String> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(ServerEvent(Inner::GetServerStatus { tx }))
            .await?;
        Ok(rx.await?)
    }
    pub async fn join_game(&self, player: Player) -> eyre::Result<()> {
        self.0
            .send(ServerEvent(Inner::JoinGame(player)))
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ServerEvent(Inner);
#[derive(Debug)]
enum Inner {
    GetServerStatus { tx: oneshot::Sender<String> },
    JoinGame(Player),
}
