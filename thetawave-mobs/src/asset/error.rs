//! Shared error types for TOML asset loaders.

use std::fmt;

/// Errors that can occur while loading TOML-based asset files (.mob, .mobpatch).
#[derive(Debug)]
pub enum TomlAssetLoaderError {
    /// IO error while reading the file
    Io(std::io::Error),
    /// Error parsing the TOML content
    Toml(toml::de::Error),
}

impl fmt::Display for TomlAssetLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TomlAssetLoaderError::Io(err) => write!(f, "IO error loading asset: {}", err),
            TomlAssetLoaderError::Toml(err) => write!(f, "TOML parse error: {}", err),
        }
    }
}

impl std::error::Error for TomlAssetLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TomlAssetLoaderError::Io(err) => Some(err),
            TomlAssetLoaderError::Toml(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for TomlAssetLoaderError {
    fn from(err: std::io::Error) -> Self {
        TomlAssetLoaderError::Io(err)
    }
}

impl From<toml::de::Error> for TomlAssetLoaderError {
    fn from(err: toml::de::Error) -> Self {
        TomlAssetLoaderError::Toml(err)
    }
}
