pub fn format_success(message: &str) -> String {
    format!("✅ {}", message)
}

pub fn format_error(message: &str) -> String {
    format!("❌ Error: {}", message)
}
