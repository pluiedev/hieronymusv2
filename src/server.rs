pub mod dimension;

use std::sync::Arc;

use eyre::eyre;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, instrument, trace};
use uuid::Uuid;

use crate::config::Config;

use self::dimension::DimensionManager;
pub struct Server {
    rx: mpsc::Receiver<ServerEvent>,
    config: Arc<Config>,
    version: Version,
    players: Vec<Player>,
    favicon: Option<String>,

    dimension_manager: DimensionManager,
}

impl Server {
    pub async fn new(rx: mpsc::Receiver<ServerEvent>, config: Arc<Config>) -> eyre::Result<Self> {
        let favicon_path = &config.favicon_path;
        trace!(?favicon_path);
        let favicon = match tokio::fs::read(favicon_path).await {
            Ok(image) => {
                let mut favicon =
                    String::with_capacity("data:image/png;base64,".len() + image.len() * 4 / 3 + 4);
                favicon.push_str("data:image/png;base64,");
                base64::encode_config_buf(image, base64::STANDARD, &mut favicon);
                Some(favicon)
            }
            Err(_) => None,
        };

        Ok(Server {
            rx,
            config,
            version: Version::CURRENT,
            players: vec![],
            favicon,
            dimension_manager: DimensionManager::new(),
        })
    }

    #[instrument(skip(self))]
    pub async fn server_loop(mut self) -> eyre::Result<()> {
        loop {
            self.handle_events().await?;
        }
    }

    #[instrument(skip(self))]
    pub async fn handle_events(&mut self) -> eyre::Result<()> {
        while let Some(ServerEvent(req)) = self.rx.recv().await {
            match req {
                Inner::GetServerStatus { tx } => {
                    let mut json = json!({
                        "version": {
                            "name": self.version.name,
                            "protocol": self.version.protocol_version,
                        },
                        "players": {
                            "max": self.config.max_players,
                            "online": self.players.len(),
                            "sample": self.players.iter().take(5).collect::<Vec<_>>()
                        },
                        "description": {
                            "text": &self.config.motd
                        },
                    });
                    if let Some(favicon) = &self.favicon {
                        json["favicon"] = json!(favicon);
                    }

                    let json = serde_json::to_string(&json)?;
                    trace!(?json);
                    tx.send(json)
                        .map_err(|_| eyre!("failed to send status data"))?;
                }
                Inner::GetDimensionInfo { tx } => {
                    let mut buf = vec![];
                    nbt::to_writer(&mut buf, &self.dimension_manager, None)?;
                    nbt::to_writer(&mut buf, &self.dimension_manager.current_dimension(), None)?;
                    tx.send(buf)
                        .map_err(|_| eyre!("failed to send dimension info"))?;
                }
                Inner::JoinGame(player) => {
                    debug!(?player, "Player joined");
                    self.players.push(player);
                }
            }
        }
        Ok(())
    }
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
    pub async fn get_dimension_info(&self) -> eyre::Result<Vec<u8>> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(ServerEvent(Inner::GetDimensionInfo { tx }))
            .await?;
        Ok(rx.await?)
    }
    pub async fn join_game(&self, player: Player) -> eyre::Result<()> {
        self.0.send(ServerEvent(Inner::JoinGame(player))).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ServerEvent(Inner);
#[derive(Debug)]
enum Inner {
    GetServerStatus { tx: oneshot::Sender<String> },
    GetDimensionInfo { tx: oneshot::Sender<Vec<u8>> },
    JoinGame(Player),
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
