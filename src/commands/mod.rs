mod simple;
mod spotify;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{bot::TwitchMessage, config::Config};

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    async fn execute(&self, message: &TwitchMessage) -> Option<String>;
}

pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
    spotify_service: spotify::SpotifyService,
}

impl CommandRegistry {
    pub fn new(config: Arc<Config>) -> Self {
        let mut commands = HashMap::new();
        let spotify_service = spotify::SpotifyService::new(Arc::clone(&config));

        Self::register_spotify_commands(&mut commands, &spotify_service);
        Self::register_simple_commands(&mut commands, &config);

        Self {
            commands,
            spotify_service,
        }
    }

    fn register_spotify_commands(
        commands: &mut HashMap<String, Arc<dyn Command>>,
        service: &spotify::SpotifyService,
    ) {
        let spotify_commands: Vec<Arc<dyn Command>> = vec![
            Arc::new(spotify::SpotifyCommand::new(service.clone())),
            Arc::new(spotify::PlayCommand::new(service.clone())),
            Arc::new(spotify::SkipCommand::new(service.clone())),
            Arc::new(spotify::PrevCommand::new(service.clone())),
        ];

        for cmd in spotify_commands {
            commands.insert(cmd.name().to_string(), Arc::clone(&cmd));
            for alias in cmd.aliases() {
                commands.insert(alias.to_string(), Arc::clone(&cmd));
            }
        }
    }

    fn register_simple_commands(commands: &mut HashMap<String, Arc<dyn Command>>, config: &Config) {
        for (name, response) in &config.commands.simple {
            let cmd = Arc::new(simple::SimpleCommand::new(name.clone(), response.clone()));
            commands.insert(name.clone(), cmd);
        }
    }

    pub async fn execute(&self, command_name: &str, message: &TwitchMessage) -> Option<String> {
        if let Some(command) = self.commands.get(command_name) {
            command.execute(message).await
        } else {
            None
        }
    }

    pub async fn handle_spotify_reward(&self, message: &str) -> Option<String> {
        self.spotify_service.add_track_from_url(message).await
    }
}
