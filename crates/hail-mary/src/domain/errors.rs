use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Invalid spec name: {0}. Spec name must be kebab-case")]
    InvalidSpecName(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_spec_name_error() {
        let error = DomainError::InvalidSpecName("Invalid_Name".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid spec name: Invalid_Name. Spec name must be kebab-case"
        );
    }

    #[test]
    fn test_domain_error_debug() {
        let error = DomainError::InvalidSpecName("Bad_Name".to_string());
        assert_eq!(format!("{:?}", error), "InvalidSpecName(\"Bad_Name\")");
    }

    #[test]
    fn test_domain_error_equality() {
        let error1 = DomainError::InvalidSpecName("Bad_Name".to_string());
        let error2 = DomainError::InvalidSpecName("Bad_Name".to_string());
        let error3 = DomainError::InvalidSpecName("Other_Bad".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_domain_error_is_error() {
        let error = DomainError::InvalidSpecName("Bad_Name".to_string());
        let boxed_error: Box<dyn std::error::Error> = Box::new(error);
        assert!(boxed_error.to_string().contains("Invalid spec name"));
    }
}
