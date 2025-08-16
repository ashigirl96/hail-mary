use thiserror::Error;

#[derive(Error, Debug)]
pub enum HailMaryError {
    #[error("Feature '{0}' already exists")]
    FeatureAlreadyExists(String),

    #[error("Invalid feature name: {0}. Must be kebab-case")]
    InvalidFeatureName(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HailMaryError>;
