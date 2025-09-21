/// Spec-related validation and utilities
pub struct SpecValidator;

impl SpecValidator {
    /// Validate spec name format (kebab-case)
    pub fn validate_spec_name(name: &str) -> Result<(), crate::domain::errors::DomainError> {
        if name.is_empty() {
            return Err(crate::domain::errors::DomainError::InvalidFeatureName(
                "Name cannot be empty".to_string(),
            ));
        }

        // Validate kebab-case format
        let regex = regex::Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap();
        if !regex.is_match(name) {
            return Err(crate::domain::errors::DomainError::InvalidFeatureName(
                name.to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_spec_name_valid() {
        assert!(SpecValidator::validate_spec_name("valid-name").is_ok());
        assert!(SpecValidator::validate_spec_name("feature-123").is_ok());
        assert!(SpecValidator::validate_spec_name("simple").is_ok());
        assert!(SpecValidator::validate_spec_name("2025-09-05-feature").is_ok());
    }

    #[test]
    fn test_validate_spec_name_invalid() {
        use crate::domain::errors::DomainError;

        // Empty name
        assert!(matches!(
            SpecValidator::validate_spec_name(""),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Uppercase letters
        assert!(matches!(
            SpecValidator::validate_spec_name("Invalid-Name"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Underscore
        assert!(matches!(
            SpecValidator::validate_spec_name("invalid_name"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Starts with dash
        assert!(matches!(
            SpecValidator::validate_spec_name("-invalid"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Ends with dash
        assert!(matches!(
            SpecValidator::validate_spec_name("invalid-"),
            Err(DomainError::InvalidFeatureName(_))
        ));

        // Double dash
        assert!(matches!(
            SpecValidator::validate_spec_name("invalid--name"),
            Err(DomainError::InvalidFeatureName(_))
        ));
    }
}
