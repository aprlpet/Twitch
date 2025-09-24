use std::sync::Arc;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use serde::Deserialize;
use tracing::error;

use super::Command;
use crate::{bot::TwitchMessage, config::Config};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct CurrentlyPlaying {
    item: Option<Track>,
    is_playing: bool,
}

#[derive(Debug, Deserialize)]
struct Track {
    name: String,
    artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
struct Artist {
    name: String,
}

#[derive(Clone)]
pub struct SpotifyService {
    config: Arc<Config>,
    client: reqwest::Client,
}

impl SpotifyService {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let auth = general_purpose::STANDARD.encode(format!(
            "{}:{}",
            self.config.spotify.client_id, self.config.spotify.client_secret
        ));

        let response = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=refresh_token&refresh_token={}",
                self.config.spotify.refresh_token
            ))
            .send()
            .await?;

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.access_token)
    }

    async fn get_currently_playing(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;

        let response = self
            .client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status() == 204 {
            return Ok(Some("😭😂✌️".to_string()));
        }

        let currently_playing: CurrentlyPlaying = response.json().await?;

        if !currently_playing.is_playing {
            return Ok(Some("😭😂✌️".to_string()));
        }

        if let Some(track) = currently_playing.item {
            let artists = track
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            Ok(Some(format!(
                "{} by {}",
                track.name.to_lowercase(),
                artists.to_lowercase()
            )))
        } else {
            Ok(Some("unable to get track information".to_string()))
        }
    }

    async fn skip_track(&self) -> Result<String, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;

        self.client
            .post("https://api.spotify.com/v1/me/player/next")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match self.get_currently_playing().await {
            Ok(Some(track_info)) => Ok(track_info),
            _ => Ok("next track".to_string()),
        }
    }

    async fn previous_track(&self) -> Result<String, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;

        self.client
            .post("https://api.spotify.com/v1/me/player/previous")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        match self.get_currently_playing().await {
            Ok(Some(track_info)) => Ok(track_info),
            _ => Ok("previous track".to_string()),
        }
    }

    async fn add_to_queue(&self, uri: &str) -> Result<String, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let track_id = uri.replace("spotify:track:", "");

        let track_response = self
            .client
            .get(&format!("https://api.spotify.com/v1/tracks/{}", track_id))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        let track: Track = track_response.json().await?;
        let artists = track
            .artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        self.client
            .post("https://api.spotify.com/v1/me/player/queue")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .query(&[("uri", uri)])
            .json(&serde_json::json!({}))
            .send()
            .await?;

        Ok(format!(
            "{} by {}",
            track.name.to_lowercase(),
            artists.to_lowercase()
        ))
    }

    pub async fn add_track_from_url(&self, url: &str) -> Option<String> {
        let track_id = url.split("/track/").nth(1)?.split('?').next()?;

        let uri = format!("spotify:track:{}", track_id);

        match self.add_to_queue(&uri).await {
            Ok(track_info) => Some(format!("{} has been added to the queue :3", track_info)),
            Err(e) => {
                error!("Failed to add track: {}", e);
                Some("😭😂✌️".to_string())
            }
        }
    }
}

pub struct SpotifyCommand {
    service: SpotifyService,
}

impl SpotifyCommand {
    pub fn new(service: SpotifyService) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Command for SpotifyCommand {
    fn name(&self) -> &str {
        "spotify"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["song", "music", "np", "nowplaying"]
    }

    async fn execute(&self, _message: &TwitchMessage) -> Option<String> {
        match self.service.get_currently_playing().await {
            Ok(response) => response,
            Err(e) => {
                error!("Spotify error: {}", e);
                Some("error connecting to spotify".to_string())
            }
        }
    }
}

pub struct PlayCommand {
    service: SpotifyService,
}

impl PlayCommand {
    pub fn new(service: SpotifyService) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Command for PlayCommand {
    fn name(&self) -> &str {
        "play"
    }

    async fn execute(&self, message: &TwitchMessage) -> Option<String> {
        if !message.has_permissions() {
            return Some("😭😂✌️".to_string());
        }

        let parts: Vec<&str> = message.message.split_whitespace().collect();
        if parts.len() < 2 {
            return Some("😭😂✌️".to_string());
        }

        self.service.add_track_from_url(parts[1]).await
    }
}

pub struct SkipCommand {
    service: SpotifyService,
}

impl SkipCommand {
    pub fn new(service: SpotifyService) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Command for SkipCommand {
    fn name(&self) -> &str {
        "skip"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["next"]
    }

    async fn execute(&self, message: &TwitchMessage) -> Option<String> {
        if !message.has_permissions() {
            return Some("😭😂✌️".to_string());
        }

        match self.service.skip_track().await {
            Ok(track_info) => Some(format!("skipped to {}", track_info)),
            Err(e) => {
                error!("Skip error: {}", e);
                Some("😭😂✌️".to_string())
            }
        }
    }
}

pub struct PrevCommand {
    service: SpotifyService,
}

impl PrevCommand {
    pub fn new(service: SpotifyService) -> Self {
        Self { service }
    }
}

#[async_trait]
impl Command for PrevCommand {
    fn name(&self) -> &str {
        "prev"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["previous", "back"]
    }

    async fn execute(&self, message: &TwitchMessage) -> Option<String> {
        if !message.has_permissions() {
            return Some("😭😂✌️".to_string());
        }

        match self.service.previous_track().await {
            Ok(track_info) => Some(format!("went back to {}", track_info)),
            Err(e) => {
                error!("Previous error: {}", e);
                Some("😭😂✌️".to_string())
            }
        }
    }
}
