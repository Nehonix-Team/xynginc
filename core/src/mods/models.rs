use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub domains: Vec<DomainConfig>,
    #[serde(default)]
    pub auto_reload: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainConfig {
    pub domain: String,
    pub port: u16,
    #[serde(default)]
    pub ssl: bool,
    pub email: Option<String>,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: String,
}

fn default_max_body_size() -> String {
    "20M".to_string()
}

fn default_host() -> String {
    "localhost".to_string()
}
