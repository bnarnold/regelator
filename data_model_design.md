# Data Model Design

## Overview

This document explains the design decisions and principles behind the Regelator database schema for storing Ultimate Frisbee rules data.

## Key Design Principles

### Stable Identifiers
- **UUIDs** for all primary keys ensure rules maintain identity across version updates
- **Semantic slugs** provide human-readable URLs (`/rules/spirit-respectful-language`)
- **Numbering history** tracks how "1.3.7" might become "1.3.6" in future versions

### Proper Normalization
- **Core entities** (`rules`, `annotations`, `glossary_terms`, `figures`) contain only structural data
- **Content tables** (`rule_content`, `annotation_content`, etc.) contain language-specific text
- **One-to-many relationship** allows multiple translations per entity
- **Source content references** link translations back to English originals

### Hierarchical Rules
- Rules can have parent/child relationships via `parent_rule_id`
- `sort_order` ensures consistent display ordering within sections
- Current numbering stored separately from hierarchy for flexibility

### Multi-language Support
- Language-specific content normalized into separate tables
- `source_content_id` in content tables points to English original
- Missing translations can fall back to English content
- Consistent UUIDs across languages for the same logical rule

### Cross-referencing
- Generic `cross_references` table handles all inter-document links
- Supports rule→rule, rule→definition, annotation→rule, etc.
- `reference_text` preserves original context ("as per rule 16.3")

### Version Management
- Rule sets have versions (2025-2029 edition)
- Numbering history tracks changes over time with effective date ranges
- Current version flag for easy querying

## Usage Patterns

### Rule Display
```sql
-- Get rule with English content
SELECT r.slug, r.current_number, rc.title, rc.content_markdown
FROM rules r
JOIN rule_content rc ON r.id = rc.rule_id
WHERE r.slug = 'spirit-respectful-language' AND rc.language = 'en';
```

### Translation Workflow
```sql
-- Get rule with German translation, fallback to English
SELECT r.slug, r.current_number,
       COALESCE(rc_de.title, rc_en.title) as title,
       COALESCE(rc_de.content_markdown, rc_en.content_markdown) as content
FROM rules r
JOIN rule_content rc_en ON r.id = rc_en.rule_id AND rc_en.language = 'en'
LEFT JOIN rule_content rc_de ON r.id = rc_de.rule_id AND rc_de.language = 'de'
WHERE r.slug = 'spirit-respectful-language';
```

### Cross-reference Resolution
```sql
-- Find all rules that reference a specific glossary term
SELECT r.slug, r.current_number, rc.title
FROM cross_references cr
JOIN rules r ON cr.from_id = r.id
JOIN rule_content rc ON r.id = rc.rule_id AND rc.language = 'en'
WHERE cr.to_type = 'glossary_term' AND cr.to_id = ?;
```

### Historical Lookup
```sql
-- Find what number a rule had in a previous version
SELECT rnh.number_path
FROM rule_numbering_history rnh
JOIN rules r ON rnh.rule_id = r.id
WHERE r.slug = 'spirit-respectful-language'
  AND rnh.effective_from <= '2021-01-01'
  AND (rnh.effective_to IS NULL OR rnh.effective_to > '2021-01-01');
```

## Implementation Strategy

### Phase 1: Core Entities
- Implement `rule_sets`, `versions`, `rules`, `rule_content`
- Basic hierarchical rule storage
- Single language (English) support

### Phase 2: Cross-referencing
- Add `cross_references` table
- Parse and link rule references in markdown content
- Implement glossary terms and figures

### Phase 3: Multi-language
- Add content tables for all entities
- Translation management interface
- Language fallback logic

### Phase 4: Annotations & History
- Full annotation support with content tables
- Rule numbering history tracking
- Version comparison features

## Schema Maintenance

The canonical schema is maintained in `docs/schema.md` as a Mermaid diagram. All schema changes should:

1. Update the Mermaid diagram first
2. Create corresponding Diesel migrations
3. Update this design document if principles change
4. Update example queries if schema changes affect them