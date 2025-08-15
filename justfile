# Hail Mary - Rust Project Justfile
# Just command runner configuration for development tasks

# Show available recipes
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run the project
run *ARGS:
    cargo run {{ARGS}}

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Format code
fmt:
    cargo fmt

# Check code formatting
fmt-check:
    cargo fmt --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run clippy with additional checks
lint-strict:
    cargo clippy --all-targets --all-features -- -D warnings

# Type check without building
check:
    cargo check

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
doc:
    cargo doc --open

# Generate documentation without opening
doc-build:
    cargo doc

# Install project dependencies
deps:
    @echo "Installing development tools..."
    cargo install cargo-watch cargo-edit cargo-audit

# Watch for changes and rebuild
dev:
    cargo watch -x check -x test -x run

# Watch and run tests
test-watch:
    cargo watch -x test

# Security audit
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Install the binary
install:
    cargo install --path .

# Run all checks (format, lint, test)
ci: fmt-check lint test

# Setup development environment
setup: deps
    @echo "Development environment setup complete!"

# Show project info
info:
    @echo "Project: hail-mary"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo "Just version: $(just --version)"