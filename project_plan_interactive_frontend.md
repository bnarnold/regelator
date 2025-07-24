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

### Story 2: Rule Display Enhancement 🎯
**Goal:** Improve rule display using content excerpts instead of slug-based titles

**Acceptance Criteria:**
- [ ] Update rule tree building to use content excerpts for display
- [ ] Show first N characters of rule content as preview in lists
- [ ] Update rule list views to show content previews
- [ ] Handle long content gracefully (truncation with ellipsis)
- [ ] Ensure display works for all languages

**Technical Notes:**
- Rules don't have separate titles, only content
- Current technical debt: using `slug.replace('-', " ")` for display
- Need to join with rule_content table during tree building
- Consider content truncation strategy (word boundaries, character limits)

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