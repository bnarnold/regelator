-- Fix primary key fields to be non-nullable
-- SQLite doesn't support ALTER COLUMN, so we need to recreate tables

-- Backup existing data
CREATE TEMPORARY TABLE rule_sets_backup AS SELECT * FROM rule_sets;
CREATE TEMPORARY TABLE rules_backup AS SELECT * FROM rules;
CREATE TEMPORARY TABLE rule_content_backup AS SELECT * FROM rule_content;
CREATE TEMPORARY TABLE versions_backup AS SELECT * FROM versions;

-- Drop existing tables (foreign keys will be recreated)
DROP TABLE rule_content;
DROP TABLE rules;
DROP TABLE versions;
DROP TABLE rule_sets;

-- Recreate rule_sets with non-nullable id
CREATE TABLE rule_sets (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(slug)
);

-- Recreate versions with non-nullable id
CREATE TABLE versions (
    id TEXT PRIMARY KEY NOT NULL,
    rule_set_id TEXT NOT NULL,
    version_name TEXT NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE,
    description TEXT,
    is_current BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_set_id) REFERENCES rule_sets (id)
);

-- Recreate rules with non-nullable id
CREATE TABLE rules (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL,
    rule_set_id TEXT NOT NULL,
    version_id TEXT NOT NULL,
    parent_rule_id TEXT,
    number TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_set_id) REFERENCES rule_sets (id),
    FOREIGN KEY (version_id) REFERENCES versions (id),
    FOREIGN KEY (parent_rule_id) REFERENCES rules (id),
    UNIQUE(slug, rule_set_id, version_id)
);

-- Recreate rule_content with non-nullable id
CREATE TABLE rule_content (
    id TEXT PRIMARY KEY NOT NULL,
    rule_id TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'en',
    content_markdown TEXT NOT NULL,
    source_content_id TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES rules (id),
    FOREIGN KEY (source_content_id) REFERENCES rule_content (id),
    UNIQUE(rule_id, language)
);

-- Restore data
INSERT INTO rule_sets SELECT * FROM rule_sets_backup;
INSERT INTO versions SELECT * FROM versions_backup;
INSERT INTO rules SELECT * FROM rules_backup;
INSERT INTO rule_content SELECT * FROM rule_content_backup;

-- Clean up backup tables
DROP TABLE rule_sets_backup;
DROP TABLE versions_backup;
DROP TABLE rules_backup;
DROP TABLE rule_content_backup;

-- Recreate indexes
CREATE INDEX idx_rules_rule_set_version ON rules(rule_set_id, version_id);
CREATE INDEX idx_rules_parent ON rules(parent_rule_id);
CREATE INDEX idx_rule_content_rule_id ON rule_content(rule_id);
CREATE INDEX idx_rule_content_language ON rule_content(language);