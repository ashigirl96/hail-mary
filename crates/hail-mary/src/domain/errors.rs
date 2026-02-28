use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Invalid spec name: {0}. Spec name must be kebab-case")]
    InvalidSpecName(String),
}
