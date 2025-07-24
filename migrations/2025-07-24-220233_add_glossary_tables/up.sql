-- Create glossary_terms table
CREATE TABLE glossary_terms (
    id TEXT PRIMARY KEY NOT NULL,
    rule_set_id TEXT NOT NULL,
    version_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_set_id) REFERENCES rule_sets (id),
    FOREIGN KEY (version_id) REFERENCES versions (id),
    UNIQUE(slug, rule_set_id, version_id)
);

-- Create glossary_content table
CREATE TABLE glossary_content (
    id TEXT PRIMARY KEY NOT NULL,
    term_id TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'en',
    term TEXT NOT NULL,
    definition_markdown TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (term_id) REFERENCES glossary_terms (id),
    UNIQUE(term_id, language)
);

-- Create indexes for performance
CREATE INDEX idx_glossary_terms_rule_set_version ON glossary_terms(rule_set_id, version_id);
CREATE INDEX idx_glossary_terms_slug ON glossary_terms(slug);
CREATE INDEX idx_glossary_content_term_id ON glossary_content(term_id);
CREATE INDEX idx_glossary_content_language ON glossary_content(language);