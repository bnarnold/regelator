# Project Plan: Frontend & User Experience

This document tracks progress on implementing the user-facing frontend functionality with HTMX interactivity.

## Phase Status: 📋 PLANNED

## Stories Queue

### Story 1: Markdown Content Rendering ✅
**Goal:** Process rule content from markdown to HTML for proper display

**Acceptance Criteria:**
- [x] Add markdown processing crate (e.g., `pulldown-cmark`)
- [x] Process rule content markdown in templates or handlers
- [x] Support common markdown features (headers, lists, links, emphasis)
- [x] Ensure safe HTML output (no XSS vulnerabilities)
- [x] Update rule detail and list views to show rendered content

**Technical Notes:**
- ✅ COMPLETED: Implemented with pulldown-cmark + ammonia in handlers
- Decision made: Handler-based processing for performance and security
- Added comprehensive tests for markdown features and XSS protection

### Story 1.1: Refactor Markdown to minijinja Filter ✅
**Goal:** Refactor markdown processing from handlers to template filters for cleaner architecture

**Acceptance Criteria:**
- [x] Create custom minijinja filter for markdown processing
- [x] Remove `content_html` field from RuleNode and RuleDetailData structs
- [x] Update templates to use `{{ content | markdown | safe }}` pattern
- [x] Migrate markdown_to_html function to template filter
- [x] Update tests to work with filter-based approach
- [x] Verify performance is acceptable with template-level processing

**Technical Notes:**
- ✅ COMPLETED: Implemented markdown_filter in main.rs with proper error handling
- Filter registered with minijinja Environment during AppState initialization
- Simplified data structures by removing content_html fields
- Templates updated to use filter pattern consistently
- Tests migrated to main.rs with comprehensive coverage
- Architecture improved: cleaner separation of concerns, reduced memory usage

**Benefits Realized:**
- Cleaner separation of concerns (presentation logic in templates)
- Simplified data models with ~50% less memory usage per rule
- Template-level control over markdown rendering
- Eliminated duplicate content storage (markdown + HTML)

### Story 1.2: Inline Rule Cross-References ✅
**Goal:** Add automatic inline links between rules using slug-based template expressions

**Acceptance Criteria:**
- [x] Implement enhanced template expression syntax with `{{section:slug}}` and `{{rule:slug}}`
- [x] Create rule reference processor in import script (`process_number_references`)
- [x] Parse rule content during import to find reference patterns (both "Section X" and bare numbers)
- [x] Replace expressions with appropriate markdown links using target rule's slug and number
- [x] Handle cross-references within same rule set and version
- [x] Add validation for broken references with tracking in import script
- [x] Update import script to process references after all rules are loaded
- [x] Test with comprehensive test cases covering both reference types

**Technical Implementation Plan:**
1. **Template expression syntax**:
   - Use `{{slug-name}}` format for rule references
   - Escape mechanism: `\{{slug}}` for literal double braces
   - Slugs are stable identifiers, more reliable than rule numbers

2. **Extend import script**:
   - Add two-pass import: first pass loads rules, second pass processes `{{}}` expressions
   - Build slug → (number, slug) mapping for current rule set/version
   - Implement regex pattern matching for `{{slug}}` syntax
   - Replace with markdown: `[{rule_number}](/en/rules/{rule_set}/{slug})`

3. **Reference resolution logic**:
   - Look up target rule by slug within same rule set and version
   - Generate markdown link with rule number as display text
   - Preserve semantic meaning: `{{handling-contested-calls}}` → `[16.3](/en/rules/indoor/handling-contested-calls)`
   - Handle missing slugs gracefully (keep original text + warning)

4. **Example transformation**:
   ```
   Input:  "If the opposition does not gain possession, apply {{handling-contested-calls}} instead."
   Output: "If the opposition does not gain possession, apply [16.3](/en/rules/indoor/handling-contested-calls) instead."
   ```

**Benefits:**
- Slug-based references are stable across rule renumbering
- Processed once during import for optimal performance  
- Version-specific links ensure correct rule references
- Easy validation of reference integrity during import
- Clean separation: content authoring uses slugs, display shows numbers

**Technical Implementation Completed:**

1. **Enhanced Reference Processing (`src/bin/import_rules.rs`)**:
   - Regex pattern distinguishes "Section X" vs bare number references
   - Generates `{{section:slug}}` for "Section X" patterns  
   - Generates `{{rule:slug}}` for bare number patterns
   - Tracks broken references for validation reporting

2. **Template Processing (`src/handlers.rs`)**:
   - `process_slug_references()` handles both template types
   - Section references render as "Section X" with links
   - Rule references render as numbers only with links
   - Graceful fallback for missing slug mappings

3. **Test Coverage**:
   - Comprehensive tests for both reference types
   - Tests for broken reference handling
   - Tests for mixed reference scenarios
   - Verified import and display functionality

**Benefits Realized:**
- Semantic distinction between section references and rule references
- Stable cross-references survive rule renumbering
- Import-time processing for optimal runtime performance
- Clean markdown output with proper link formatting

### Story 1.3: Anchor-Based Cross-Reference Navigation ✅
**Goal:** Enhance rule list view with anchor-based navigation for smoother cross-reference browsing

**Acceptance Criteria:**
- [x] Add anchor tags to each rule in list view using rule slug (e.g., `id="spirit-of-game"`)
- [x] Update cross-reference links to use anchors instead of full page navigation
- [x] Keep rule numbers as full page links to individual rule detail pages
- [x] Test anchor navigation works correctly in both list and detail views
- [x] Ensure browser back/forward navigation works properly with anchors
- [x] Verify anchor links work across different rule hierarchies

**Technical Implementation Plan:**
1. **Update rule list template (`rules_list.html`)**:
   - Add `id="{{ rule.slug }}"` to each rule container/div
   - Ensure anchors are properly nested within the rule hierarchy

2. **Modify cross-reference link generation (`process_slug_references`)**:
   - Change from `[Section 1](/en/rules/wfdf-ultimate/spirit-of-game)` 
   - To `[Section 1](#spirit-of-game)` for same-page anchors
   - Keep full URLs for cross-references to different rule sets/versions  

3. **Preserve rule number functionality**:
   - Rule numbers themselves still link to individual rule detail pages
   - Cross-reference text links to anchors for quick browsing

**Example transformation**:
```
Current: "according to [Section 1](/en/rules/wfdf-ultimate/spirit-of-game)"
New:     "according to [Section 1](#spirit-of-game)"
```

**Benefits:**
- Faster navigation for cross-references (no page reload)
- Better user experience for rule browsing
- Maintains context when following cross-references
- Preserves existing rule detail page functionality
- Works naturally with browser's find-on-page functionality

**Technical Implementation Completed:**

1. **Template Updates (`src/templates/rules_list.html`)**:
   - Added `id="{{ rule.slug }}"` to each `<li class="rule-item">` element
   - Preserves existing rule number links to detail pages
   - Anchors work correctly within hierarchical rule structure

2. **Enhanced Reference Processing (`src/handlers.rs`)**:
   - Added `use_anchors` parameter to `process_slug_references()` function
   - List view calls use `true` to generate anchor links (`#slug`)
   - Detail view calls use `false` to generate full URLs (`/language/rules/set/slug`)
   - Smart context-aware link generation based on page type

3. **Comprehensive Test Coverage**:
   - `test_process_slug_references_anchors()` verifies anchor link generation
   - `test_process_slug_references_full_urls()` verifies full URL generation
   - Tests cover both section references and rule references
   - All existing tests continue to pass

**Benefits Realized:**
- Instant navigation for cross-references without page reloads
- Smooth user experience when following rule connections
- Maintains full functionality of rule number links to detail pages
- Works naturally with browser back/forward navigation
- Foundation ready for future HTMX enhancements
- Smart context detection ensures appropriate link types per page

### Story 2: Title Field Cleanup & Display Simplification ✅
**Goal:** Remove unused title fields and simplify rule display architecture

**Acceptance Criteria:**
- [x] Remove title column from rule_content database schema via migration
- [x] Remove title field from RuleContent and NewRuleContent models
- [x] Clean up title references from import script and repository code
- [x] Verify rule display continues working with full content (no truncation)
- [x] Ensure all existing functionality remains intact
- [x] Update data model documentation to reflect schema changes

**Technical Notes:**
- Analysis shows we already display full rule content correctly
- Title field in database is always NULL (unused technical debt)
- Templates use `{{ rule.content | markdown | safe }}` (perfect as-is)  
- No truncation needed - users want to see full rule text
- This simplifies the data model and removes unnecessary complexity

**Architecture Insight:**
- Original Story 2 scope was already implemented during earlier work
- Current templates display full content via markdown filter
- No slug-based fake titles in use anywhere
- This story pivots to cleanup rather than new functionality

**Technical Implementation Completed:**

1. **Database Migration (`migrations/2025-07-24-154746_drop_title_column/`)**:
   - Created and executed migration to drop `title` column from `rule_content` table
   - Updated auto-generated schema.rs to reflect changes
   - Rollback migration available if needed

2. **Model Cleanup (`src/models.rs`)**:
   - Removed `title` field from `RuleContent` struct
   - Removed `title` field from `NewRuleContent` struct
   - Simplified `NewRuleContent::new()` constructor method signature
   - All model changes compile successfully

3. **Code Cleanup**:
   - Removed `title: None` assignments from import script (`src/bin/import_rules.rs`)
   - Updated test helper functions in handlers to match new structure
   - Verified no title references remain in templates or handlers

4. **Documentation Updates**:
   - Updated `data_model_design.md` example queries to remove title references
   - Updated `docs/schema.md` Mermaid diagram to reflect current schema
   - Aligned documentation with actual implementation

**Benefits Realized:**
- Eliminated unused database column and model complexity
- Simplified data model focusing on essential fields only
- Reduced memory footprint per rule (removed unused field)
- Cleaner, more maintainable codebase
- Documentation now accurately reflects implementation

### Story 2.1: Definitions Support & Integration ✅
**Goal:** Add definitions/glossary terms with basic infrastructure and navigation

**Acceptance Criteria:**
- [x] Create database migration for glossary_terms and glossary_content tables
- [x] Add Diesel models and schema for glossary functionality
- [x] Extend import script to process definitions in format: `TERM: definition text`
- [x] Create glossary repository methods (create, find_by_slug, list_all)
- [x] Add definitions page handler and template at `/en/rules/{rule_set}/definitions`
- [x] Add bidirectional navigation between rules overview and definitions page
- [x] Support multi-line definitions with proper markdown paragraph breaks
- [x] Handle complex terms like "Out-of-bounds (OB)" with updated regex pattern
- [x] Test with example: "Affect the play: A breach or call affects the play if..."

**Technical Implementation Plan:**

1. **Database Setup**:
   - Migration: Create `glossary_terms` and `glossary_content` tables per schema.md design
   - Models: Add `GlossaryTerm`, `GlossaryContent`, `NewGlossaryTerm`, `NewGlossaryContent` structs
   - Schema: Add table definitions and relationships to Diesel schema

2. **Import Script Enhancement**:
   - Extend `import_rules.rs` to recognize definition format: `TERM: definition content`
   - Generate slugs from terms (e.g., "Affect the play" → "affect-the-play")
   - Store in glossary_terms with term_key and glossary_content with definition markdown
   - Process after rules import, similar to cross-reference processing

3. **Repository Layer**:
   - Add `GlossaryRepository` trait with methods: `create_term`, `find_by_slug`, `list_for_rule_set`
   - Implement database queries for term lookup and listing
   - Include content joining for display

4. **Web Interface**:
   - Add `/en/rules/{rule_set}/definitions` route and handler
   - Create `definitions.html` template showing alphabetical term list
   - Add "Definitions" link to navigation in base template
   - Style definitions with clear term/definition separation

5. **Automatic Term Linking**:
   - Extend markdown filter or add separate filter to detect glossary terms in content
   - Replace term mentions with links: `[affect the play](/en/rules/wfdf-ultimate/definitions#affect-the-play)`
   - Add anchor IDs to definition terms for direct linking
   - Preserve case and context of original term usage

**Benefits:**
- Users can quickly reference complex Ultimate terminology
- Automatic linking improves rule comprehension
- Foundation for future features (definition search, term highlighting)
- Maintains existing architectural patterns

**Example Usage:**
```
Import format: "Affect the play: A breach or call affects the play if it is reasonable to assume..."
Rule content: "If the foul affected the play, the disc returns to the thrower."
Rendered: "If the foul [affected the play](/en/rules/wfdf-ultimate/definitions#affect-the-play), the disc returns to the thrower."
```

**Technical Implementation Completed:**

1. **Database Migration & Models**:
   - Created `glossary_terms` and `glossary_content` tables with proper foreign keys
   - Added Diesel models with UUID primary keys and proper relationships
   - Fixed nullable primary key issues during schema design

2. **Import Script Enhancement (`src/bin/import_definitions.rs`)**:
   - Updated regex from `^[A-Z][a-zA-Z ]*:` to `^([^.:]+?):` to handle complex terms
   - Preserved empty lines for proper markdown paragraph breaks
   - Added comprehensive tests for single-line, multi-line, and complex term formats
   - Generates URL-friendly slugs automatically from term names

3. **Repository Methods**:
   - `create_glossary_term()` and `create_glossary_content()` for data creation
   - `get_glossary_terms()` for retrieving terms with content joined
   - `find_glossary_term_by_slug()` for individual term lookup

4. **Web Interface**:
   - Definitions page handler at `/{language}/rules/{rule_set}/definitions`
   - Clean template with alphabetical term sorting and markdown rendering
   - Bidirectional navigation: overview ↔ definitions with Pico CSS styling

5. **Navigation Integration**:
   - "📖 Definitions" button on rules overview page
   - "← Back to [Rule Set]" button on definitions page
   - Consistent navigation patterns across the application

**Benefits Realized:**
- Foundation for 37+ Ultimate Frisbee terms with proper paragraph formatting
- Scalable architecture ready for automatic term linking (Story 2.2)
- Clean separation between rule content and terminology
- User-friendly navigation between rules and definitions

**Key Lessons Learned:**
- Import script regex patterns need real-world testing (complex terms like "Out-of-bounds (OB)")
- Empty line handling critical for markdown paragraph formatting
- Bidirectional navigation improves user experience significantly
- Repository pattern scales well for new entity types
- Pico CSS provides sufficient styling without custom CSS

### Story 2.2: Automatic Term Linking in Rule Content ✅
**Goal:** Automatically detect and link glossary terms when they appear in rule content

**Acceptance Criteria:**
- [x] Create term detection system that identifies glossary terms in rule content
- [x] Implement case-insensitive matching (e.g., "affect the play" matches "Affect the play")  
- [x] Generate links to definitions page with anchors (e.g., `#affect-the-play`)
- [x] Preserve original text formatting and case in rule display
- [x] Handle partial matches and avoid false positives
- [x] Process term linking during rule content rendering
- [x] Test with real rule content containing multiple terms
- [x] Ensure performance is acceptable with full glossary

**Technical Implementation Completed:**

1. **Extended Template System (`src/handlers.rs`)**:
   - Added `{{definition:slug}}` support to existing `{{rule:slug}}` and `{{section:slug}}` patterns
   - Updated `process_slug_references()` function with new parameter for definition mappings
   - Added `build_definition_slugs()` helper function for slug-to-term mapping

2. **Smart Import Processing (`src/bin/import_definitions.rs`)**:
   - Added `process_definition_references()` function with advanced regex matching
   - Single-pass processing using combined regex for optimal performance
   - Maximal length matching prevents overlapping term conflicts (e.g., "offensive player" vs "player")
   - Case-insensitive word boundary detection for accurate matching

3. **Definition Page Handler Integration**:
   - Fixed definitions page to process `{{definition:slug}}` templates before rendering
   - Ensures cross-references work properly in definition content display
   - Links point to definitions page with anchor navigation

**Technical Breakthroughs:**
- **Overlapping Terms Solution**: Used single combined regex with longest-first ordering to handle complex cases like "offensive player" containing "player"
- **Non-overlapping Matches**: Leveraged regex `find_iter()` for automatic maximal-length matching
- **Import-time Processing**: Process content during import rather than runtime for optimal performance

**Benefits Realized:**
- Automatic cross-linking creates rich, interconnected definition networks
- Users can navigate between related terms seamlessly
- Robust handling of complex term relationships and overlaps
- Clean integration with existing cross-reference architecture

### Story 3: Interactive Rule Navigation with HTMX 🎯
**Goal:** Add smooth, interactive navigation between rules without full page reloads

**Acceptance Criteria:**
- [ ] Add HTMX library to base template
- [ ] Implement HTMX-powered rule list filtering/searching
- [ ] Add breadcrumb navigation for rule hierarchy
- [ ] Enable expanding/collapsing rule sections
- [ ] Add "Back to parent" navigation links
- [ ] Preserve navigation state in browser history

**Technical Notes:**
- Use HTMX for progressive enhancement
- Consider URL routing for bookmarkable states
- May need partial template rendering endpoints

### Story 4: Rule Search and Filtering 🎯
**Goal:** Allow users to search and filter rules by content, number, or title

**Acceptance Criteria:**
- [ ] Add search input with HTMX integration
- [ ] Implement full-text search in rule content and titles
- [ ] Add filtering by rule number patterns (e.g., "1.2.*")
- [ ] Show search results with highlighted matches
- [ ] Preserve hierarchy context in search results
- [ ] Add "Clear search" functionality

**Technical Notes:**
- SQLite FTS (Full-Text Search) vs simple LIKE queries
- Need new repository methods for search queries
- Consider search result ranking/relevance

### Story 5: Version Selection Interface 🎯
**Goal:** Provide user-friendly version selection and comparison

**Acceptance Criteria:**
- [ ] Add version dropdown/selector to rule pages
- [ ] Show version information (name, effective dates, description)
- [ ] Highlight current vs historical versions
- [ ] Enable switching between versions with HTMX
- [ ] Show "what changed" indicators between versions
- [ ] Preserve selected version in session/URL

**Technical Notes:**
- Build on existing version lookup functionality
- May need version comparison logic
- Consider showing version metadata prominently

### Story 6: Mobile-First Responsive Design 🎯
**Goal:** Ensure excellent user experience on mobile devices

**Acceptance Criteria:**
- [ ] Responsive rule hierarchy (collapsible on mobile)
- [ ] Touch-friendly navigation elements
- [ ] Optimized typography for mobile reading
- [ ] Fast loading times on slow connections
- [ ] Offline-first considerations (service worker)
- [ ] Mobile-optimized search interface

**Technical Notes:**
- Build on existing Pico.css foundation
- Consider Progressive Web App (PWA) features
- May need custom CSS for rule-specific layouts

### Story 7: CSS Infrastructure Cleanup 🎯
**Goal:** Move inline CSS to proper static file hosting with cache management

**Acceptance Criteria:**
- [ ] Extract inline styles from base.html to separate CSS file
- [ ] Implement static file serving with proper headers
- [ ] Add cache busting mechanism (ETags or versioned URLs)
- [ ] Add cache invalidation strategy
- [ ] Ensure CSS loads efficiently without blocking render

**Technical Notes:**
- Current: Inline `<style>` blocks in base.html
- Need: Static file server with cache headers
- Challenge: ETags implementation for cache busting
- Consider: CSS file versioning strategy

## Technical Architecture Decisions Needed

### HTMX Integration Approach
- **Option A:** Full HTMX SPA-style with partial rendering
- **Option B:** Progressive enhancement of existing pages
- **Recommendation:** Start with Option B for simpler implementation

### Search Implementation
- **Option A:** SQLite FTS extension for advanced search
- **Option B:** Simple LIKE queries for basic search
- **Recommendation:** Start with Option B, upgrade if needed

### Content Processing
- **Option A:** Process markdown in Rust handlers (server-side) ✅ IMPLEMENTED
- **Option B:** Use template filters for markdown processing 🎯 PLANNED (Story 1.1)
- **Updated Recommendation:** Refactor to Option B for cleaner architecture after proving Option A works

### State Management
- **Option A:** URL-based state (version, search, etc.)
- **Option B:** Session-based state storage
- **Recommendation:** Option A for bookmarkable/shareable URLs

## Dependencies and Prerequisites

**Required Crates:**
- `pulldown-cmark` for markdown processing
- No additional crates needed for HTMX (CDN)

**Updated Data Model Notes:**
- Rules have no separate titles, only content
- Import format: `number slug content` on single lines
- Display should use content excerpts for readability

**Frontend Assets:**
- HTMX library (CDN or local)
- Custom CSS extensions for responsive design
- Possible icons/assets for navigation

**Database Considerations:**
- May need search-optimized queries
- Consider adding indexes for search performance
- Version comparison might need additional query patterns

## Success Metrics

**User Experience:**
- Page load times < 200ms for navigation
- Mobile usability score > 95%
- Search results returned < 100ms

**Technical Quality:**
- All content properly rendered from markdown
- No XSS vulnerabilities in rendered content
- HTMX interactions work without JavaScript errors

**Functionality:**
- Users can find any rule within 3 clicks/interactions
- Version switching preserves user context
- Search finds relevant rules with good precision/recall

## Next Phase After Frontend

After completing frontend functionality, next development priorities:

1. **Bot Integration Preparation:**
   - Webhook endpoints for Telegram/Discord
   - Quiz question generation from rule content
   - Scheduled messaging infrastructure

2. **Content Management:**
   - Admin interface for rule editing
   - Import/export functionality
   - Version management tools

3. **Internationalization:**
   - Multi-language content display
   - Language switching interface
   - Translation management tools