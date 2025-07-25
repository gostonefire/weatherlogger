use std::fmt;
use std::fmt::Formatter;
use log4rs::config::runtime::ConfigErrors;
use log::SetLoggerError;
use crate::manager_db::errors::DBError;

/// Error representing an unrecoverable error that will halt the application
///
#[derive(Debug)]
pub struct UnrecoverableError(pub String);
impl fmt::Display for UnrecoverableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UnrecoverableError: {}", self.0)
    }
}
impl From<rusqlite::Error> for UnrecoverableError {
    fn from(e: rusqlite::Error) -> Self { UnrecoverableError(e.to_string()) }
}
impl From<std::io::Error> for UnrecoverableError {
    fn from(e: std::io::Error) -> Self { UnrecoverableError(e.to_string()) }
}
impl From<ConfigError> for UnrecoverableError {
    fn from(e: ConfigError) -> Self {
        UnrecoverableError(e.to_string())
    }
}
impl From<DBError> for UnrecoverableError {
    fn from(e: DBError) -> Self { UnrecoverableError(e.to_string()) }
}

/// Errors while managing configuration
///
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ConfigError: {}", self.0)
    }
}
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self { ConfigError(e.to_string()) }
}
impl From<SetLoggerError> for ConfigError {
    fn from(e: SetLoggerError) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<ConfigErrors> for ConfigError {
    fn from(e: ConfigErrors) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<&str> for ConfigError {
    fn from(e: &str) -> Self { ConfigError(e.to_string()) }
}
impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError(e.to_string())
    }
}

pub struct TempError(pub String);
impl fmt::Display for TempError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { write!(f, "TempError: {}", self.0) }
}
impl From<reqwest::Error> for TempError {
    fn from(e: reqwest::Error) -> Self { TempError(e.to_string()) }
}
impl From<serde_json::Error> for TempError {
    fn from(e: serde_json::Error) -> Self { TempError(e.to_string()) }
}
