pub mod message;

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
pub use message::TwitchMessage;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use tracing::{debug, error, info, warn};

use crate::{
    commands::CommandRegistry,
    config::Config,
    error::{BotError, Result},
};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct TwitchBot {
    config: Arc<Config>,
    websocket: Option<WebSocket>,
    commands: CommandRegistry,
}

impl TwitchBot {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let commands = CommandRegistry::new(Arc::clone(&config));

        Self {
            config,
            websocket: None,
            commands,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Twitch IRC");

        let (ws_stream, _) = connect_async("wss://irc-ws.chat.twitch.tv:443").await?;
        self.websocket = Some(ws_stream);

        self.send_raw(&format!("PASS {}", self.config.twitch.oauth_token))
            .await?;
        self.send_raw(&format!("NICK {}", self.config.twitch.username))
            .await?;
        self.send_raw("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands")
            .await?;
        self.send_raw(&format!("JOIN #{}", self.config.twitch.channel))
            .await?;

        info!("Connected to channel: #{}", self.config.twitch.channel);
        Ok(())
    }

    async fn send_raw(&mut self, message: &str) -> Result<()> {
        if let Some(ws) = &mut self.websocket {
            ws.send(Message::Text(message.into())).await?;
            debug!("Sent: {}", message);
        }
        Ok(())
    }

    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        self.send_raw(&format!(
            "PRIVMSG #{} :{}",
            self.config.twitch.channel, message
        ))
        .await
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(ws) = &mut self.websocket {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    if let Err(e) = self.handle_message(&text).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Some(Ok(Message::Close(_))) => {
                    warn!("WebSocket connection closed");
                    break;
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                None => {
                    warn!("WebSocket stream ended");
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }

    async fn handle_message(&mut self, raw_message: &str) -> Result<()> {
        debug!("Received: {}", raw_message);

        if raw_message.contains("Login authentication failed") {
            return Err(BotError::AuthenticationFailed);
        }

        if raw_message.starts_with("PING") {
            self.send_raw(&raw_message.replace("PING", "PONG")).await?;
            return Ok(());
        }

        if let Some(message) = TwitchMessage::parse(raw_message, &self.config.twitch.channel) {
            if message.custom_reward_id.as_ref() == Some(&self.config.spotify.reward_id) {
                self.handle_spotify_reward(&message).await?;
            }

            if message.message.starts_with('!') {
                self.handle_command(&message).await?;
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, message: &TwitchMessage) -> Result<()> {
        let command_text = message.message.trim_start_matches('!');
        let command_name = command_text.split_whitespace().next().unwrap_or("");

        if let Some(response) = self.commands.execute(command_name, message).await {
            self.send_message(&response).await?;
        }

        Ok(())
    }

    async fn handle_spotify_reward(&mut self, message: &TwitchMessage) -> Result<()> {
        if message.message.contains("open.spotify.com/track/") {
            if let Some(response) = self.commands.handle_spotify_reward(&message.message).await {
                self.send_message(&response).await?;
            }
        } else {
            self.send_message("😭😂✌️").await?;
        }
        Ok(())
    }
}
