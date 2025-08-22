use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DomainError {
    #[error("Invalid confidence value: {0}. Confidence must be between 0.0 and 1.0")]
    InvalidConfidence(f32),
    
    #[error("Invalid memory type: {0}")]
    InvalidMemoryType(String),
    
    #[error("Invalid feature name: {0}. Feature name must be kebab-case")]
    InvalidFeatureName(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_confidence_error() {
        let error = DomainError::InvalidConfidence(1.5);
        assert_eq!(
            error.to_string(),
            "Invalid confidence value: 1.5. Confidence must be between 0.0 and 1.0"
        );
    }

    #[test]
    fn test_invalid_memory_type_error() {
        let error = DomainError::InvalidMemoryType("invalid".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid memory type: invalid"
        );
    }

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
        let error = DomainError::InvalidConfidence(2.0);
        assert_eq!(format!("{:?}", error), "InvalidConfidence(2.0)");
    }

    #[test]
    fn test_domain_error_equality() {
        let error1 = DomainError::InvalidConfidence(1.5);
        let error2 = DomainError::InvalidConfidence(1.5);
        let error3 = DomainError::InvalidConfidence(2.0);
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_domain_error_is_error() {
        let error = DomainError::InvalidConfidence(1.5);
        let boxed_error: Box<dyn std::error::Error> = Box::new(error);
        assert!(boxed_error.to_string().contains("Invalid confidence value"));
    }
}