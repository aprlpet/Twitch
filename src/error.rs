use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum BotError {
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Spotify API error: {status}: {message}")]
    SpotifyApi { status: u16, message: String },

    #[error("Authentication failed")]
    AuthenticationFailed,
}

pub type Result<T> = std::result::Result<T, BotError>;
