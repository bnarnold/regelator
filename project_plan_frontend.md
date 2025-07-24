# Project Plan: Frontend & User Experience

This document tracks progress on implementing the user-facing frontend functionality with HTMX interactivity.

## Phase Status: 📋 PLANNED

## Stories Queue

### Story 1: Markdown Content Rendering 🎯
**Goal:** Process rule content from markdown to HTML for proper display

**Acceptance Criteria:**
- [ ] Add markdown processing crate (e.g., `pulldown-cmark`)
- [ ] Process rule content markdown in templates or handlers
- [ ] Support common markdown features (headers, lists, links, emphasis)
- [ ] Ensure safe HTML output (no XSS vulnerabilities)
- [ ] Update rule detail and list views to show rendered content

**Technical Notes:**
- Current technical debt: raw markdown text displayed instead of rendered HTML
- Need to decide: process in Rust handler vs template filter
- Consider caching rendered content for performance

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
- **Option A:** Process markdown in Rust handlers (server-side)
- **Option B:** Use template filters for markdown processing
- **Recommendation:** Option A for better performance and security

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