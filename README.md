# Regelator

A comprehensive web application for learning and referencing Ultimate Frisbee rules, featuring interactive quizzes, multilingual support, and advanced analytics.

*Note: This codebase is primarily AI-generated through collaboration with Claude Code.*

## Overview

Regelator provides multiple ways to interact with Ultimate Frisbee rules:

- **Interactive Web Application**: Browse rules with hierarchical navigation, search, and responsive design
- **Quiz System**: Practice rule knowledge with randomized multiple-choice questions and detailed explanations
- **Analytics Dashboard**: Track learning progress and rule usage patterns
- **Export Capabilities**: Export analytics data in CSV and Parquet formats
- **Multilingual Support**: Localized content for English, German, and other languages

## Features

### Rule Management
- Hierarchical rule display with automatic numbering
- Version management for different rulebooks
- Markdown rendering for rich content formatting
- Cross-referencing between related rules
- Search functionality across all rules

### Learning & Assessment
- Interactive quiz system with immediate feedback
- Educational explanations for all questions
- Session tracking and progress analytics
- Randomized question selection
- Answer distribution analytics

### User Experience
- Server-side rendered with HTMX for smooth interactivity
- Responsive design for mobile and desktop
- Dark/light theme support with browser preference detection
- Semantic HTML with accessibility focus
- Progressive enhancement architecture

### Analytics & Export
- Comprehensive usage analytics
- GDPR-compliant data collection
- CSV export with customizable date ranges
- Parquet export for advanced analytics
- Chart visualizations for data insights

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: Server-side rendering with MiniJinja templates and HTMX
- **Database**: SQLite with Diesel ORM
- **Styling**: Pico CSS framework
- **Configuration**: TOML-based with environment variable overrides
- **Analytics**: Charming charts with export capabilities
- **Security**: JWT authentication with Argon2 password hashing

## Quick Start

### Prerequisites
- Rust (latest stable version)
- SQLite

### Setup

1. Clone the repository
2. Copy environment configuration:
   ```bash
   cp .env.example .env
   ```
3. Set required environment variables in `.env`:
   ```
   REGELATOR__SECURITY__JWT_SECRET=your-secure-32-character-minimum-secret-key
   ```
4. Run the application:
   ```bash
   cargo run
   ```

The web application will be available at `http://localhost:8000`.

### Importing Rules Data

Import rule data from a text file:
```bash
cargo run --bin import_rules < rules_file.txt
```

**Input format**: Each line should contain rule number (ending with dot), slug, and content:
```
1. spirit-of-the-game Ultimate stresses fair play and sportsmanship.
1.1. self-refereeing Players are responsible for their own foul calls.
15.13. calling-hand-signals Players are encouraged to use the WFDF Hand Signals.
```

## Development

### Commands
- **Run application**: `cargo run`
- **Run tests**: `cargo test`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`
- **Build**: `cargo build`

### Code Quality
Before committing changes, always run:
1. `cargo fmt`
2. `cargo clippy --fix`
3. `cargo check`
4. `cargo clippy`

### Configuration

Regelator uses a flexible configuration system:

- **Environment selection**: Set `REGELATOR_ENV` (default: `local`)
- **Configuration files**: `config/shared.toml` and `config/{environment}.toml`
- **Environment overrides**: Use `REGELATOR__` prefixed variables
  - Example: `REGELATOR__SERVER__PORT=8000`

## User Personas

### Nathan Newman (New Player)
Recent Ultimate player wanting engaging rule learning through gamification and interactive features.

### Valeria Veterana (Experienced Player)
Long-time player needing quick rule references and staying updated with rule changes.

### Fred Federator (Federation Coordinator)
Rules committee member coordinating translations and collecting usage statistics.

## Project Status

**Active Development**: The project is organized into epics covering setup, core functionality, interactive frontend, quiz system, production readiness, and AI features.

See `project_plan.md` for complete roadmap and current status.

## Contributing

This project follows trunk-based development with Jujutsu version control:
- Work directly on main branch
- Use `jj status` and `jj diff` to track changes
- Commit with descriptive messages using `jj describe`

## Architecture

The application follows a backend-first architecture with server-side rendering:
- Semantic HTML with progressive enhancement
- CSS-first presentation layer
- HTMX for interactive features
- Modular component design

For detailed technical information, see `architecture.md` and other documentation files.