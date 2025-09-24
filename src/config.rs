use std::{collections::HashMap, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub twitch: TwitchConfig,
    pub spotify: SpotifyConfig,
    pub commands: CommandsConfig,
}

#[derive(Debug, Deserialize)]
pub struct TwitchConfig {
    pub username: String,
    pub channel: String,
    pub oauth_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub reward_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandsConfig {
    pub simple: HashMap<String, String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}
