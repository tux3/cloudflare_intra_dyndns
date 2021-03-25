use serde::Deserialize;
use anyhow::Result;
use directories_next::ProjectDirs;

const DEFAULT_REDIS_POLL_INTERVAL: u64 = 5000;

#[derive(Deserialize)]
struct ConfigFile {
    pub cf_token: String,
    pub zone_name: String,
    pub record_name: String,
    pub redis_host: String,
    pub redis_key: String,
    pub redis_poll_interval: Option<u64>,
}

pub struct Config {
    pub cf_token: String,
    pub zone_name: String,
    pub record_name: String,
    pub redis_host: String,
    pub redis_key: String,
    pub redis_poll_interval: u64,
}

impl Config {
    pub fn from_file() -> Result<Self> {
        let dirs = ProjectDirs::from("net",
                                     "alacrem",
                                     "cloudflare_intra_dyndns").unwrap();
        let config_dir = dirs.config_dir();
        let config_text = std::fs::read_to_string(config_dir.join("config.toml"))?;
        let config: ConfigFile = toml::from_str(&config_text)?;
        Ok(Config {
            cf_token: config.cf_token,
            zone_name: config.zone_name,
            record_name: config.record_name,
            redis_host: config.redis_host,
            redis_key: config.redis_key,
            redis_poll_interval: config.redis_poll_interval.unwrap_or(DEFAULT_REDIS_POLL_INTERVAL),
    })
    }
}
