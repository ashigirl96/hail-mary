use super::error::{HailMaryError, Result};

/// Validates if a string is in kebab-case format
///
/// Kebab-case rules:
/// - Only lowercase letters, numbers, and hyphens
/// - Cannot start or end with a hyphen
/// - Cannot have consecutive hyphens
/// - Must have at least one character
#[allow(dead_code)] // Used in CLI validation and future features
pub fn validate_kebab_case(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(HailMaryError::InvalidFeatureName(
            "Feature name cannot be empty".to_string(),
        ));
    }

    // Check if starts or ends with hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Err(HailMaryError::InvalidFeatureName(format!(
            "{} - cannot start or end with hyphen",
            name
        )));
    }

    // Check for consecutive hyphens
    if name.contains("--") {
        return Err(HailMaryError::InvalidFeatureName(format!(
            "{} - cannot contain consecutive hyphens",
            name
        )));
    }

    // Check if all characters are valid (lowercase letters, numbers, and single hyphens)
    for char in name.chars() {
        if !char.is_ascii_lowercase() && !char.is_ascii_digit() && char != '-' {
            return Err(HailMaryError::InvalidFeatureName(format!(
                "{} - only lowercase letters, numbers, and hyphens allowed",
                name
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_kebab_case() {
        assert!(validate_kebab_case("feature").is_ok());
        assert!(validate_kebab_case("my-feature").is_ok());
        assert!(validate_kebab_case("feature-123").is_ok());
        assert!(validate_kebab_case("complex-feature-name").is_ok());
        assert!(validate_kebab_case("a").is_ok());
        assert!(validate_kebab_case("123").is_ok());
    }

    #[test]
    fn test_invalid_kebab_case() {
        // Empty string
        assert!(validate_kebab_case("").is_err());

        // Starts with hyphen
        assert!(validate_kebab_case("-feature").is_err());

        // Ends with hyphen
        assert!(validate_kebab_case("feature-").is_err());

        // Consecutive hyphens
        assert!(validate_kebab_case("feature--name").is_err());

        // Uppercase letters
        assert!(validate_kebab_case("Feature").is_err());
        assert!(validate_kebab_case("FEATURE").is_err());

        // Invalid characters
        assert!(validate_kebab_case("feature_name").is_err());
        assert!(validate_kebab_case("feature.name").is_err());
        assert!(validate_kebab_case("feature name").is_err());
        assert!(validate_kebab_case("feature@name").is_err());
    }
}
