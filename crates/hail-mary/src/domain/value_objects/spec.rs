pub struct SpecValidator;

impl SpecValidator {
    pub fn validate_spec_name(name: &str) -> Result<(), crate::domain::errors::DomainError> {
        if name.is_empty() {
            return Err(crate::domain::errors::DomainError::InvalidSpecName(
                "Name cannot be empty".to_string(),
            ));
        }

        // Validate kebab-case: lowercase alphanumeric segments separated by single hyphens
        let mut prev_was_hyphen = false;
        for (i, c) in name.chars().enumerate() {
            if c == '-' {
                if i == 0 || i == name.len() - 1 || prev_was_hyphen {
                    return Err(crate::domain::errors::DomainError::InvalidSpecName(
                        name.to_string(),
                    ));
                }
                prev_was_hyphen = true;
            } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
                prev_was_hyphen = false;
            } else {
                return Err(crate::domain::errors::DomainError::InvalidSpecName(
                    name.to_string(),
                ));
            }
        }

        Ok(())
    }
}
