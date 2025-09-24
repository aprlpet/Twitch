#[derive(Debug, Clone)]
pub struct TwitchMessage {
    #[allow(dead_code)]
    pub username: String,
    pub message: String,
    #[allow(dead_code)]
    pub channel: String,
    pub is_moderator: bool,
    pub is_broadcaster: bool,
    pub is_vip: bool,
    pub custom_reward_id: Option<String>,
}

impl TwitchMessage {
    pub fn parse(raw_message: &str, channel: &str) -> Option<Self> {
        if !raw_message.contains("PRIVMSG") {
            return None;
        }

        let mut is_moderator = false;
        let mut is_vip = false;
        let mut custom_reward_id = None;

        let message_without_tags = if raw_message.starts_with('@') {
            let space_pos = raw_message.find(' ')?;
            let tags = &raw_message[1..space_pos];

            for tag in tags.split(';') {
                if let Some((key, value)) = tag.split_once('=') {
                    match key {
                        "badges" => {
                            is_moderator = value.contains("moderator/");
                            is_vip = value.contains("vip/");
                        }
                        "custom-reward-id" if !value.is_empty() => {
                            custom_reward_id = Some(value.to_string());
                        }
                        _ => {}
                    }
                }
            }
            &raw_message[space_pos + 1..]
        } else {
            raw_message
        };

        let parts: Vec<&str> = message_without_tags.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        let username = parts[0]
            .trim_start_matches(':')
            .split('!')
            .next()?
            .to_string();

        let parsed_channel = parts[2].trim_start_matches('#').to_string();
        let colon_pos = message_without_tags.find(" :")?;
        let message = message_without_tags[colon_pos + 2..].to_string();

        let is_broadcaster = username.to_lowercase() == channel.to_lowercase();
        if is_broadcaster {
            is_moderator = true;
        }

        Some(Self {
            username,
            message,
            channel: parsed_channel,
            is_moderator,
            is_broadcaster,
            is_vip,
            custom_reward_id,
        })
    }

    pub fn has_permissions(&self) -> bool {
        self.is_moderator || self.is_vip || self.is_broadcaster
    }
}
