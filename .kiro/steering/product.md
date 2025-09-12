# Product Overview

**Hail-Mary** - A sophisticated Rust CLI application for Kiro project specification management and file-based context steering.

## Product Overview

Hail-Mary provides intelligent project specification management through the Kiro system, designed for spec-driven development workflows. It bridges the gap between AI model interactions and persistent project knowledge through a file-based steering system that offers version-controllable context management and comprehensive specification lifecycle support.

## Core Features

- **Kiro Specification Management**: Complete project specification lifecycle with templates and interactive TUI
- **Steering System**: File-based context management for version-controllable project knowledge  
- **Claude Code Integration**: Seamless integration with Claude Code through system prompts and specifications
- **Interactive TUI**: Terminal user interface for specification selection and management
- **Clean Architecture**: Well-structured Rust implementation with clear separation of concerns
- **Shell Completions**: Auto-completion support for all major shells
- **Template System**: Structured specification templates with requirements, design, tasks, and investigation phases
- **Anthropic OAuth Client**: Separate crate with OAuth2 token management and Claude API integration
- **Custom Slash Commands**: `/hm:steering` and `/hm:steering-remember` for Claude Code steering management
- **Just Task Runner**: Comprehensive development workflow automation
- **Workspace Architecture**: Multi-crate Cargo workspace with clean separation

## Target Use Case

### Primary Use Cases
1. **Project Specification Management**: Complete spec-driven development workflow with documentation templates
2. **Context Management**: File-based steering system for persistent project knowledge
3. **Claude Code Enhancement**: Rich context provision for AI-assisted development sessions
4. **Team Collaboration**: Version-controllable project knowledge sharing

### Specific Scenarios
- Development teams using spec-driven development methodologies
- AI-assisted development with persistent project context through steering files
- Structured documentation workflows with requirements, design, and task management
- Teams requiring version-controllable project knowledge and decision tracking
- Projects needing seamless Claude Code integration with comprehensive context

## Key Value Proposition

### Unique Benefits
- **File-Based Knowledge Management**: Version-controllable steering system for transparent project context
- **AI-First Design**: Built specifically for Claude Code integration with structured context provision
- **Specification-Driven Development**: Complete workflow from specs to implementation with interactive tools
- **Developer Experience**: Clean CLI with interactive TUI components and shell completions
- **Architectural Quality**: Well-structured Rust implementation with clean architecture patterns
- **Team Collaboration**: Shared context through version control without database synchronization

### Differentiators
- File-based approach enabling git-tracked project knowledge evolution
- Native Claude Code integration with structured system prompts
- Interactive specification management with comprehensive template system
- Professional CLI application with proper TTY management and job control
- Complete specification lifecycle from creation to archival

