-- Drop triggers first
DROP TRIGGER IF EXISTS update_rule_content_updated_at;
DROP TRIGGER IF EXISTS update_rules_updated_at;
DROP TRIGGER IF EXISTS update_rule_sets_updated_at;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS rule_content;
DROP TABLE IF EXISTS rules;
DROP TABLE IF EXISTS versions;
DROP TABLE IF EXISTS rule_sets;