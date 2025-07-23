use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use eyre::{Result, WrapErr};
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead};
use uuid::Uuid;

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
    // Database setup
    // TODO: Pass database URL as CLI parameter
    let database_url = "db/regelator.db";
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .wrap_err("Failed to create connection pool")?;

    let repo = RuleRepository::new(pool);

    // Create rule set
    let rule_set_id = Uuid::now_v7().to_string();
    // TODO: Pass rule set name and slug as CLI parameters
    let rule_set = NewRuleSet {
        id: rule_set_id.clone(),
        name: "WFDF Ultimate rules".to_string(),
        slug: "wfdf-ultimate".to_string(),
        description: Some("Official WFDF Ultimate rules".to_string()),
    };

    println!("Creating rule set...");
    repo.create_rule_set(rule_set)?;

    // Create version
    let version_id = Uuid::now_v7().to_string();
    // TODO: Pass version name and effective dates as CLI parameters
    let version = NewVersion {
        id: version_id.clone(),
        rule_set_id: rule_set_id.clone(),
        version_name: "2025-2028".to_string(),
        effective_from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        effective_to: None,
        description: Some("WFDF Ultimate rules 2025-2028".to_string()),
        is_current: true,
    };

    println!("Creating version...");
    repo.create_version(version)?;

    // Track rule IDs by their number for parent relationships
    let mut rule_ids: HashMap<String, String> = HashMap::new();

    // Sort rules by number to ensure parents are created before children
    let mut sorted_rules = rule_data;
    sorted_rules.sort_by(|a, b| {
        let a_parts = parse_rule_number(&a.number);
        let b_parts = parse_rule_number(&b.number);
        a_parts.cmp(&b_parts)
    });

    println!("Creating {} rules...", sorted_rules.len());

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

        println!("Creating rule {} ({}): {}", rule_data.number, rule_data.slug, rule_data.content);
        repo.create_rule(rule)?;

        // Store rule ID for parent lookup
        rule_ids.insert(rule_data.number.clone(), rule_id.clone());

        // Create rule content
        let content_id = Uuid::now_v7().to_string();
        let content = NewRuleContent {
            id: content_id,
            rule_id: rule_id.clone(),
            language: "en".to_string(),
            title: None,
            content_markdown: rule_data.content,
            source_content_id: None,
        };

        repo.create_rule_content(content)?;
    }

    println!("Import completed successfully!");
    Ok(())
}

fn main() -> Result<()> {
    let rules = read_rules_from_stdin()?;

    if rules.is_empty() {
        println!("No rules provided");
        return Ok(());
    }

    import_rules(rules)
}
