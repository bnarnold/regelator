# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Start for New Sessions

**Before starting any work, read these files in order:**

1. **`pair_programming.md`** - Our workflow process for story-driven development
2. **`project_plan_{some_extension}.md`** - Current progress and next story to work on  
3. **`technical_considerations.md`** - Lessons learned and implementation decisions
4. **`architecture.md`** - Overall architecture and design decisions

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

- Run the application with `cargo run`
- Test the application with `cargo test`
- Install crates with `cargo add` (the lock file is `Cargo.lock`)
- Build and check for compilation errors with `cargo build`
- Format code with `cargo fmt`
- Lint code with `cargo clippy`

## Technology stack

- **Backend**: Rust + `axum` v0.8 + `minijinja`
- **Frontend**: `htmx` for interactity, `pico.css` for styling
- **Data storage**: SQLite with `diesel`
