# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Start for New Sessions

**Before starting any work, read these files in order:**

1. **`pair_programming.md`** - Our workflow process for story-driven development
2. **`project_plan.md`** - Master roadmap showing current epic and status
3. **`project_plan_{epic}.md`** - Detailed plan for the current active epic
4. **`technical_considerations.md`** - Lessons learned and implementation decisions
5. **`architecture.md`** - Overall architecture and design decisions
6. **`data_model_design.md`** - Database schema design principles and decisions
7. **`docs/schema.md`** - Current database schema (Mermaid diagram)

**Key workflow reminders:**

- Always use the TodoWrite tool to track story progress
- Follow the exact human verification format from pair_programming.md
- Update `technical_considerations.md` with lessons learned after each story
- Always run `jj commit` with a descriptive message after finishing a story

## Project Planning Structure

The project is organized into **epics** (major phases) containing **stories** (specific features):

### Master Plan: `project_plan.md`
- Lists all epics with who/what/why format using our personas (Nathan, Valeria, Fred)
- Shows epic status: 🏗️ Foundation, ✅ Completed, 📋 Planned
- Points to detailed epic plans without duplicating content

### Epic Plans: `project_plan_{epic}.md`
- Detailed planning for each epic (e.g., `project_plan_core.md`, `project_plan_interactive_frontend.md`)
- Contains stories, acceptance criteria, technical implementation details
- Tracks progress within that epic

### Workflow
1. Check `project_plan.md` to see current epic status
2. Read the detailed plan for the active epic
3. Work on the next story in that epic
4. Update progress in the detailed epic plan
5. Update epic status in master plan when complete

## Overview

Regelator is a tool for interacting with rules for the sport Ultimate Frisbee.
This includes:

- Interactive display of rules in a web application
- Telegram and Discord bots with regularly scheduled rules quizzes
- Chatbots for clarifying questions about the rules, with web and bot (Telegram/Discord) interfaces

The underlying rules document, and all applications, should be localized, with at least English and German supported for every feature.

## Development Commands

- Run the web application with `cargo run`
- Import rules data with `cargo run --bin import_rules < rules_file.txt`
- Test the application with `cargo test`
- Install crates with `cargo add` (the lock file is `Cargo.lock`)
- Build and check for compilation errors with `cargo build`
- Format code with `cargo fmt`
- Lint code with `cargo clippy`

## Configuration System

Regelator uses a TOML-based configuration system with environment variable overrides:

### Environment Selection
- Set `REGELATOR_ENV` to choose config: `local` (default), `dev`, or `prod`
- Configuration files: `config/shared.toml` and `config/{environment}.toml`

### Required Environment Variables
- `REGELATOR__SECURITY__JWT_SECRET`: JWT signing key (minimum 32 characters)

### Configuration Files
- `config/shared.toml`: Common settings across all environments
- `config/local.toml`: Local development overrides

### Environment Variable Overrides
All TOML settings can be overridden with `REGELATOR__` prefixed environment variables:
- `REGELATOR__SERVER__HOST=0.0.0.0`
- `REGELATOR__SERVER__PORT=3000`
- `REGELATOR__DATABASE__URL=db/custom.db`

### Setup
1. Copy `.env.example` to `.env`
2. Set `REGELATOR__SECURITY__JWT_SECRET` to a secure random string (32+ characters)
3. Customize other settings as needed

## Data Import

The import script (`cargo run --bin import_rules`) reads rule data from stdin and expects:

- Each line contains: rule number (ending with dot), slug, and content
- Rule numbers in format like `1.`, `1.2.`, `1.2.3.`
- Slug used for URL generation (e.g., `calling-hand-signals`)
- Content is everything after the slug on the same line
- Automatic hierarchy detection from rule numbers (1.2.1 is child of 1.2)
- Numeric sorting (so 1.2.10 comes after 1.2.9, not 1.2.2)

Example format:
```
1. spirit-of-the-game Ultimate stresses fair play and sportsmanship.
1.1. self-refereeing Players are responsible for their own foul calls.
15.13. calling-hand-signals Players are encouraged to use the WFDF Hand Signals to communicate all calls.
```

## Version Control

- **VCS**: Jujutsu (`jj`) with colocated Git backend
- **Workflow**: Trunk-based development
- **Branching**: Work directly on main branch, use `jj` for change management

### Key Commands
- Check current state with `jj status` or `jj st`
- View change history with `jj log` or `jj log -r 'all()'`
- See current changes with `jj diff`
- Create new change with `jj new` (optional, changes are auto-tracked)
- Describe changes with `jj describe -m "message"`

## Technology stack

- **Backend**: Rust + `axum` v0.8 + `minijinja`
- **Frontend**: `htmx` for interactity, `pico.css` for styling
- **Data storage**: SQLite with `diesel`