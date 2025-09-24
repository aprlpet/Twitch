mod bot;
mod commands;
mod config;
mod error;

use bot::TwitchBot;
use config::Config;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    info!("Starting Twitch bot");

    let config = Config::load()?;
    let mut bot = TwitchBot::new(config);

    bot.connect().await?;

    if let Err(e) = bot.run().await {
        error!("Bot error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
