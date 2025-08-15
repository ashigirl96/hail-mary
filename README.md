# Hail Mary

A Rust project with modern development setup using Just task runner.

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Just](https://just.systems/) command runner

### Installation

```bash
# Clone the repository
git clone https://github.com/ashigirl96/hail-mary.git
cd hail-mary

# Install development dependencies
just setup

# Build the project
just build

# Run the project
just run
```

## 🛠️ Development

### Available Commands

View all available commands:
```bash
just
```

Common development tasks:
```bash
# Build the project
just build

# Run tests
just test

# Format code
just fmt

# Run linter
just lint

# Clean build artifacts
just clean

# Watch for changes and rebuild
just dev

# Run all CI checks locally
just ci
```

### Development Environment

The project includes configuration for:
- **VSCode**: Automatic rust-analyzer setup, formatting, and linting
- **GitHub Actions**: Automated testing, linting, and building
- **Rust toolchain**: Pinned to stable with required components

### Project Structure

```
hail-mary/
├── src/           # Source code
├── docs/          # Documentation
├── reference/     # Technical references
├── .vscode/       # VSCode settings
├── .github/       # GitHub workflows
├── justfile       # Task definitions
└── rust-toolchain.toml  # Rust version pinning
```

## 📋 Tasks

The project uses [Just](https://just.systems/) for task management. All available tasks are defined in the `justfile`.

### Core Tasks
- `build` - Build the project
- `run` - Run the project
- `test` - Run tests
- `fmt` - Format code
- `lint` - Run clippy linter
- `clean` - Clean build artifacts

### Development Tasks
- `dev` - Watch for changes and rebuild
- `test-watch` - Watch and run tests
- `doc` - Generate and open documentation
- `audit` - Security audit
- `update` - Update dependencies

### CI Tasks
- `ci` - Run all CI checks (format, lint, test)
- `setup` - Setup development environment

## 🧪 Testing

```bash
# Run all tests
just test

# Run tests with output
just test-verbose

# Watch and run tests on changes
just test-watch
```

## 📝 Code Style

The project uses standard Rust formatting and linting:
- `cargo fmt` for code formatting
- `cargo clippy` for linting
- Automatic formatting on save in VSCode

## 🔒 Security

Security auditing is integrated into the development workflow:
```bash
# Run security audit
just audit
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `just ci` to ensure all checks pass
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Task management with [Just](https://just.systems/)
- Development setup inspired by Rust community best practices