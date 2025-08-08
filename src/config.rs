use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{de, Deserialize, Deserializer};
use std::{env, str::FromStr};
use tracing::Level;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub session_duration_hours: u32,
    pub jwt_secret: String,
}

fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    struct LevelVisitor;

    impl<'de> de::Visitor<'de> for LevelVisitor {
        type Value = Level;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string representing a log level")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Level::from_str(value).map_err(de::Error::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_any(LevelVisitor)
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(deserialize_with = "deserialize_level")]
    pub level: Level,
    pub format: String,
    pub enable_colors: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImportConfig {
    pub rule_set_name: String,
    pub rule_set_slug: String,
    pub version_name: String,
    pub version_effective_date: String,
}

impl ImportConfig {
    /// Load configuration from TOML files and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if it exists (ignore errors if it doesn't exist)
        let _ = dotenvy::dotenv();

        let environment = env::var("REGELATOR_ENV").unwrap_or_else(|_| "local".to_string());

        let config = ConfigBuilder::builder()
            // Load shared configuration
            .add_source(File::with_name("config/shared").required(false))
            // Load environment-specific configuration
            .add_source(File::with_name(&format!("config/{environment}")).required(false))
            // Override with environment variables prefixed with REGELATOR__
            .add_source(Environment::with_prefix("REGELATOR").separator("__"))
            .build()?;

        let loaded_config: ImportConfig = config.try_deserialize()?;

        Ok(loaded_config)
    }
}
impl Config {
    /// Load configuration from TOML files and environment variables
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if it exists (ignore errors if it doesn't exist)
        let _ = dotenvy::dotenv();

        let environment = env::var("REGELATOR_ENV").unwrap_or_else(|_| "local".to_string());

        let config = ConfigBuilder::builder()
            // Load shared configuration
            .add_source(File::with_name("config/shared").required(false))
            // Load environment-specific configuration
            .add_source(File::with_name(&format!("config/{environment}")).required(false))
            // Override with environment variables prefixed with REGELATOR__
            .add_source(Environment::with_prefix("REGELATOR").separator("__"))
            .build()?;

        let loaded_config: Config = config.try_deserialize()?;

        // Validate configuration
        loaded_config.validate()?;

        Ok(loaded_config)
    }

    /// Validate the configuration
    fn validate(&self) -> Result<(), ConfigError> {
        if self.security.jwt_secret.is_empty() {
            return Err(ConfigError::Message(
                "JWT secret cannot be empty".to_string(),
            ));
        }

        if self.security.jwt_secret.len() < 32 {
            return Err(ConfigError::Message(
                "JWT secret must be at least 32 characters long".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Get session duration as chrono::Duration
    pub fn session_duration(&self) -> chrono::Duration {
        chrono::Duration::hours(self.security.session_duration_hours as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_address() {
        let config = Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "test.db".to_string(),
            },
            security: SecurityConfig {
                session_duration_hours: 2,
                jwt_secret: "test-secret-that-is-long-enough-for-validation".to_string(),
            },
            logging: LoggingConfig {
                level: Level::INFO,
                format: "tree".to_string(),
                enable_colors: true,
            },
        };

        assert_eq!(config.bind_address(), "0.0.0.0:3000");
    }

    #[test]
    fn test_session_duration() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8000,
            },
            database: DatabaseConfig {
                url: "test.db".to_string(),
            },
            security: SecurityConfig {
                session_duration_hours: 4,
                jwt_secret: "test-secret-that-is-long-enough-for-validation".to_string(),
            },
            logging: LoggingConfig {
                level: Level::INFO,
                format: "tree".to_string(),
                enable_colors: true,
            },
        };

        assert_eq!(config.session_duration(), chrono::Duration::hours(4));
    }
}
