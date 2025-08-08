use color_eyre::{eyre::WrapErr, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead};
use tracing::{info, warn};
use uuid::Uuid;

use regelator::config::{Config, ImportConfig};
use regelator::models::*;
use regelator::repository::RuleRepository;

#[derive(Debug)]
struct RuleData {
    number: String,
    slug: String,
    content: String,
}

fn parse_rule_number(number: &str) -> Vec<u32> {
    number.split('.').map(|s| s.parse().unwrap_or(0)).collect()
}

fn find_parent_rule(rules: &HashMap<String, String>, current_number: &str) -> Option<String> {
    let current_parts = parse_rule_number(current_number);
    if current_parts.len() <= 1 {
        return None; // Top-level rule has no parent
    }

    // Parent has one less level
    let parent_parts = &current_parts[..current_parts.len() - 1];
    let parent_number = parent_parts
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".");

    rules.get(&parent_number).cloned()
}

/// Process rule number references in content and replace with {{slug}} templates
fn process_number_references(
    content: &str,
    number_to_slug: &HashMap<String, String>, // rule number -> slug
) -> (String, Vec<String>) {
    // Match rule references: numbers with dots OR numbers prefixed by "Section"
    let reference_pattern =
        Regex::new(r"\b(?:Section\s+(\d+(?:\.\d+)*)|(\d+\.\d+(?:\.\d+)*))\b").unwrap();
    let mut processed_content = content.to_string();
    let mut broken_references = Vec::new();

    // Find all rule reference patterns and collect replacements
    let mut replacements = Vec::new();

    for mat in reference_pattern.find_iter(content) {
        let full_match = mat.as_str();
        let captures = reference_pattern.captures(full_match).unwrap();

        let rule_number = if let Some(section_num) = captures.get(1) {
            section_num.as_str() // "Section 16" -> "16"
        } else {
            captures.get(2).unwrap().as_str() // "16.3" -> "16.3"
        };

        if let Some(slug) = number_to_slug.get(rule_number) {
            // Replace with markdown link - both "Section X" and "X.Y" use rule: scheme
            let markdown_link = format!("[{full_match}](rule:{slug})");
            replacements.push((mat.start(), mat.end(), markdown_link));
        } else {
            // Keep original but track as potential broken reference
            broken_references.push(rule_number.to_string());
        }
    }

    // Apply replacements from end to start to preserve indices
    for (start, end, replacement) in replacements.into_iter().rev() {
        processed_content.replace_range(start..end, &replacement);
    }

    (processed_content, broken_references)
}

fn read_rules_from_stdin() -> Result<Vec<RuleData>> {
    let stdin = io::stdin();
    let rule_pattern = Regex::new(r"^((?:\d+\.)+)\s+(\S+)\s+(.+)$").unwrap();
    let mut rules = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();

        if rule_pattern.is_match(line) {
            // Extract rule number, slug, and content
            if let Some(caps) = rule_pattern.captures(line) {
                let number_with_dot = caps.get(1).unwrap().as_str();
                let number = number_with_dot.trim_end_matches('.').to_string();
                let slug = caps.get(2).unwrap().as_str().to_string();
                let content = caps.get(3).unwrap().as_str().to_string();

                rules.push(RuleData {
                    number,
                    slug,
                    content,
                });
            }
        }
    }

    Ok(rules)
}

fn import_rules(rule_data: Vec<RuleData>) -> Result<()> {
    // Load configuration
    let config = Config::load().wrap_err("Failed to load configuration")?;
    let import_config = ImportConfig::load().wrap_err("Failed to load configuration")?;

    // Database setup
    let manager = ConnectionManager::<SqliteConnection>::new(&config.database.url);
    let pool = Pool::builder()
        .build(manager)
        .wrap_err("Failed to create connection pool")?;

    let repo = RuleRepository::new(pool);

    // Create rule set
    let rule_set_id = Uuid::now_v7().to_string();
    let rule_set = NewRuleSet {
        id: rule_set_id.clone(),
        name: import_config.rule_set_name.clone(),
        slug: import_config.rule_set_slug.clone(),
        description: Some("Official WFDF Ultimate rules".to_string()),
    };

    info!("Creating rule set...");
    repo.create_rule_set(rule_set)?;

    // Create version
    let version_id = Uuid::now_v7().to_string();
    let effective_from =
        chrono::NaiveDate::parse_from_str(&import_config.version_effective_date, "%Y-%m-%d")
            .wrap_err("Invalid version_effective_date format in config (expected YYYY-MM-DD)")?;
    let version = NewVersion {
        id: version_id.clone(),
        rule_set_id: rule_set_id.clone(),
        version_name: import_config.version_name.clone(),
        effective_from,
        effective_to: None,
        description: Some("WFDF Ultimate rules 2025-2028".to_string()),
        is_current: true,
    };

    info!("Creating version...");
    repo.create_version(version)?;

    // Build number -> slug mapping for reference processing
    let number_to_slug: HashMap<String, String> = rule_data
        .iter()
        .map(|rule| (rule.number.clone(), rule.slug.clone()))
        .collect();

    // Sort rules by number to ensure parents are created before children
    let mut sorted_rules: Vec<RuleData> = rule_data
        .into_iter()
        .map(|mut rule| {
            // Process rule content to replace number references with {{slug}} templates
            let (processed_content, broken_refs) =
                process_number_references(&rule.content, &number_to_slug);

            if !broken_refs.is_empty() {
                warn!(
                    "Rule {} contains potential broken references: {:?}",
                    rule.number, broken_refs
                );
            }

            rule.content = processed_content;
            rule
        })
        .collect();

    // Sort by rule number hierarchy
    sorted_rules.sort_by(|a, b| {
        let a_parts = parse_rule_number(&a.number);
        let b_parts = parse_rule_number(&b.number);
        a_parts.cmp(&b_parts)
    });

    // Track rule IDs by their number for parent relationships
    let mut rule_ids: HashMap<String, String> = HashMap::new();

    info!("Creating {} rules...", sorted_rules.len());

    // Create rules
    for rule_data in sorted_rules {
        let rule_id = Uuid::now_v7().to_string();

        // Find parent rule ID
        let parent_rule_id = find_parent_rule(&rule_ids, &rule_data.number);

        let rule = NewRule {
            id: rule_id.clone(),
            slug: rule_data.slug.clone(),
            rule_set_id: rule_set_id.clone(),
            version_id: version_id.clone(),
            parent_rule_id,
            number: rule_data.number.clone(),
        };

        info!(
            "Creating rule {} ({}): {}",
            rule_data.number, rule_data.slug, rule_data.content
        );
        repo.create_rule(rule)?;

        // Store rule ID for parent lookup
        rule_ids.insert(rule_data.number.clone(), rule_id.clone());

        // Create rule content
        let content_id = Uuid::now_v7().to_string();
        let content = NewRuleContent {
            id: content_id,
            rule_id: rule_id.clone(),
            language: "en".to_string(),
            content_markdown: rule_data.content,
            source_content_id: None,
        };

        repo.create_rule_content(content)?;
    }

    info!("Import completed successfully!");
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rules = read_rules_from_stdin()?;

    if rules.is_empty() {
        warn!("No rules provided");
        return Ok(());
    }

    import_rules(rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_number_references() {
        let mut number_to_slug = HashMap::new();
        number_to_slug.insert("16.3".to_string(), "handling-contested-calls".to_string());
        number_to_slug.insert("1".to_string(), "spirit-of-the-game".to_string());
        number_to_slug.insert(
            "11.8".to_string(),
            "observers-and-rules-advisors".to_string(),
        );

        let content = "If the opposition does not gain possession, apply 16.3 according to Section 1 and Section 11.8.";
        let (processed, broken_refs) = process_number_references(content, &number_to_slug);

        let expected = "If the opposition does not gain possession, apply [16.3](rule:handling-contested-calls) according to [Section 1](rule:spirit-of-the-game) and [Section 11.8](rule:observers-and-rules-advisors).";
        assert_eq!(processed, expected);
        assert!(broken_refs.is_empty());
    }

    #[test]
    fn test_process_number_references_ignores_parenthetical() {
        let mut number_to_slug = HashMap::new();
        number_to_slug.insert("16.3".to_string(), "handling-contested-calls".to_string());

        let content = "Add two (2) seconds to the stall count. Apply 16.3 if needed. This results in ten (10) seconds.";
        let (processed, broken_refs) = process_number_references(content, &number_to_slug);

        let expected = "Add two (2) seconds to the stall count. Apply [16.3](rule:handling-contested-calls) if needed. This results in ten (10) seconds.";
        assert_eq!(processed, expected);
        assert!(broken_refs.is_empty());
    }

    #[test]
    fn test_process_number_references_with_broken_refs() {
        let mut number_to_slug = HashMap::new();
        number_to_slug.insert("16.3".to_string(), "handling-contested-calls".to_string());

        let content = "Apply 16.3 and also 99.9 here.";
        let (processed, broken_refs) = process_number_references(content, &number_to_slug);

        let expected = "Apply [16.3](rule:handling-contested-calls) and also 99.9 here.";
        assert_eq!(processed, expected);
        assert_eq!(broken_refs, vec!["99.9"]);
    }
}
