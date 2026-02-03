pub mod spec;
pub mod steering;

// Re-export main types for convenience
pub use spec::SpecValidator;
pub use steering::{Criterion, Steering, SteeringConfig, SteeringType, Steerings};
