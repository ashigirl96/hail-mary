use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(value: f32) -> Result<Self, DomainError> {
        if value < 0.0 || value > 1.0 {
            return Err(DomainError::InvalidConfidence(value));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f32 {
        self.0
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_valid_range() {
        // Red: 失敗するテストを書く
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        
        assert!(Confidence::new(1.5).is_err());
        assert!(Confidence::new(-0.1).is_err());
    }

    #[test]
    fn test_confidence_invalid_values() {
        let result = Confidence::new(1.5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DomainError::InvalidConfidence(1.5));

        let result = Confidence::new(-0.5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DomainError::InvalidConfidence(-0.5));
    }

    #[test]
    fn test_confidence_value_getter() {
        let confidence = Confidence::new(0.75).unwrap();
        assert_eq!(confidence.value(), 0.75);
    }

    #[test]
    fn test_confidence_default() {
        let confidence = Confidence::default();
        assert_eq!(confidence.value(), 1.0);
    }

    #[test]
    fn test_confidence_clone() {
        let confidence1 = Confidence::new(0.8).unwrap();
        let confidence2 = confidence1.clone();
        assert_eq!(confidence1, confidence2);
        assert_eq!(confidence1.value(), confidence2.value());
    }

    #[test]
    fn test_confidence_debug() {
        let confidence = Confidence::new(0.95).unwrap();
        let debug_str = format!("{:?}", confidence);
        assert_eq!(debug_str, "Confidence(0.95)");
    }

    #[test]
    fn test_confidence_edge_cases() {
        // Boundary values
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        
        // Just outside boundaries
        assert!(Confidence::new(-0.000001).is_err());
        assert!(Confidence::new(1.000001).is_err());
        
        // Special float values
        assert!(Confidence::new(f32::NAN).is_err());
        assert!(Confidence::new(f32::INFINITY).is_err());
        assert!(Confidence::new(f32::NEG_INFINITY).is_err());
    }
}