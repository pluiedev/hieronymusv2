use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
pub struct Server {
    rx: mpsc::Receiver<ServerEvent>,
    version: Version,
    players: Players,
}

impl Server {
    pub fn new(rx: mpsc::Receiver<ServerEvent>, max_players: usize) -> Self {
        Server {
            rx,
            version: Version::CURRENT,
            players: Players {
                maximum: max_players,
                players: vec![],
            },
        }
    }

    pub async fn event_loop(&mut self) {
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
                                "max": self.players.maximum,
                                "online": self.players.num_online(),
                                "sample": self.players.players.iter().take(5).collect::<Vec<_>>()
                            }
                        });
                        let json = serde_json::to_string(&json).unwrap();
                        tx.send(json).unwrap()
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

pub struct Players {
    maximum: usize,
    players: Vec<Player>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    username: String,
    uuid: String, //Uuid,
}
impl Players {
    fn num_online(&self) -> usize {
        self.players.len()
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
}

#[derive(Debug)]
pub struct ServerEvent(Inner);
#[derive(Debug)]
enum Inner {
    GetServerStatus { tx: oneshot::Sender<String> },
}
