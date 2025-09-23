use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, from_value};
use std::fs;
use std::sync::OnceLock;

// Define your config structure to match config.json
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database_path: String,
    pub logging: LoggingConfig,
    pub radarr: RadarrConfig,
}

// Hierarchical access modules
pub mod app {
    use super::*;

    pub fn config() -> Result<AppConfig, ConfigError> {
        let config = CONFIG
            .get()
            .ok_or_else(|| ConfigError::ValidationError("Config not initialized".to_string()))?;

        from_value(config.clone())
            .map_err(|e| ConfigError::TypeError(format!("Cannot deserialize config: {}", e)))
    }

    pub mod server {
        use super::*;

        pub fn config() -> Result<ServerConfig, ConfigError> {
            super::config().map(|c| c.server)
        }

        pub fn host() -> Result<String, ConfigError> {
            config().map(|c| c.host)
        }

        pub fn port() -> Result<u16, ConfigError> {
            config().map(|c| c.port)
        }
    }

    pub mod logging {
        use super::*;

        pub fn config() -> Result<LoggingConfig, ConfigError> {
            super::config().map(|c| c.logging)
        }

        pub fn level() -> Result<String, ConfigError> {
            config().map(|c| c.level)
        }

        pub fn file() -> Result<Option<String>, ConfigError> {
            config().map(|c| c.file)
        }
    }

    pub mod radarr {
        use super::*;

        pub fn config() -> Result<RadarrConfig, ConfigError> {
            super::config().map(|c| c.radarr)
        }

        pub fn url() -> Result<String, ConfigError> {
            config().map(|c| c.url)
        }

        pub fn apikey() -> Result<String, ConfigError> {
            config().map(|c| c.apikey)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RadarrConfig {
    pub url: String,
    pub apikey: String,
}

// Global config instance
static CONFIG: OnceLock<Value> = OnceLock::new();

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    InvalidJson(serde_json::Error),
    ValidationError(String),
    SettingNotFound(String),
    TypeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileNotFound => write!(f, "Config file not found"),
            ConfigError::InvalidJson(e) => write!(f, "Invalid JSON: {}", e),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::SettingNotFound(key) => write!(f, "Setting not found: {}", key),
            ConfigError::TypeError(msg) => write!(f, "Type error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn init_config(file_path: &str) -> Result<(), ConfigError> {
    // Read and parse config file
    let config_str = fs::read_to_string(file_path).map_err(|_| ConfigError::FileNotFound)?;

    let config_value: Value =
        serde_json::from_str(&config_str).map_err(ConfigError::InvalidJson)?;

    // Validate by deserializing to our typed struct
    let _typed_config: AppConfig = from_value(config_value.clone())
        .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

    // Store the validated config
    CONFIG
        .set(config_value)
        .map_err(|_| ConfigError::ValidationError("Config already initialized".to_string()))?;

    Ok(())
}

// Get setting with dot notation path and typed output
pub fn get_setting<T>(path: &str) -> Result<T, ConfigError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let config = CONFIG
        .get()
        .ok_or_else(|| ConfigError::ValidationError("Config not initialized".to_string()))?;

    let value = get_nested_value(config, path)
        .ok_or_else(|| ConfigError::SettingNotFound(path.to_string()))?;

    from_value(value.clone()).map_err(|e| {
        ConfigError::TypeError(format!("Cannot convert {} to requested type: {}", path, e))
    })
}

// Get setting with typed enum input
pub trait ConfigPath {
    fn path(&self) -> &'static str;
}

#[derive(Debug)]
pub enum Setting {
    ServerHost,
    ServerPort,
    // Note: workers is not in the AppConfig struct, removing this variant
    DatabasePath,
    LoggingLevel,
    LoggingFile,
    RadarrUrl,
    RadarrApikey,
}

impl ConfigPath for Setting {
    fn path(&self) -> &'static str {
        match self {
            Setting::ServerHost => "server.host",
            Setting::ServerPort => "server.port",
            // Note: workers is not in the AppConfig struct, removing this variant
            Setting::DatabasePath => "database_path",
            Setting::LoggingLevel => "logging.level",
            Setting::LoggingFile => "logging.file",
            Setting::RadarrUrl => "radarr.url",
            Setting::RadarrApikey => "radarr.apikey",
        }
    }
}

pub fn get_setting_typed<T>(setting: Setting) -> Result<T, ConfigError>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    get_setting(setting.path())
}

// Helper function to navigate nested JSON
fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        current = current.get(part)?;
    }

    Some(current)
}

// Get entire config section as typed struct
pub fn get_config_section<T>(section: &str) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let config = CONFIG
        .get()
        .ok_or_else(|| ConfigError::ValidationError("Config not initialized".to_string()))?;

    let section_value = config
        .get(section)
        .ok_or_else(|| ConfigError::SettingNotFound(section.to_string()))?;

    from_value(section_value.clone()).map_err(|e| {
        ConfigError::TypeError(format!("Cannot deserialize section {}: {}", section, e))
    })
}
