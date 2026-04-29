use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub channels: Vec<ChannelConfig>,
    pub endpoints: Vec<EndpointConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChannelConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EndpointConfig {
    pub path: String,
    pub token: Option<String>,
    pub channels: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("failed to read config: {e}"))?;
        let config: Config = serde_yaml::from_str(&content).map_err(|e| format!("failed to parse config: {e}"))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        let channel_names: Vec<&str> = self.channels.iter().map(|c| c.name.as_str()).collect();
        for ep in &self.endpoints {
            for ch in &ep.channels {
                if !channel_names.contains(&ch.as_str()) {
                    return Err(format!("endpoint '{}': unknown channel '{}'", ep.path, ch));
                }
            }
        }
        Ok(())
    }
}
