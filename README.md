# 🚀 Hail-Mary

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**A Rust CLI for specification-driven development with Kiro project management.**

Hail-Mary provides project specification management through the Kiro system, designed for spec-driven development workflows with Claude Code integration.

## ✨ Features

### 🎯 Kiro Specification Management
- **Structured Specifications**: Project specification lifecycle with requirements and tasks
- **Template Generation**: Automatic creation of `tasks.md` and `memo.md` templates
- **Interactive Archiving**: TUI-based interface for marking completed specs
- **Date-based Organization**: Chronological specification organization with automatic naming

### 📄 File-based Steering System
- **Version-Controllable Context**: Git-trackable steering files for project knowledge
- **Core Steering Types**: Product overview, technology stack, and project structure documentation
- **Backup Protection**: Automatic backup creation before modifications
- **Team Collaboration**: Shared context through version control

### 🔗 Claude Code Integration
- **Seamless Launch**: TTY-aware Claude Code launching with proper job control
- **Specification Context**: `plansDirectory` setting for spec awareness
- **Interactive Selection**: TUI for choosing existing specs or creating new ones
- **Slash Commands**: `/hm:steering` and `/hm:steering-remember` for steering management

### 🖥️ Terminal User Interface (TUI)
- **Interactive Selection**: Specification selector with keyboard navigation
- **Visual Design**: Clean interface with real-time feedback
- **Built with Ratatui**: Modern terminal UI framework

### 🔧 Developer Experience
- **Shell Completions**: Auto-completion support for bash, zsh, fish, PowerShell, and elvish
- **Clean Architecture**: Well-structured Rust implementation
- **Comprehensive Testing**: Unit and integration tests
- **Configuration Management**: TOML-based configuration system

## 🚀 Quick Start

### Prerequisites

- **Rust**: Latest stable version ([Install Rust](https://rustup.rs/))
- **Just**: Command runner ([Install Just](https://just.systems/))
- **Claude Code**: CLI tool ([Install Claude Code](https://claude.ai/code))

### Installation

```bash
# Clone the repository
git clone https://github.com/ashigirl96/hail-mary.git
cd hail-mary

# Setup development environment
just setup

# Build the project
just build

# Initialize and work with specs
cargo run --package hail-mary -- code
```

### Project Structure

```
.
├── Cargo.toml                  # Workspace root configuration
└── crates/
    └── hail-mary/              # Main application crate
        ├── Cargo.toml          # Application dependencies
        └── src/                # Source code (Clean Architecture)
```

## 📋 Usage Examples

### Launch Claude Code with Spec Context

```bash
# Launch Claude Code with Kiro specification context
# Provides interactive TUI for spec selection
hail-mary code

# Skip dangerous permissions flag
hail-mary code --no-danger

# Continue previous conversation
hail-mary code --continue
```

The `code` command automatically initializes the project if needed (idempotent).

### Specification Management

```bash
# Create new specification (interactive TUI)
hail-mary code
# Navigate to "Create New" → Enter spec name

# Mark completed specifications as done (interactive TUI)
hail-mary complete
# - Use arrow keys or j/k to navigate
# - Press Space to select specifications
# - Press Enter to archive selected specs
```

### Steering Management

```bash
# Create a backup of steering files
hail-mary steering backup
```

### Shell Completions

```bash
# Generate bash completions
hail-mary shell-completions bash > /etc/bash_completion.d/hail-mary

# Generate zsh completions
hail-mary shell-completions zsh > ~/.zsh/completions/_hail-mary

# Generate fish completions
hail-mary shell-completions fish > ~/.config/fish/completions/hail-mary.fish
```

## ⚙️ Configuration

### Project Configuration (`.kiro/config.toml`)

```toml
[spec]
lang = "en"  # or "ja" for Japanese

[steering.backup]
max = 10  # Maximum number of backups to retain

[[steering.types]]
name = "product"
purpose = "Product overview and value proposition"
criteria = [
    "Product Overview: Brief description of what the product is",
    "Core Features: Bulleted list of main capabilities",
]
allowed_operations = ["refresh", "discover"]

[[steering.types]]
name = "tech"
purpose = "Technical stack and development environment"
criteria = [
    "Architecture: High-level system design",
    "Development Environment: Required tools and setup",
]
allowed_operations = ["refresh", "discover"]

[[steering.types]]
name = "structure"
purpose = "Code organization and project structure patterns"
criteria = [
    "Root Directory Organization: Top-level structure",
    "Code Organization Patterns: How code is structured",
]
allowed_operations = ["refresh", "discover"]
```

### Update Strategy Control

Each steering type includes `allowed_operations`:

- **`["refresh", "discover"]`** - Update existing info and add new discoveries
- **`["refresh"]`** - Only update existing information
- **`["discover"]`** - Only add new discoveries
- **`[]`** - Manual updates only via `/hm:steering-remember`

### File System Organization

```
.kiro/
├── config.toml              # Project configuration
├── specs/                   # Active specifications
│   └── 2024-03-15-feature/
│       ├── tasks.md         # Task tracking and timeline
│       └── memo.md          # Notes and memos
├── archive/                 # Completed specifications
└── steering/                # Steering files
    ├── product.md
    ├── tech.md
    ├── structure.md
    └── backup/              # Automatic backups
```

## 🔌 Claude Code Integration

### Slash Commands

Hail-mary provides slash commands in `.claude/commands/hm/`:

- **`/hm:steering`**: Update and maintain steering files with parallel investigation
- **`/hm:steering-remember [hint]`**: Capture learning and insights to steering files

### Specification Context

When launching Claude Code, hail-mary sets the `plansDirectory` setting to point to the selected spec directory, enabling Claude to be aware of the current specification context.

## 🏗️ Architecture

Hail-Mary follows clean architecture principles:

```
CLI Interface → Command Layer → Use Case Layer → Repository Layer → Infrastructure
```

### Key Components

- **CLI Interface**: Clap-based command routing with shell completions
- **Use Case Layer**: Business logic for project and specification management
- **Repository Pattern**: Abstracted file system operations
- **Infrastructure Layer**: File system operations, process management, TUI

## 🛠️ Development

### Available Commands

```bash
just              # View all available tasks
just build        # Build the project
just test         # Run all tests
just fmt          # Format code
just lint         # Run clippy linter
just dev          # Watch and rebuild on changes
just ci           # Run all CI checks locally
```

### Testing

```bash
just test         # Run comprehensive test suite
just test-verbose # Run tests with output
just test-watch   # Watch and run tests on changes
```

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Test** your changes (`just ci`)
5. **Push** to the branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Rust Community**: For excellent tooling and libraries
- **Ratatui Community**: For the terminal UI framework
- **Clap**: For the CLI framework

---

**Built with ❤️ in Rust**
