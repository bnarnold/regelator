# Project Plan: Core Functionality

This document tracks progress on implementing the core rule display and data management functionality.

## Phase Status: ✅ COMPLETED

## Stories Completed

### Story 1: Version Lookup Implementation ✅

**Goal:** Implement version lookup by name functionality to handle version query parameters

**Acceptance Criteria:**

- [x] Add `get_version_by_name` method to repository
- [x] Update handlers to use version lookup instead of returning errors
- [x] Support both current version (no parameter) and specific version (with parameter)
- [x] Version lookup works for both list and detail views

**Technical Notes:**

- Added proper database query with join between versions and rule sets
- Handles missing versions gracefully with appropriate error messages

### Bugfix Story: Fix Rule Display Formatting ✅

**Goal:** Clean up rule list display to avoid duplicate numbering and improve content presentation

**Current Issues Identified:**

- `<ol>` elements create automatic numbering that duplicates rule numbers (e.g., "1. 1. Spirit of Game")
- Rule content is not being displayed, only slug-based titles
- Need direct links on rule numbers for navigation
- Template should show actual rule content instead of generated titles

**Acceptance Criteria:**

- [x] Remove automatic `<ol>` numbering (use `list-style: none` or different structure)
- [x] Make rule numbers clickable links to individual rules
- [x] Display actual rule content from database instead of slug-based titles
- [x] Update rule tree template to show content excerpts
- [x] Ensure hierarchy is still visually clear without `<ol>` indicators
- [x] Test with long content to ensure proper truncation/display

**Technical Implementation Plan:**

1. **Update Templates:**
   - Change `<ol>` to `<div>` or use `<ol style="list-style: none">`
   - Make rule numbers (`{{ rule.number }}`) into clickable links
   - Replace `{{ rule.title }}` with actual content display

2. **Update Handler Logic:**
   - Modify `build_rule_tree` to fetch rule content from database
   - Join with `rule_content` table during tree building
   - **IMPORTANT: Show full content - DO NOT truncate rules content**

3. **Update RuleNode Structure:**
   - Replace `title: String` with `content: String`
   - **IMPORTANT: Show full content - DO NOT truncate**
   - Ensure content is properly escaped for HTML

**Database Query Changes Needed:**

```rust
// Current: only fetches rule metadata
let rules = repo.get_rules_for_version(&version.id)?;

// New: needs to fetch rule + content
let rules_with_content = repo.get_rules_with_content_for_version(&version.id, &language)?;
```

**Template Changes:**

```html
<!-- Current (problematic) -->
<ol class="rule-list">
    <li class="rule-item">
        <a href="/{{ language }}/rules/{{ rule_set_slug }}/{{ rule.slug }}">
            <span class="rule-number">{{ rule.number }}</span>
            <span class="rule-title">{{ rule.title }}</span>
        </a>
    </li>
</ol>

<!-- New (fixed) -->
<div class="rule-list">
    <div class="rule-item">
        <a href="/{{ language }}/rules/{{ rule_set_slug }}/{{ rule.slug }}" class="rule-number-link">
            {{ rule.number }}
        </a>
        <div class="rule-content">
            {{ rule.content }}
        </div>
    </div>
</div>
```

**CSS Considerations:**

- Style `.rule-number-link` to look like a clickable number
- Use indentation or borders to show hierarchy instead of `<ol>` nesting
- Ensure content is readable with proper spacing
- Consider truncation indicators (ellipsis) for long content

**Testing Requirements:**

- [x] Verify no duplicate numbering appears
- [x] Confirm rule numbers are clickable and navigate correctly
- [x] Test with various content lengths (short, medium, very long) - **show all content in full**
- [x] Verify hierarchy is visually clear
- [x] Test responsive behavior on mobile
- [x] Confirm content displays properly from database

**Files to Modify:**

1. [x] `src/templates/rules_list.html` - Fix template structure
2. [x] `src/handlers.rs` - Update `RuleNode` and `build_rule_tree`
3. [x] `src/repository.rs` - Add method to get rules with content
4. [x] `src/templates/base.html` - Add CSS for new structure

**Success Criteria:**

- [x] Rule lists show "1.2.3 [actual rule content]" instead of "1. 1.2.3 generated-title"
- [x] Rule numbers are clickable links
- [x] Hierarchy is visually clear without duplicate numbering
- [x] Content displays properly from database
- [x] No semantic HTML violations

**Technical Notes:**

- This addresses the current technical debt of using slug-based titles
- Improves accessibility by making numbers focusable links
- Sets foundation for content-based search/filtering
- May need performance optimization for large rule sets

**Priority:** ✅ COMPLETED - Fixed user-visible display issues and technical debt

## Bugfix story: Fix Single Rule View ✅

### Issues Identified

1. **Uses title instead of content**: Template displays `{{ rule.title }}` and `{{ child.title }}` but titles are NULL in database
2. **Lexicographical sorting**: Child rules not numerically sorted (e.g., "1.10" comes before "1.2")  
3. **Only first level hierarchy**: Shows immediate children only, not full nested hierarchy

### Implementation Plan

**Step 1: Update Data Structures**

- Remove `title` field from `RuleDetailData`
- Change `RuleData.title` to `RuleData.content`
- Change `child_rules: Vec<RuleData>` to `child_rules: Vec<RuleNode>` for hierarchy

**Step 2: Update Handler Logic**

- Remove title assignment in `RuleDetailData` creation
- Replace `get_child_rules()` with filtered `get_rules_with_content_for_version()`
- Build full hierarchy using existing `build_rule_tree` function
- Ensure numeric sorting is applied

**Step 3: Update Template**

- Remove `{{ rule.title }}` display (show content only)
- Replace `{{ child.title }}` with `{{ child.content }}`
- Use recursive macro for full hierarchy (like rules_list.html)
- Apply consistent CSS classes

**Success Criteria:**

- [x] Rule detail shows content (not NULL title)
- [x] Child rules show actual content excerpts (not slug-based titles)
- [x] Child rules sorted numerically (1.1, 1.2, 1.10)
- [x] Full multi-level hierarchy displayed
- [x] Navigation links work correctly
- [x] Consistent styling with rules list
- [x] **BONUS:** Added parent rule navigation link

**Files to Modify:**

1. [x] `src/handlers.rs` - Update data structures and handler logic
2. [x] `src/templates/rule_detail.html` - Fix template to use content
3. [x] `src/repository.rs` - Added `get_rule_by_id` method for parent lookup
4. [x] Tests - Updated and verified hierarchy and sorting work correctly

### Story 2: Rule Sorting and Hierarchy ✅

**Goal:** Fix rule sorting to use numeric comparison instead of string comparison

**Acceptance Criteria:**

- [x] Rules sorted numerically (1.2.10 comes after 1.2.9, not before 1.2.2)
- [x] Hierarchical sorting works at all levels of nesting
- [x] Root rules sorted correctly
- [x] Child rules sorted correctly within their parent

**Technical Notes:**

- Replaced string sorting with numeric parsing of rule number segments
- Used HashMap with `entry().or_default()` pattern for clean tree building
- Implemented recursive sorting for all levels of hierarchy

### Story 3: Template Rendering ✅

**Goal:** Fix MiniJinja macro rendering for hierarchical rule display

**Acceptance Criteria:**

- [x] Template macros work correctly without runtime errors
- [x] Hierarchical rule structure displays properly
- [x] Rules display in correct numerical order

**Technical Notes:**

- MiniJinja macros called directly without function wrapper
- Recursive template structure handles unlimited nesting levels

### Story 4: Data Import System ✅

**Goal:** Create import script for populating database with rule data

**Acceptance Criteria:**

- [x] Import script reads from stdin
- [x] Supports single-line format: `number slug content`
- [x] Automatically detects hierarchy from rule numbers
- [x] Creates proper database relationships
- [x] Uses UUID v7 for all identifiers
- [x] Uses explicit slugs for URL control

**Technical Notes:**

- Regex pattern `^((?:\d+\.)+)\s+(\S+)\s+(.+)$` to match number, slug, content
- Automatic parent-child relationship detection
- Proper sorting ensures parents created before children
- No titles - rules only have content

### Story 5: Testing Infrastructure ✅

**Goal:** Add comprehensive tests for rule sorting and tree building logic

**Acceptance Criteria:**

- [x] Test root level sorting
- [x] Test hierarchical sorting
- [x] Test deep hierarchy (3+ levels)
- [x] Test mixed hierarchy scenarios
- [x] Test recursive sorting function

**Technical Notes:**

- 5 comprehensive test cases covering all scenarios
- Helper function for creating test rules
- Tests verify both structure and ordering

## Technical Architecture Decisions Made

1. **Rule Hierarchy:** Parent-child relationships stored in database, tree built in application layer
2. **Sorting:** Numeric comparison by parsing rule number segments (split by dots)
3. **Data Import:** Single-pass stdin reading with regex-based rule detection
4. **Tree Building:** HashMap-based approach using `entry().or_default()` pattern
5. **Testing:** Unit tests focused on core business logic (sorting/hierarchy)

## Next Phase Recommendations

The core functionality is solid. Next development should focus on:

1. **User Experience Improvements:**
   - Markdown processing for rule content
   - Proper title retrieval from database
   - Configuration management

2. **Content Management:**
   - Web interface for rule editing
   - Version management UI
   - Multi-language support

3. **API Development:**
   - REST API for external integrations
   - Bot integration preparation

## Database Schema Status

Current schema supports:

- ✅ Multiple rule sets
- ✅ Version management with effective dates
- ✅ Hierarchical rules with parent-child relationships
- ✅ Multi-language content support
- ✅ Proper indexing and constraints

## Dependencies and Technical Debt

**Current Technical Debt:**

- Markdown processing not implemented (raw text displayed)
- Rule titles use slugs instead of content titles  
- Database URL hardcoded in multiple places

**No Blocking Issues:** All technical debt items are non-critical and don't prevent core functionality.

