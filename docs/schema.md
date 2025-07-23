# Database Schema

## Overview

Normalized schema for storing Ultimate Frisbee rules with version-stable identifiers, multi-language support, and cross-referencing.

## Entity Relationship Diagram

```mermaid
erDiagram
    rule_sets ||--o{ versions : has
    rule_sets ||--o{ rules : contains
    
    rules ||--o{ rules : "parent/child"
    rules ||--o{ rule_content : "translated as"
    rules ||--o{ annotations : "documented by"
    rules ||--o{ rule_numbering_history : "numbered in"
    
    annotations ||--o{ annotation_content : "translated as"
    glossary_terms ||--o{ glossary_content : "translated as"
    figures ||--o{ figure_content : "translated as"
    
    rules ||--o{ cross_references : "references from"
    rules ||--o{ cross_references : "references to"
    annotations ||--o{ cross_references : "references from"
    glossary_terms ||--o{ cross_references : "referenced by"
    figures ||--o{ cross_references : "referenced by"
    
    versions ||--o{ rule_numbering_history : tracks

    rule_sets {
        uuid id PK
        text name "Indoor, Beach, Grass"
        text slug UK "indoor, beach, grass"
        text description
        timestamp created_at
        timestamp updated_at
    }

    versions {
        uuid id PK
        uuid rule_set_id FK
        text version_name "2025-2029"
        date effective_from
        date effective_to
        text description
        boolean is_current
        timestamp created_at
    }

    rules {
        uuid id PK
        text slug UK "spirit-respectful-language"
        uuid rule_set_id FK
        uuid version_id FK
        uuid parent_rule_id FK "nullable, for hierarchy"
        text current_number "1.3.7"
        integer sort_order "for consistent ordering"
        timestamp created_at
        timestamp updated_at
    }

    rule_content {
        uuid id PK
        uuid rule_id FK
        text language "en, de"
        text title "Spirit of the Game"
        text content_markdown
        uuid source_content_id FK "nullable, points to English original"
        timestamp created_at
        timestamp updated_at
    }

    rule_numbering_history {
        uuid id PK
        uuid rule_id FK
        uuid version_id FK
        text number_path "1.3.7"
        date effective_from
        date effective_to
    }

    annotations {
        uuid id PK
        uuid rule_id FK
        text annotation_key "16.1, spirit-contact-example"
        text slug UK "continuation-after-call"
        uuid rule_set_id FK
        uuid version_id FK
        timestamp created_at
        timestamp updated_at
    }

    annotation_content {
        uuid id PK
        uuid annotation_id FK
        text language "en, de"
        text title "Continuation after a Call"
        text content_markdown
        uuid source_content_id FK "nullable"
        timestamp created_at
        timestamp updated_at
    }

    glossary_terms {
        uuid id PK
        uuid rule_set_id FK
        uuid version_id FK
        text slug UK "act-of-throwing"
        text term_key "act_of_throwing"
        integer sort_order
        timestamp created_at
        timestamp updated_at
    }

    glossary_content {
        uuid id PK
        uuid term_id FK
        text language "en, de"
        text term "Act of throwing"
        text definition_markdown
        uuid source_content_id FK "nullable"
        timestamp created_at
        timestamp updated_at
    }

    figures {
        uuid id PK
        uuid rule_set_id FK
        uuid version_id FK
        text slug UK "playing-field-diagram"
        text figure_key "figure_1"
        text file_path "/static/images/rules/field-v2.svg"
        timestamp created_at
        timestamp updated_at
    }

    figure_content {
        uuid id PK
        uuid figure_id FK
        text language "en, de"
        text title "Figure 1 - Playing Field"
        text description
        text alt_text "Diagram showing Ultimate field dimensions"
        uuid source_content_id FK "nullable"
        timestamp created_at
        timestamp updated_at
    }

    cross_references {
        uuid id PK
        text from_type "rule, annotation, glossary_term"
        uuid from_id
        text to_type "rule, annotation, glossary_term, figure"
        uuid to_id
        text reference_text "rule 16.3, Figure 1"
        timestamp created_at
    }
```