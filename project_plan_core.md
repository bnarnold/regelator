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