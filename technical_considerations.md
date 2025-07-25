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

### Story 1.4 - Database Setup
**What worked well:**
- Diesel setup was straightforward with CLI
- R2D2 connection pooling integrates well with Axum state
- Synchronous diesel works fine without spawn_blocking for simple queries
- Health check provides good database connectivity verification

**What to improve:**
- Need configuration system for database URL (Story 1.5)

**Technical debt:**
- Hardcoded database path needs configuration

**Key decisions:**
- Used synchronous diesel instead of diesel-async (SQLite not supported)
- R2D2 connection pooling for thread-safe database access
- Database stored in `/db` folder, gitignored
- Health endpoint performs `SELECT 1=1` query to verify connectivity

### Story 1.2 - Inline Rule Cross-References
**What worked well:**
- Enhanced regex pattern to distinguish "Section X" vs bare number references
- Import-time processing provides optimal runtime performance
- Broken reference tracking helps validate rule content integrity
- Template-based approach scales well across different content types

**What to improve:**
- Consider adding reference validation reports in import output

**Technical debt:**
- None identified

**Key decisions:**
- Used semantic distinction: `{{section:slug}}` vs `{{rule:slug}}`
- Section references display as "Section X", rule references as numbers only
- Processing happens once during import, not on every request
- Graceful fallback preserves original text for missing references

### Story 1.3 - Anchor-Based Cross-Reference Navigation
**What worked well:**
- Context-aware link generation works cleanly with function parameter
- Anchor tags integrate naturally with existing template structure
- Template-level anchor IDs provide clean, semantic HTML
- Browser back/forward navigation works seamlessly with anchors

**What to improve:**
- Could add smooth scrolling CSS for better anchor navigation experience

**Technical debt:**
- None identified

**Key decisions:**
- Added `use_anchors` parameter to distinguish list vs detail view contexts
- List view generates `#anchor` links, detail view generates full URLs
- Rule numbers preserve full page navigation to detail views
- Anchor IDs use rule slugs for consistency and readability

### Story 2 - Title Field Cleanup & Display Simplification
**What worked well:**
- Database migration executed cleanly with automatic schema.rs updates
- Diesel's generated schema kept models in sync automatically
- Templates were already perfect - no display changes needed
- All tests continued passing after field removal

**What to improve:**
- Could have caught this unused field earlier with better code reviews

**Technical debt:**
- Eliminated: Unused database column and model fields

**Key decisions:**
- Removed `title` field entirely rather than keeping it optional
- Updated documentation to match actual implementation
- Focused on simplification over feature addition
- Discovered that sometimes the best enhancement is removing complexity

### Story 2.2: Definition Cross-Reference Integration
**What worked well:**
- Extended existing `{{rule:slug}}` pattern to support `{{definition:slug}}`
- Clean separation: import script processes content, handler resolves links
- Comprehensive test coverage for edge cases (word boundaries, case insensitivity, overlapping terms)
- Direct content processing avoids database update complexity
- Single-pass regex solution elegantly handles overlapping term conflicts

**What to improve:**
- Proper logging framework instead of println! statements
- Configurable term matching (currently case-insensitive with word boundaries)

**Technical debt:**
- **Logging**: Using println! for import progress instead of proper log levels (info, debug, etc.)
- **Future improvement**: Add structured logging crate (e.g., `log` + `env_logger`) for better debugging

**Key decisions:**
- Process definition content before import rather than post-processing database
- **Overlapping Terms Solution**: Single combined regex with longest-first term ordering
- Non-overlapping maximal matches via regex `find_iter()` prevents conflicts like `{{definition:offensive-{{definition:player}}}}`
- Always link to definitions page with anchors (no context-dependent behavior)
- Import-time processing for optimal runtime performance

**Major Technical Breakthrough:**
- **Problem**: Sequential regex replacements created overlapping substitutions (e.g., "offensive player" → "offensive {{definition:player}}" → "{{definition:offensive-{{definition:player}}}}")
- **Solution**: Build single regex with all terms as alternatives, sorted by length, using `find_iter()` for non-overlapping maximal matches
- **Result**: Clean handling of complex term relationships like "offensive player" vs "player"

**Architecture Lessons:**
- Template-based cross-reference system scales well across different content types
- Import-time content processing more efficient than runtime processing
- Regex alternation with maximal matching handles complex linguistic overlaps better than sequential processing

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

*Last updated: 2025-07-24*