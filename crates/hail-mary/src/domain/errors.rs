use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Invalid feature name: {0}. Feature name must be kebab-case")]
    InvalidFeatureName(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_feature_name_error() {
        let error = DomainError::InvalidFeatureName("Invalid_Name".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid feature name: Invalid_Name. Feature name must be kebab-case"
        );
    }

    #[test]
    fn test_domain_error_debug() {
        let error = DomainError::InvalidFeatureName("Bad_Name".to_string());
        assert_eq!(format!("{:?}", error), "InvalidFeatureName(\"Bad_Name\")");
    }

    #[test]
    fn test_domain_error_equality() {
        let error1 = DomainError::InvalidFeatureName("Bad_Name".to_string());
        let error2 = DomainError::InvalidFeatureName("Bad_Name".to_string());
        let error3 = DomainError::InvalidFeatureName("Other_Bad".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_domain_error_is_error() {
        let error = DomainError::InvalidFeatureName("Bad_Name".to_string());
        let boxed_error: Box<dyn std::error::Error> = Box::new(error);
        assert!(boxed_error.to_string().contains("Invalid feature name"));
    }
}
