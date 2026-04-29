pub mod discord;
pub mod email;

use crate::config::ChannelConfig;
use crate::notification::Notification;
use async_trait::async_trait;

#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn send(&self, notification: &Notification) -> Result<(), String>;
}

fn get_string(extra: &std::collections::HashMap<String, serde_yaml::Value>, key: &str) -> Option<String> {
    extra.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn get_u16(extra: &std::collections::HashMap<String, serde_yaml::Value>, key: &str) -> Option<u16> {
    extra.get(key).and_then(|v| v.as_u64()).map(|n| n as u16)
}

pub async fn build_channels(configs: &[ChannelConfig]) -> Vec<Box<dyn Channel>> {
    let mut channels: Vec<Box<dyn Channel>> = Vec::new();

    for cfg in configs {
        let ch: Option<Box<dyn Channel>> = match cfg.channel_type.as_str() {
            "email" => {
                let smtp_host = match get_string(&cfg.extra, "smtp_host") {
                    Some(h) => h,
                    None => {
                        log::error!("channel '{}': missing smtp_host", cfg.name);
                        continue;
                    }
                };
                let smtp_port = get_u16(&cfg.extra, "smtp_port").unwrap_or(587);
                let username = get_string(&cfg.extra, "username").unwrap_or_default();
                let password = get_string(&cfg.extra, "password").unwrap_or_default();
                let from = match get_string(&cfg.extra, "from") {
                    Some(f) => f,
                    None => {
                        log::error!("channel '{}': missing from", cfg.name);
                        continue;
                    }
                };
                let to = match get_string(&cfg.extra, "to") {
                    Some(t) => t,
                    None => {
                        log::error!("channel '{}': missing to", cfg.name);
                        continue;
                    }
                };
                let format = get_string(&cfg.extra, "format").unwrap_or_else(|| "text".to_string());

                match email::EmailChannel::new(
                    cfg.name.clone(),
                    &smtp_host,
                    smtp_port,
                    &username,
                    &password,
                    &from,
                    &to,
                    &format,
                ) {
                    Ok(ch) => {
                        log::info!("channel '{}' (email) loaded: {} -> {}", cfg.name, from, to);
                        Some(Box::new(ch))
                    }
                    Err(e) => {
                        log::error!("channel '{}': {}", cfg.name, e);
                        continue;
                    }
                }
            }
            "discord" => {
                let webhook_url = match get_string(&cfg.extra, "webhook_url") {
                    Some(u) => u,
                    None => {
                        log::error!("channel '{}': missing webhook_url", cfg.name);
                        continue;
                    }
                };
                let ch = discord::DiscordChannel::new(cfg.name.clone(), webhook_url);
                log::info!("channel '{}' (discord) loaded", cfg.name);
                Some(Box::new(ch))
            }
            "telegram" => {
                log::warn!("channel '{}': telegram not yet implemented", cfg.name);
                None
            }
            other => {
                log::warn!("channel '{}': unknown type '{}', skipping", cfg.name, other);
                None
            }
        };

        if let Some(ch) = ch {
            channels.push(ch);
        }
    }

    channels
}
