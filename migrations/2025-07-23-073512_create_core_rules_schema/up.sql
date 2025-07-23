-- Create core entities for Ultimate Frisbee rules storage
-- Phase 1: rule_sets, versions, rules, rule_content

CREATE TABLE rule_sets (
    id TEXT PRIMARY KEY,  -- UUID as TEXT in SQLite
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE versions (
    id TEXT PRIMARY KEY,
    rule_set_id TEXT NOT NULL REFERENCES rule_sets(id),
    version_name TEXT NOT NULL,
    effective_from DATE NOT NULL,
    effective_to DATE,
    description TEXT,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL,
    rule_set_id TEXT NOT NULL REFERENCES rule_sets(id),
    version_id TEXT NOT NULL REFERENCES versions(id),
    parent_rule_id TEXT REFERENCES rules(id),
    number TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE rule_content (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL REFERENCES rules(id),
    language TEXT NOT NULL DEFAULT 'en',
    title TEXT,
    content_markdown TEXT NOT NULL,
    source_content_id TEXT REFERENCES rule_content(id),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX idx_rules_rule_set_version ON rules(rule_set_id, version_id);
CREATE INDEX idx_rules_parent ON rules(parent_rule_id);
CREATE INDEX idx_rules_number ON rules(number);
CREATE INDEX idx_rule_content_rule_language ON rule_content(rule_id, language);
CREATE INDEX idx_versions_rule_set_current ON versions(rule_set_id, is_current);

-- Ensure only one current version per rule set
CREATE UNIQUE INDEX idx_versions_unique_current 
ON versions(rule_set_id) 
WHERE is_current = TRUE;

-- Same logical rule can only appear once per version
CREATE UNIQUE INDEX idx_rules_slug_version ON rules(slug, version_id);

-- Triggers to auto-update updated_at timestamps
CREATE TRIGGER update_rule_sets_updated_at
    AFTER UPDATE ON rule_sets
    FOR EACH ROW
BEGIN
    UPDATE rule_sets SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_rules_updated_at
    AFTER UPDATE ON rules
    FOR EACH ROW
BEGIN
    UPDATE rules SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER update_rule_content_updated_at
    AFTER UPDATE ON rule_content
    FOR EACH ROW
BEGIN
    UPDATE rule_content SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;