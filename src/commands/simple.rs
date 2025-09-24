use async_trait::async_trait;

use super::Command;
use crate::bot::TwitchMessage;

pub struct SimpleCommand {
    name: String,
    response: String,
}

impl SimpleCommand {
    pub fn new(name: String, response: String) -> Self {
        Self { name, response }
    }
}

#[async_trait]
impl Command for SimpleCommand {
    fn name(&self) -> &str {
        &self.name
    }

    async fn execute(&self, _message: &TwitchMessage) -> Option<String> {
        Some(self.response.clone())
    }
}
