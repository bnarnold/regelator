# Project Plan: UI/Template Cleanup

This document tracks progress on cleaning up the template display and improving rule presentation.

## Phase Status: ✅ COMPLETED

## Current Story

### Story 1: Fix Rule Display Formatting 🎯
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

## Story 2: Fix Single Rule View 🎯

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

## Next Stories After These Fixes

1. **Markdown Content Rendering** - Process the content as markdown
2. **Interactive HTMX Navigation** - Add smooth navigation
3. **Rule Search and Filtering** - Search within actual content
4. **Mobile-First Responsive Design** - Optimize for mobile

## Dependencies

- Requires database join operations (rule + rule_content tables)
- Reuses existing `build_rule_tree` infrastructure
- CSS updates for hierarchy display without `<ol>`