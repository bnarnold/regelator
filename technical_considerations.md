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
- Axum's State extraction works seamlessly with new modular structure
- Re-exports in mod.rs maintain backward compatibility during refactoring

### Claude Code Slash Commands (2025-08-09)
**Decision:** Extract commit workflow into custom slash command `/commit`
**Rationale:**
- Separates reusable commands from project documentation
- Follows Claude Code best practices for custom commands
- Makes commit workflow easily accessible and consistent

**Implementation:**
- Created `.claude/commands/commit.md` with proper frontmatter
- Includes `allowed-tools` specification for required tools
- Handles `cargo clippy --fix` limitation with post-commit workflow
- Removed commit instructions from CLAUDE.md to avoid duplication

**Lessons Learned:**
- Slash commands need frontmatter with `description` and `allowed-tools`
- Command files should be prompts for Claude to execute, not documentation
- `cargo clippy --fix` requires clean working tree, so must run after commit
- Use `jj squash` to combine lint fixes back into main commit

### Frontend: HTMX + Pico.css
**Decision:** Server-side rendered HTML with HTMX for interactivity
**Rationale:**
- Simpler than full SPA for rule browsing use case
- Better SEO and initial load performance
- Reduces complexity compared to separate frontend framework

**Lessons Learned:**
- HTMX works well for quiz flow with form submissions and partial page updates
- Relative paths in form actions can be problematic - absolute paths more reliable
- Hidden form inputs provide clean session tracking without cookies

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

### Chart Theme Control Implementation (2025-08-11)
**What worked well:**
- Client Hints (`Accept-CH`, `Sec-CH-Prefers-Color-Scheme`) provide seamless browser theme detection
- Charming library's built-in theme system (`CharmingTheme::Default`/`Dark`) handles theming elegantly
- Custom axum extractors with `FromRequestParts` integrate cleanly with handler functions
- Middleware approach for global Client Hints header works across all routes
- Theme detection is automatic and requires no user interaction

**What to improve:**
- Initial implementation made assumptions about charming API instead of consulting documentation
- Should verify library capabilities before implementing workarounds

**Technical debt:**
- None identified - implementation uses proper library patterns

**Key decisions:**
- Used Client Hints standard over manual theme toggles for better UX
- Applied theme at renderer level (`ImageRenderer::new(800, 400).theme(theme.into())`)
- Created reusable `Theme` enum with `Into<CharmingTheme>` conversion
- Added middleware globally rather than per-route for consistency

**Pair Programming Learning:**
- Added section to pair programming guide about handling unfamiliar libraries
- Emphasized importance of acknowledging knowledge gaps early
- Documented approach for collaborative API exploration

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

**Strategic Pivot: External LLM Processing**
- **Problem**: Complex semantic analysis (overlapping terms, context-dependent meanings) would require significant development effort
- **Solution**: Used external LLM processing for one-time content transformation instead of building automation
- **Result**: Cross-reference generation completed in hours vs weeks of complex regex development
- **Key Insight**: Sometimes the most efficient path is external processing, not internal automation

**When to Choose External LLM Processing:**
- Semantic complexity exceeds simple pattern matching capabilities
- One-time content transformation with high linguistic nuance
- Development effort significantly exceeds LLM processing cost/time
- Task requires human-like understanding of context and meaning

---

## Implementation Strategy Lessons

### Tool Selection Framework
**Always consider three approaches for complex tasks:**
1. **Code Implementation**: Full programmatic solution
2. **External LLM Processing**: Ad-hoc processing for one-time transformations
3. **Manual Processing**: Human-driven approach

**Decision Criteria:**
- **Complexity**: How much semantic understanding is required?
- **Frequency**: Is this a one-time task or recurring operation?
- **Development Cost**: Implementation effort vs alternative approaches
- **Maintenance**: Long-term support and update requirements

### Content Processing Strategy
**For content curation tasks:**
- **Simple patterns**: Use regex and programmatic processing
- **Complex semantics**: Consider external LLM processing first
- **Recurring operations**: Build automation only if frequently needed
- **Quality requirements**: LLM + human review often better than pure automation

---

## Future Technical Decisions

### Questions to resolve:
1. Authentication strategy for different user types
2. Internationalization approach for multi-language support
3. Caching strategy for rule content
4. API versioning strategy
5. **NEW**: When to build vs buy vs LLM-process for future content features

### Research needed:
- Ultimate Frisbee rules data structure and format
- Integration options for Telegram/Discord bots
- Localization libraries for Rust web applications
- **NEW**: Cost-benefit analysis framework for LLM processing vs development

---

## Quiz System Implementation Lessons (Story 4)

### Session Tracking Without Cookies
**Approach:** Use hidden form inputs to carry session ID through quiz flow
**Benefits:**
- No GDPR cookie banner required
- Clean session isolation
- Works with all browsers/privacy settings

**Implementation:**
- Generate UUID session ID on quiz start
- Pass through hidden inputs in all forms
- Store attempts in database with anonymous session ID

### Database Design for Session Statistics
**Pattern:** Separate view models from database models for template rendering
**Rationale:**
- Keeps database models focused on persistence
- Template-specific fields (like computed statistics) stay in handlers
- Easier to evolve database schema independently

**Example:**
```rust
// Database model - no Serialize
pub struct SessionStatistics { ... }

// View model - for templates  
#[derive(Serialize)]
struct SessionStatsView { ... }
```

### URL Structure and Form Actions
**Issue:** Relative paths in forms can resolve incorrectly
**Solution:** Use absolute paths with template variables
```html
<!-- Instead of: action="submit" -->
<form action="/{{ language }}/quiz/{{ rule_set_slug }}/submit">
```

**Benefit:** Consistent routing regardless of current page structure

### Session Completion Flow
**Pattern:** Helper functions for shared logic between handlers
**Example:** `get_quiz_question_for_session()` used by both start and next question handlers
**Benefit:** DRY principle, consistent session completion detection

---

## Production Readiness Implementation Lessons (Epic 5)

### Story 1: Environment-Based Configuration System
**What worked well:**
- `config` crate handles TOML files and environment variable merging cleanly
- `dotenvy` crate seamlessly loads .env files for development
- Double underscore separator (`__`) works perfectly for nested configuration
- Environment variable validation catches missing secrets early

**What to improve:**
- Initial confusion about environment variable naming conventions
- Documentation needed clearer examples of REGELATOR__ prefix usage

**Technical debt:**
- None identified - clean implementation

**Key decisions:**
- Used TOML files for structured configuration with environment-specific overrides
- Implemented JWT secret as required environment variable (REGELATOR__SECURITY__JWT_SECRET)
- Environment variable prefix REGELATOR with __ separator for nested fields
- Added .env support for development convenience while keeping .env.example for documentation
- All import scripts now use configuration instead of hardcoded values

**Architecture Lessons:**
- Configuration loading should happen once at startup with validation
- Environment variables should override TOML settings for deployment flexibility
- Secrets must never be in TOML files - environment variables only
- Clear separation between development (.env) and production (environment) configuration

**Production Readiness Achieved:**
- Eliminated all TODO/production hardcoded values
- Secure secret management via environment variables
- Environment-specific deployment configuration
- Production-ready configuration validation and error handling

### Story 1.1: Code Organization and Module Structure
**What worked well:**
- Domain-based module boundaries (web, quiz, admin) align with team structure
- Rust's module system with re-exports maintains backward compatibility during refactoring
- All 23 existing tests pass without modification after restructuring
- Rust-code-reviewer agent confirmed production readiness of restructured code

**What to improve:**
- Minor unused variable warnings from restructuring (easily fixed with underscore prefixes)
- Could add module-level documentation for better developer onboarding
- Some error messages could be more descriptive for production debugging

**Technical debt:**
- Eliminated: Monolithic 1,700+ line files that were difficult to navigate
- Created: Well-organized modules with clear responsibilities

**Key decisions:**
- Split handlers.rs (1,234 lines) → handlers/{web,quiz,admin}.rs (555+261+234 lines)
- Split models.rs (505 lines) → models/{core,quiz,admin}.rs (193+252+25 lines)  
- Used `pub use *` in mod.rs files for seamless API compatibility
- Chose functional domain boundaries over technical layer boundaries

**Architecture Lessons:**
- Large files become maintenance bottlenecks as team grows
- Domain-driven module organization scales better than technical layer organization
- Re-export patterns in mod.rs enable safe refactoring without breaking dependent code
- Code review tools (rust-code-reviewer agent) provide valuable feedback on structural changes
- TodoWrite tool essential for tracking complex multi-step refactoring tasks

**Production Readiness Achieved:**
- Improved code maintainability for team development
- Better separation of concerns for debugging and monitoring
- Foundation for implementing logging, metrics, and observability
- Reduced cognitive load when working on specific functional areas

### Story 2: Comprehensive Observability with Tracing and Instrumentation
**What worked well:**
- `tracing` crate provides excellent structured logging with minimal overhead
- `#[instrument]` macro automatically captures function parameters and spans
- `color-eyre` significantly improves error reporting with better stack traces
- Configuration-based logging enables different formats (tree/JSON) per environment
- Span recording allows dynamic addition of context (usernames, IDs, operation details)
- `tracing-tree` provides beautiful hierarchical logging for development debugging

**What to improve:**
- Initial learning curve for understanding span context and recording patterns
- Need to balance instrumentation detail with performance overhead
- Some redundant context recording could be optimized

**Technical debt:**
- Eliminated: println! statements throughout import scripts and handlers
- Eliminated: Basic eyre error handling without context
- Created: Comprehensive observability infrastructure ready for production monitoring

**Key decisions:**
- Used `#[instrument]` macros on all handler functions for automatic span creation
- Added span recording for security-sensitive operations (username tracking, admin actions)
- Configured environment-specific logging (tree format for local, JSON for production)
- Replaced `eyre` with `color-eyre` for enhanced error context and stack traces
- Added structured logging configuration with levels, formats, and color control

**Architecture Lessons:**
- Tracing instrumentation should be added early in development, not retrofitted
- `#[instrument]` macro with selective field recording provides the right balance of detail vs performance
- Environment-based logging configuration enables development debugging while maintaining production efficiency
- Span context recording allows rich operational visibility without performance impact
- Import scripts benefit significantly from proper logging instead of println! debugging

**Production Readiness Achieved:**
- **Operational Visibility**: Complete request tracing with user context and timing information
- **Error Debugging**: Enhanced stack traces and error context for production troubleshooting
- **Audit Trail**: Structured logging captures all admin operations with user attribution
- **Performance Monitoring**: Automatic timing and span hierarchy for performance analysis
- **Development Experience**: Beautiful tree-formatted logs make local debugging significantly easier

**Security and Compliance Benefits:**
- **User Attribution**: All admin actions automatically logged with username context
- **Operation Tracking**: Complete audit trail of system modifications and access
- **Error Analysis**: Structured error reporting enables security incident investigation
- **Non-PII Logging**: Careful span recording avoids logging sensitive user data while maintaining operational visibility

### Story 1.2: Admin Authentication System Unification
**What worked well:**
- AdminToken newtype wrapper provides compile-time authentication guarantees
- Axum's FromRequestParts trait enables clean extractor implementation
- Automatic redirect to login page improves user experience
- Native async fn in trait support eliminates need for async_trait macro
- Rust's type system enforces authentication - handlers cannot compile without AdminToken parameter

**What to improve:**
- Initial attempt included unnecessary Clone trait requirement
- Had to adjust IntoResponse implementation for proper admin flow redirects

**Technical debt:**
- Eliminated: 60+ lines of duplicated authentication logic across 8 admin handlers
- Created: Clean, centralized authentication system with compile-time safety

**Key decisions:**
- Used newtype pattern (AdminToken(AdminClaims)) to prevent manual construction
- Implemented automatic redirect to /admin/login for authentication failures
- Made AdminToken extractor return admin context (username, admin_id) directly to handlers
- Leveraged Axum's native async fn in trait support instead of async_trait macro
- Authentication logic now centralized in single extractor implementation

**Architecture Lessons:**
- Axum extractors provide excellent abstraction for cross-cutting concerns like authentication
- Newtype wrappers with private fields ensure security invariants cannot be bypassed
- FromRequestParts enables early request processing before handler execution
- Redirect responses for authentication failures provide better UX than HTTP error codes
- Compile-time authentication guarantees prevent accidental security holes

**Security Benefits:**
- **Impossible to Forget**: Handlers must accept AdminToken parameter or compilation fails
- **Centralized Logic**: Single point of authentication reduces audit surface
- **Automatic Context**: Admin info available without additional database lookups
- **User Experience**: Authentication failures redirect to login instead of showing error pages

**Development Benefits:**
- **Code Maintainability**: Eliminated duplicated authentication boilerplate
- **Type Safety**: Rust's type system enforces authentication requirements at compile time
- **Rich Context**: Handlers get admin user info (username, admin_id) directly from token
- **Future-Proof**: New admin endpoints automatically require authentication token

### Quiz Session Management Refactoring
**What worked well:**
- Middleware approach for automatic cookie management eliminates boilerplate
- `QuizSession` extractor provides consistent session access across all handlers
- Cookie-based sessions much cleaner than hidden form inputs
- Axum's `merge` vs deprecated `nest` for root-level routing
- Context-aware UI dramatically improves user experience

**What to improve:**
- Initial attempt used deprecated `nest` at root level instead of `merge`
- URL paths shouldn't contain session IDs when cookies handle session state
- Template data should be minimal - don't pass data that handlers can provide

**Technical debt:**
- Eliminated: Hidden form inputs for session tracking across 3+ templates
- Eliminated: Manual cookie handling in individual handlers
- Eliminated: Session IDs in URL paths for security and cleanliness
- Created: Clean middleware-based session management with automatic cookie handling

**Key decisions:**
- Used dedicated `src/quiz_session.rs` module for separation of concerns
- Implemented middleware that automatically sets session cookies for quiz routes
- `QuizSession` extractor never rejects - always provides a valid session
- Context-aware quiz landing page shows progress and appropriate actions
- Session clearing uses cookie-based session ID rather than URL parameters

**Architecture Lessons:**
- **Middleware Pattern**: Automatic cross-cutting concerns (like session cookies) should be handled by middleware, not individual handlers
- **Extractor Pattern**: Custom extractors (like `AdminToken`, `QuizSession`) provide compile-time guarantees and clean handler signatures
- **URL Design**: URLs should represent resources, not sensitive session state
- **Context-Aware UX**: Applications should adapt UI based on user state rather than showing static interfaces
- **Router Composition**: Use `merge` for combining routers, not deprecated `nest` at root level

**Production Benefits:**
- **Enhanced UX**: Users see "Continue Quiz (5/20)" instead of generic "Start Quiz"
- **Better Security**: Session IDs not exposed in URLs or form data
- **Improved Observability**: Session IDs automatically logged in all quiz operations via middleware
- **Maintainability**: Centralized session logic reduces duplication and errors
- **Developer Experience**: Clean handler signatures with automatic session context

**User Experience Improvements:**
- **Smart Landing Page**: Shows progress bar and appropriate action based on session state
- **Session Continuity**: Progress preserved across browser sessions via HTTP-only cookies
- **Clear Navigation**: "Start over" option available when needed
- **Progress Visibility**: Users can see completion status at a glance

### Timestamp Consistency and Date Validation

**What worked well:**
- Chrono's serde integration provides automatic date validation at deserialization layer
- Database migration approach preserves existing data while changing schema types
- Using `NaiveDate.and_hms_opt()` to convert dates to datetime ranges for filtering
- "Less than start of next day" approach handles milliseconds correctly
- Repository method signatures with `Option<chrono::NaiveDate>` provide type safety

**What to improve:**
- Should have used proper timestamp types from the beginning
- String-based date manipulation was error-prone and difficult to maintain
- Manual date parsing scattered throughout handlers created maintenance burden

**Technical debt resolved:**
- Eliminated: String-based timestamp storage in quiz tables
- Eliminated: Manual string parsing and formatting for date operations  
- Eliminated: Inconsistent date handling between different table schemas
- Created: Comprehensive date validation pipeline from URL parameters to database
- Created: Type-safe date filtering with proper datetime range conversion

**Key decisions:**
- Used database migration to convert TEXT timestamps to TIMESTAMP types
- Implemented validation at HTTP parameter deserialization boundary
- Repository methods now accept validated `chrono::NaiveDate` types directly
- Date filtering uses proper datetime ranges (start of day to start of next day)

**Architecture Lessons:**
- **Early Validation**: Validate data types as early as possible in the request pipeline
- **Type Safety**: Use strong types (chrono::NaiveDate) instead of stringly-typed data
- **Database Schema**: Use proper database types rather than storing dates as text
- **Migration Strategy**: Database migrations can safely convert data types with proper conversion
- **Serde Integration**: Leverage library features (Chrono's serde) for automatic validation

**Production Benefits:**
- **Better UX**: Invalid dates in URLs return clear HTTP 400 errors immediately
- **Type Safety**: No runtime date parsing errors - all validation happens at request boundary
- **Performance**: Proper database indexes on timestamp columns for efficient filtering
- **Maintainability**: Centralized date handling logic eliminates scattered parsing code
- **Data Integrity**: Consistent timestamp handling across all analytics and reporting

**Data Visualization Enablement:**
- **Temporal Charts**: Real date types enable proper time-series visualizations
- **Date Filtering**: Analytics dashboards can reliably filter by date ranges
- **Chart Libraries**: Integration with charming/Apache ECharts now has clean date data
- **Export Formats**: CSV and Parquet exports will have properly typed date columns

---

*Last updated: 2025-08-10*