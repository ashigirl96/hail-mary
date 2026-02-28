# Hail-Mary

[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust CLI for specification-driven development with Claude Code.

Hail-Mary manages project specifications (`.specs/`) and launches Claude Code with an interactive TUI for spec selection.

## Features

- **Specification Management**: Create, select, and archive specs with date-based naming
- **Interactive TUI**: Ratatui-based spec selector with keyboard navigation
- **Claude Code Integration**: Seamless launch with TTY-aware process replacement
- **Shell Completions**: bash, zsh, fish, PowerShell, elvish
- **Auto-migration**: Automatically migrates from `.kiro/specs/` to `.specs/`

## Installation

```bash
cargo install --git https://github.com/ashigirl96/hail-mary.git
```

Or build from source:

```bash
git clone https://github.com/ashigirl96/hail-mary.git
cd hail-mary
cargo install --path crates/hail-mary
```

### Prerequisites

- **Rust**: Latest stable version ([Install Rust](https://rustup.rs/))
- **Claude Code**: CLI tool ([Install Claude Code](https://claude.ai/code))

## Usage

### Launch Claude Code with Spec Context

```bash
# Interactive TUI for spec selection, then launches Claude Code
hail-mary code

# Skip dangerous permissions flag
hail-mary code --no-danger

# Continue previous conversation
hail-mary code --continue
```

The `code` command automatically initializes `.specs/` if needed (idempotent).

### Archive Completed Specifications

```bash
# Interactive TUI for selecting specs to archive
hail-mary complete
```

### Shell Completions

```bash
hail-mary shell-completions bash > /etc/bash_completion.d/hail-mary
hail-mary shell-completions zsh > ~/.zsh/completions/_hail-mary
hail-mary shell-completions fish > ~/.config/fish/completions/hail-mary.fish
```

## Directory Structure

```
.specs/
  archive/                       # Completed specifications
  2026-02-28-feature-name/
    memo.md                      # Notes and context
  2026-02-27-another-feature/
    memo.md
```

## Architecture

Clean architecture with hexagonal layers:

```
CLI (clap) -> Application (use cases) -> Domain (validation) -> Infrastructure (fs, TUI, process)
```

## License

MIT License - see [LICENSE](LICENSE) for details.
