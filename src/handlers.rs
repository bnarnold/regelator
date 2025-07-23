use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{models::Rule, repository::RuleRepository, AppError};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct VersionQuery {
    version: Option<String>,
}

#[derive(Serialize)]
struct RuleSetContext {
    language: String,
    rule_sets: Vec<RuleSetData>,
}

#[derive(Serialize)]
struct RuleSetData {
    name: String,
    slug: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct RulesListContext {
    language: String,
    rule_set_slug: String,
    rule_set_name: String,
    version_name: String,
    rule_tree: Vec<RuleNode>,
}

#[derive(Serialize, Debug)]
struct RuleNode {
    number: String,
    slug: String,
    title: String,
    children: Vec<RuleNode>,
}

#[derive(Serialize)]
struct RuleData {
    number: String,
    slug: String,
    title: String,
}

#[derive(Serialize)]
struct RuleDetailContext {
    language: String,
    rule_set_slug: String,
    rule: RuleDetailData,
    child_rules: Vec<RuleData>,
}

#[derive(Serialize)]
struct RuleDetailData {
    number: String,
    slug: String,
    title: String,
    content_markdown: String,
}

/// GET /en/rules - List all rule sets
pub async fn list_rule_sets(
    Path(language): Path<String>,
    State(templates): State<Arc<Environment<'static>>>,
    State(repo): State<RuleRepository>,
) -> Result<Html<String>, AppError> {
    let rule_sets = repo.get_rule_sets()?;

    let context = RuleSetContext {
        language: language.clone(),
        rule_sets: rule_sets
            .into_iter()
            .map(|rs| RuleSetData {
                name: rs.name,
                slug: rs.slug,
                description: rs.description,
            })
            .collect(),
    };

    let tmpl = templates.get_template("rule_sets.html")?;
    let rendered = tmpl.render(context)?;

    Ok(Html(rendered))
}

/// GET /en/rules/indoor - List rules for a rule set
pub async fn list_rules(
    Path((language, rule_set_slug)): Path<(String, String)>,
    Query(query): Query<VersionQuery>,
    State(templates): State<Arc<Environment<'static>>>,
    State(repo): State<RuleRepository>,
) -> Result<Html<String>, AppError> {
    // Get the version (current if not specified)
    let version = if let Some(version_name) = query.version {
        // Get version by name
        match repo.get_version_by_name(&rule_set_slug, &version_name)? {
            Some(v) => v,
            None => return Err(AppError(eyre::eyre!("Version not found"))),
        }
    } else {
        // Get current version
        match repo.get_current_version(&rule_set_slug)? {
            Some(v) => v,
            None => return Err(AppError(eyre::eyre!("Rule set not found"))),
        }
    };

    // Get all rules for this version
    let rules = repo.get_rules_for_version(&version.id)?;

    // Build hierarchical tree structure
    let rule_tree = build_rule_tree(rules);

    let context = RulesListContext {
        language: language.clone(),
        rule_set_slug: rule_set_slug.clone(),
        rule_set_name: rule_set_slug.clone(), // TODO: Get actual name
        version_name: version.version_name,
        rule_tree,
    };

    let tmpl = templates.get_template("rules_list.html")?;
    let rendered = tmpl.render(context)?;

    Ok(Html(rendered))
}

/// GET /en/rules/indoor/spirit-respectful-language - Show specific rule
pub async fn show_rule(
    Path((language, rule_set_slug, rule_slug)): Path<(String, String, String)>,
    Query(query): Query<VersionQuery>,
    State(templates): State<Arc<Environment<'static>>>,
    State(repo): State<RuleRepository>,
) -> Result<Html<String>, AppError> {
    // Get the version (current if not specified)
    let version = if let Some(version_name) = query.version {
        // Get version by name
        match repo.get_version_by_name(&rule_set_slug, &version_name)? {
            Some(v) => v,
            None => return Err(AppError(eyre::eyre!("Version not found"))),
        }
    } else {
        // Get current version
        match repo.get_current_version(&rule_set_slug)? {
            Some(v) => v,
            None => return Err(AppError(eyre::eyre!("Rule set not found"))),
        }
    };

    // Get the rule
    let rule = match repo.get_rule_by_slug(&rule_slug, &version.id)? {
        Some(r) => r,
        None => return Err(AppError(eyre::eyre!("Rule not found"))),
    };

    // Get rule content in requested language
    let content = match repo.get_rule_content(&rule.id, &language)? {
        Some(c) => c,
        None => return Err(AppError(eyre::eyre!("Rule content not found"))),
    };

    // Get child rules if any
    let child_rules = repo.get_child_rules(&rule.id)?;

    let context = RuleDetailContext {
        language: language.clone(),
        rule_set_slug: rule_set_slug.clone(),
        rule: RuleDetailData {
            number: rule.number,
            slug: rule.slug,
            title: content.title.unwrap_or_else(|| "Untitled".to_string()),
            content_markdown: content.content_markdown,
        },
        child_rules: child_rules
            .into_iter()
            .map(|r| RuleData {
                number: r.number,
                slug: r.slug.clone(),
                title: r.slug.replace('-', " "), // TODO: Get actual title
            })
            .collect(),
    };

    let tmpl = templates.get_template("rule_detail.html")?;
    let rendered = tmpl.render(context)?;

    Ok(Html(rendered))
}

/// Recursively sort rule nodes by number (numeric comparison)
fn sort_rule_nodes_recursively(nodes: &mut Vec<RuleNode>) {
    // Sort current level
    nodes.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.number.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.number.split('.').filter_map(|s| s.parse().ok()).collect();
        a_parts.cmp(&b_parts)
    });

    // Recursively sort children
    for node in nodes.iter_mut() {
        sort_rule_nodes_recursively(&mut node.children);
    }
}

/// Build hierarchical tree structure from flat rule list
fn build_rule_tree(rules: Vec<Rule>) -> Vec<RuleNode> {
    let mut nodes: HashMap<String, RuleNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();

    // Create all nodes and build children mapping
    for rule in &rules {
        let node = RuleNode {
            number: rule.number.clone(),
            slug: rule.slug.clone(),
            title: rule.slug.replace('-', " "), // TODO: Get actual title from content
            children: Vec::new(),
        };
        nodes.insert(rule.id.clone(), node);

        if let Some(parent_id) = &rule.parent_rule_id {
            children_map
                .entry(parent_id.clone())
                .or_default()
                .push(rule.id.clone());
        } else {
            root_ids.push(rule.id.clone());
        }
    }

    // Recursive function to build tree with children
    fn build_node(
        node_id: &str,
        nodes: &mut HashMap<String, RuleNode>,
        children_map: &HashMap<String, Vec<String>>,
    ) -> Option<RuleNode> {
        if let Some(mut node) = nodes.remove(node_id) {
            if let Some(child_ids) = children_map.get(node_id) {
                for child_id in child_ids {
                    if let Some(child_node) = build_node(child_id, nodes, children_map) {
                        node.children.push(child_node);
                    }
                }
            }
            Some(node)
        } else {
            None
        }
    }

    // Build root nodes
    let mut root_nodes: Vec<RuleNode> = root_ids
        .into_iter()
        .filter_map(|id| build_node(&id, &mut nodes, &children_map))
        .collect();

    // Sort all nodes recursively by rule number (numeric comparison)
    sort_rule_nodes_recursively(&mut root_nodes);

    root_nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rule(id: &str, number: &str, slug: &str, parent_id: Option<String>) -> Rule {
        Rule {
            id: id.to_string(),
            slug: slug.to_string(),
            rule_set_id: "test_rule_set".to_string(),
            version_id: "test_version".to_string(),
            parent_rule_id: parent_id,
            number: number.to_string(),
            created_at: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    #[test]
    fn test_rule_tree_sorting_root_level() {
        let rules = vec![
            create_test_rule("rule_10", "10", "rule-10", None),
            create_test_rule("rule_2", "2", "rule-2", None),
            create_test_rule("rule_1", "1", "rule-1", None),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[1].number, "2");
        assert_eq!(tree[2].number, "10");
    }

    #[test]
    fn test_rule_tree_sorting_hierarchical() {
        let rules = vec![
            create_test_rule("rule_1", "1", "rule-1", None),
            create_test_rule("rule_1_10", "1.10", "rule-1-10", Some("rule_1".to_string())),
            create_test_rule("rule_1_2", "1.2", "rule-1-2", Some("rule_1".to_string())),
            create_test_rule("rule_1_1", "1.1", "rule-1-1", Some("rule_1".to_string())),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].children.len(), 3);

        // Check children are sorted correctly
        assert_eq!(tree[0].children[0].number, "1.1");
        assert_eq!(tree[0].children[1].number, "1.2");
        assert_eq!(tree[0].children[2].number, "1.10");
    }

    #[test]
    fn test_rule_tree_sorting_deep_hierarchy() {
        let rules = vec![
            create_test_rule("rule_1", "1", "rule-1", None),
            create_test_rule("rule_1_2", "1.2", "rule-1-2", Some("rule_1".to_string())),
            create_test_rule(
                "rule_1_2_10",
                "1.2.10",
                "rule-1-2-10",
                Some("rule_1_2".to_string()),
            ),
            create_test_rule(
                "rule_1_2_2",
                "1.2.2",
                "rule-1-2-2",
                Some("rule_1_2".to_string()),
            ),
            create_test_rule(
                "rule_1_2_1",
                "1.2.1",
                "rule-1-2-1",
                Some("rule_1_2".to_string()),
            ),
        ];

        let tree = build_rule_tree(rules);

        println!("Tree: {:#?}", tree);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].number, "1.2");
        assert_eq!(tree[0].children[0].children.len(), 3);

        // Check deep children are sorted correctly
        let deep_children = &tree[0].children[0].children;
        assert_eq!(deep_children[0].number, "1.2.1");
        assert_eq!(deep_children[1].number, "1.2.2");
        assert_eq!(deep_children[2].number, "1.2.10");
    }

    #[test]
    fn test_rule_tree_mixed_hierarchy() {
        let rules = vec![
            create_test_rule("rule_10", "10", "rule-10", None),
            create_test_rule("rule_2", "2", "rule-2", None),
            create_test_rule("rule_2_10", "2.10", "rule-2-10", Some("rule_2".to_string())),
            create_test_rule("rule_2_1", "2.1", "rule-2-1", Some("rule_2".to_string())),
            create_test_rule("rule_1", "1", "rule-1", None),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 3);

        // Check root level sorting
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[1].number, "2");
        assert_eq!(tree[2].number, "10");

        // Check rule 2 has sorted children
        assert_eq!(tree[1].children.len(), 2);
        assert_eq!(tree[1].children[0].number, "2.1");
        assert_eq!(tree[1].children[1].number, "2.10");
    }

    #[test]
    fn test_sort_rule_nodes_recursively() {
        let mut nodes = vec![
            RuleNode {
                number: "10".to_string(),
                slug: "rule-10".to_string(),
                title: "Rule 10".to_string(),
                children: vec![
                    RuleNode {
                        number: "10.10".to_string(),
                        slug: "rule-10-10".to_string(),
                        title: "Rule 10.10".to_string(),
                        children: vec![],
                    },
                    RuleNode {
                        number: "10.2".to_string(),
                        slug: "rule-10-2".to_string(),
                        title: "Rule 10.2".to_string(),
                        children: vec![],
                    },
                ],
            },
            RuleNode {
                number: "2".to_string(),
                slug: "rule-2".to_string(),
                title: "Rule 2".to_string(),
                children: vec![],
            },
        ];

        sort_rule_nodes_recursively(&mut nodes);

        assert_eq!(nodes[0].number, "2");
        assert_eq!(nodes[1].number, "10");
        assert_eq!(nodes[1].children[0].number, "10.2");
        assert_eq!(nodes[1].children[1].number, "10.10");
    }
}
