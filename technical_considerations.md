# Technical Considerations

## Overview
This document captures technical decisions, lessons learned, and implementation notes for the Regelator project.

---

## Technology Stack Decisions

### Backend: Rust + Axum
**Decision:** Use Axum 0.8 for web server framework
**Rationale:** 
- Modern async framework with good performance
- Excellent integration with tokio ecosystem
- Type-safe routing and middleware

**Lessons Learned:**
- *[To be filled as we encounter issues/successes]*

### Frontend: HTMX + Pico.css
**Decision:** Server-side rendered HTML with HTMX for interactivity
**Rationale:**
- Simpler than full SPA for rule browsing use case
- Better SEO and initial load performance
- Reduces complexity compared to separate frontend framework

**Lessons Learned:**
- *[To be filled during implementation]*

### Database: SQLite + Diesel
**Decision:** SQLite for data storage with Diesel ORM
**Rationale:**
- Simple deployment model (single binary + database file)
- Diesel provides type-safe database interactions
- Sufficient for expected load and data complexity

**Lessons Learned:**
- *[To be filled during database implementation]*

---

## Architecture Patterns

### Request Handling
**Pattern:** Standard Axum handler functions
**Notes:**
- Keep handlers thin, delegate business logic to service layer
- Use proper error handling with custom error types

### Data Access
**Pattern:** Repository pattern with Diesel
**Notes:**
- Abstract database operations behind repository traits
- Makes testing easier with mock implementations

---

## Development Workflow

### Code Quality
**Standards Applied:**
- `cargo fmt` for consistent formatting
- `cargo clippy` for linting and best practices
- Unit tests for business logic functions
- Integration tests for API endpoints

### Testing Strategy
**Approach:**
- Unit tests for pure business logic
- Integration tests for database operations
- Manual testing for user-facing features
- Health check endpoint for deployment verification

---

## Performance Considerations

### Current Assumptions
- Small to medium rule dataset (< 10MB)
- Low to medium concurrent users (< 1000)
- Primary read operations with occasional updates

### Optimization Notes
- Static assets cached for 1 year with versioned filenames
- GZip compression reduces bandwidth usage (~70% compression for CSS/HTML)
- Template compilation happens once at startup

---

## Deployment Considerations

### Target Environment
- Single server deployment initially
- Linux-based hosting
- Reverse proxy (nginx) for static file serving in production

### Configuration
- Environment variables for database path and port
- Graceful shutdown handling for container deployments

---

## Known Issues & Workarounds

*[To be filled as issues are encountered]*

---

## Lessons Learned

### Story 1.1 - Basic Web Server
**What worked well:**
- Axum setup was straightforward
- Tokio async runtime works smoothly
- Health check endpoint useful for deployment verification

**What to improve:**
- *[To be filled after more implementation]*

**Technical debt:**
- None identified yet

### Story 1.2 - Basic HTML Templating
**What worked well:**
- Minijinja integrates cleanly with Axum
- Template inheritance works as expected
- Pico.css CDN provides clean styling without local assets
- Application state pattern scales well for shared resources

**What to improve:**
- Consider template hot-reloading for development

**Technical debt:**
- Templates loaded from filesystem at runtime (acceptable for now)

**Key decisions:**
- Used `loader` feature for minijinja to load templates from filesystem
- Implemented proper error handling with AppError wrapper
- Eyre errors logged and mapped to opaque 500 responses for security

### Story 1.3 - Static File Serving
**What worked well:**
- Tower-http ServeDir integrates cleanly with Axum
- Versioned filenames enable aggressive caching (1 year max-age)
- GZip compression works automatically for all responses
- Layer composition pattern scales well

**What to improve:**
- Manual versioning process could be automated in future

**Technical debt:**
- None identified

**Key decisions:**
- Used versioned filenames (`pico-v2.css`) for cache busting
- Applied long-term caching headers with `immutable` directive
- Added GZip compression layer for bandwidth optimization
- Used tower Layer trait for composing middleware

---

## Future Technical Decisions

### Questions to resolve:
1. Authentication strategy for different user types
2. Internationalization approach for multi-language support
3. Caching strategy for rule content
4. API versioning strategy

### Research needed:
- Ultimate Frisbee rules data structure and format
- Integration options for Telegram/Discord bots
- Localization libraries for Rust web applications

---

*Last updated: 2025-07-22*