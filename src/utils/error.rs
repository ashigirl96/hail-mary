use thiserror::Error;

#[derive(Error, Debug)]
pub enum HailMaryError {
    #[error("Feature '{0}' already exists")]
    #[allow(dead_code)] // Used in CLI error handling and future features
    FeatureAlreadyExists(String),

    #[error("Invalid feature name: {0}. Must be kebab-case")]
    #[allow(dead_code)] // Used in CLI error handling and future features
    InvalidFeatureName(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HailMaryError>;
