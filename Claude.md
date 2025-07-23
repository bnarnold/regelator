# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Start for New Sessions

**Before starting any work, read these files in order:**

1. **`pair_programming.md`** - Our workflow process for story-driven development
2. **`project_plan_{some_extension}.md`** - Current progress and next story to work on  
3. **`technical_considerations.md`** - Lessons learned and implementation decisions
4. **`architecture.md`** - Overall architecture and design decisions
5. **`data_model_design.md`** - Database schema design principles and decisions
6. **`docs/schema.md`** - Current database schema (Mermaid diagram)

**Key workflow reminders:**

- Always use the TodoWrite tool to track story progress
- Follow the exact human verification format from pair_programming.md
- Update `technical_considerations.md` with lessons learned after each story

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

## Technology stack

- **Backend**: Rust + `axum` v0.8 + `minijinja`
- **Frontend**: `htmx` for interactity, `pico.css` for styling
- **Data storage**: SQLite with `diesel`
