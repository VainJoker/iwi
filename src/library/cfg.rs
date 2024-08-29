use std::{fmt::Debug, fs, sync::OnceLock};

// use config::Config;
use serde::{Deserialize, Serialize};

// Create a static lock for the configuration, ensuring
// that it's only initialized once across the entire application.
static CFG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub log: LogConfig,
    pub app: AppConfig,
    pub mail: MailConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub path: String,
    pub mine_formatting_level: String,
    pub other_formatting_level: String,

    pub mine_file: String,
    pub other_file: String,
    pub database_file: String,
    pub error_file: String,

    pub file_level: String,

    pub mine_target: String,
    pub database_target: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MailConfig {
    pub username: String,
    pub password: String,
    pub host: String,
}

impl Debug for MailConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("username", &self.username)
            .field("password", &"&self.password")
            .field("host", &self.host)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JWTConfig {
    pub secret: String,
    pub secret_expiration: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub env: String,
    pub host: String,
    pub port: usize,
    pub db_url: String,
    pub redis_url: String,
    pub redis_prefix: String,
    pub mq_url: String,
    pub access_token: JWTConfig,
    pub refresh_token: JWTConfig,
}

/// Initializes the application's configuration from the provided file.
/// Expected to be run on startup of the application.
pub fn init(cfg_file: &String) {
    // Attempt to extract the canonical, absolute path of the configuration
    // file. Panic if this operation fails, as the configuration is critical
    // for execution.
    let path = fs::canonicalize(cfg_file).unwrap_or_else(|e| {
        panic!("💥 Failed to load configuration file: {e} - {cfg_file}");
    });

    // Attempt to build the configuration from the file.
    // Panic if any errors occur during loading or validation.
    let cfg = config::Config::builder()
        .add_source(config::File::with_name(path.to_str().unwrap()))
        .build()
        .unwrap_or_else(|e| {
            panic!("💥 Failed to build configuration: {e}");
        });

    let pay: Config = cfg.try_deserialize().unwrap_or_else(|e| {
        panic!("💥 Failed to deserialize configuration: {e}");
    });
    // Attempt to lock the configuration for the first time.
    // Ignore the result because we'd panic if locking fails.
    let _ = CFG.set(pay);
    tracing::info!("🚀 Configuration loading is successful!");
}

/// Accesses the application's configuration, once initialized.
/// Panics if called before `init`.
pub fn config() -> &'static Config {
    CFG.get().unwrap_or_else(|| {
        panic!("💥 Configuration accessed before initialization");
    })
}
