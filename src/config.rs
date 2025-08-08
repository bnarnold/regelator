use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub import: ImportConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    pub session_duration_hours: u32,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImportConfig {
    pub rule_set_name: String,
    pub rule_set_slug: String,
    pub version_name: String,
    pub version_effective_date: String,
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
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Override with environment variables prefixed with REGELATOR__
            .add_source(Environment::with_prefix("REGELATOR").separator("__"))
            .build()?;

        let mut loaded_config: Config = config.try_deserialize()?;

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
            import: ImportConfig {
                rule_set_name: "Test Rules".to_string(),
                rule_set_slug: "test-rules".to_string(),
                version_name: "Test Version".to_string(),
                version_effective_date: "2025-01-01".to_string(),
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
            import: ImportConfig {
                rule_set_name: "Test Rules".to_string(),
                rule_set_slug: "test-rules".to_string(),
                version_name: "Test Version".to_string(),
                version_effective_date: "2025-01-01".to_string(),
            },
        };

        assert_eq!(config.session_duration(), chrono::Duration::hours(4));
    }
}
