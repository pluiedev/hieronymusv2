use std::{
    fs::{self, File},
    path::{Path, PathBuf}, io::Write, time::SystemTime,
};

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_is_online")]
    pub online_mode: bool,
    #[serde(default = "Config::default_max_players")]
    pub max_players: usize,
    #[serde(default = "Config::default_motd")]
    pub motd: String,
    #[serde(default = "Config::default_favicon_path")]
    pub favicon_path: PathBuf,
}

impl Config {
    pub const DEFAULT_PATH: &'static str = "./config.toml";

    pub fn read_from_default_path() -> eyre::Result<Self> {
        Self::read_from(Self::DEFAULT_PATH)
    }
    pub fn read_from<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        match fs::read_to_string(&path) {
            Ok(s) => {
                let config = toml::from_str(&s)?;
                debug!(?config, "Read config");
                Ok(config)
            }
            Err(_) => {
                warn!("Config file not found! Creating a default one...");
                let default = include_str!("config/default_config.toml");
                let mut file = File::create(&path)?;
                let timestamp = format!("# {}\n", humantime::format_rfc3339(SystemTime::now()));
                file.write(&timestamp.into_bytes())?;
                file.write(default.as_bytes())?;
                let default = toml::from_str(default)?;
                Ok(default)
            }
        }
    }
    fn default_is_online() -> bool {
        true
    }
    fn default_max_players() -> usize {
        20
    }
    fn default_motd() -> String {
        "Just another impostor Minecraft server".into()
    }
    fn default_favicon_path() -> PathBuf {
        "favicon.png".into()
    }
}