use config::ConfigError;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    pub http: HttpConfig,
    pub db: DatabaseConfig,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: Option<u16>
}

pub async fn load() -> Result<(), ConfigError> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("US").separator("_"))
        .build()?
        .try_deserialize::<Config>()?;

    *CONFIG.write().await = config;

    Ok(())
}
