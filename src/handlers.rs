use axum::{
    extract::{Path, Query, State},
    response::Html,
};
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use pulldown_cmark::{html, Parser};
use ammonia::clean;

use crate::{models::{Rule, RuleContent}, repository::RuleRepository, AppError};
use std::collections::HashMap;

/// Convert markdown text to safe HTML
fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    clean(&html_output).to_string()
}

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

#[derive(Serialize, Debug, Clone)]
struct RuleNode {
    number: String,
    slug: String,
    content: String,
    content_html: String,
    children: Vec<RuleNode>,
}

#[derive(Serialize)]
struct RuleData {
    number: String,
    slug: String,
    content: String,
}

#[derive(Serialize)]
struct RuleDetailContext {
    language: String,
    rule_set_slug: String,
    rule: RuleDetailData,
    parent_rule: Option<RuleDetailData>,
    child_rules: Vec<RuleNode>,
}

#[derive(Serialize)]
struct RuleDetailData {
    number: String,
    slug: String,
    content_markdown: String,
    content_html: String,
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

    // Get all rules with content for this version
    let rules_with_content = repo.get_rules_with_content_for_version(&version.id, &language)?;

    // Build hierarchical tree structure
    let rule_tree = build_rule_tree(rules_with_content);

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

    // Get parent rule if it exists
    let parent_rule = if let Some(parent_id) = &rule.parent_rule_id {
        let parent = repo.get_rule_by_id(parent_id)?;
        if let Some(parent) = parent {
            let parent_content = repo.get_rule_content(&parent.id, &language)?;
            if let Some(parent_content) = parent_content {
                Some(RuleDetailData {
                    number: parent.number,
                    slug: parent.slug,
                    content_markdown: parent_content.content_markdown.clone(),
                    content_html: markdown_to_html(&parent_content.content_markdown),
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Get all rules with content for this version and build the full tree
    let all_rules_with_content = repo.get_rules_with_content_for_version(&version.id, &language)?;
    let full_tree = build_rule_tree(all_rules_with_content);
    
    // Find the current rule in the tree and get its children
    let child_rules = find_rule_in_tree(&full_tree, &rule.slug)
        .map(|node| node.children.clone())
        .unwrap_or_else(Vec::new);

    let context = RuleDetailContext {
        language: language.clone(),
        rule_set_slug: rule_set_slug.clone(),
        rule: RuleDetailData {
            number: rule.number,
            slug: rule.slug,
            content_markdown: content.content_markdown.clone(),
            content_html: markdown_to_html(&content.content_markdown),
        },
        parent_rule,
        child_rules,
    };

    let tmpl = templates.get_template("rule_detail.html")?;
    let rendered = tmpl.render(context)?;

    Ok(Html(rendered))
}

/// Find a rule node in the tree by slug (recursive search)
fn find_rule_in_tree<'a>(nodes: &'a [RuleNode], target_slug: &str) -> Option<&'a RuleNode> {
    for node in nodes {
        if node.slug == target_slug {
            return Some(node);
        }
        if let Some(found) = find_rule_in_tree(&node.children, target_slug) {
            return Some(found);
        }
    }
    None
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

/// Build hierarchical tree structure from rules with content
fn build_rule_tree(rules_with_content: Vec<(Rule, RuleContent)>) -> Vec<RuleNode> {
    let mut nodes: HashMap<String, RuleNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();

    // Create all nodes and build children mapping
    for (rule, content) in &rules_with_content {
        let node = RuleNode {
            number: rule.number.clone(),
            slug: rule.slug.clone(),
            content: content.content_markdown.clone(),
            content_html: markdown_to_html(&content.content_markdown),
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

    fn create_test_rule_with_content(id: &str, number: &str, slug: &str, content: &str, parent_id: Option<String>) -> (Rule, RuleContent) {
        let rule = Rule {
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
        };
        
        let rule_content = RuleContent {
            id: format!("{}_content", id),
            rule_id: id.to_string(),
            language: "en".to_string(),
            title: None,
            content_markdown: content.to_string(),
            source_content_id: None,
            created_at: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        };
        
        (rule, rule_content)
    }

    #[test]
    fn test_rule_tree_sorting_root_level() {
        let rules = vec![
            create_test_rule_with_content("rule_10", "10", "rule-10", "Rule 10 content", None),
            create_test_rule_with_content("rule_2", "2", "rule-2", "Rule 2 content", None),
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].content, "Rule 1 content");
        assert_eq!(tree[0].content_html, "<p>Rule 1 content</p>\n");
        assert_eq!(tree[1].number, "2");
        assert_eq!(tree[1].content, "Rule 2 content");
        assert_eq!(tree[1].content_html, "<p>Rule 2 content</p>\n");
        assert_eq!(tree[2].number, "10");
        assert_eq!(tree[2].content, "Rule 10 content");
        assert_eq!(tree[2].content_html, "<p>Rule 10 content</p>\n");
    }

    #[test]
    fn test_rule_tree_sorting_hierarchical() {
        let rules = vec![
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
            create_test_rule_with_content("rule_1_10", "1.10", "rule-1-10", "Rule 1.10 content", Some("rule_1".to_string())),
            create_test_rule_with_content("rule_1_2", "1.2", "rule-1-2", "Rule 1.2 content", Some("rule_1".to_string())),
            create_test_rule_with_content("rule_1_1", "1.1", "rule-1-1", "Rule 1.1 content", Some("rule_1".to_string())),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].content, "Rule 1 content");
        assert_eq!(tree[0].children.len(), 3);

        // Check children are sorted correctly
        assert_eq!(tree[0].children[0].number, "1.1");
        assert_eq!(tree[0].children[0].content, "Rule 1.1 content");
        assert_eq!(tree[0].children[1].number, "1.2");
        assert_eq!(tree[0].children[1].content, "Rule 1.2 content");
        assert_eq!(tree[0].children[2].number, "1.10");
        assert_eq!(tree[0].children[2].content, "Rule 1.10 content");
    }

    #[test]
    fn test_rule_tree_sorting_deep_hierarchy() {
        let rules = vec![
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
            create_test_rule_with_content("rule_1_2", "1.2", "rule-1-2", "Rule 1.2 content", Some("rule_1".to_string())),
            create_test_rule_with_content("rule_1_2_10", "1.2.10", "rule-1-2-10", "Rule 1.2.10 content", Some("rule_1_2".to_string())),
            create_test_rule_with_content("rule_1_2_2", "1.2.2", "rule-1-2-2", "Rule 1.2.2 content", Some("rule_1_2".to_string())),
            create_test_rule_with_content("rule_1_2_1", "1.2.1", "rule-1-2-1", "Rule 1.2.1 content", Some("rule_1_2".to_string())),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].number, "1.2");
        assert_eq!(tree[0].children[0].children.len(), 3);

        // Check deep children are sorted correctly
        let deep_children = &tree[0].children[0].children;
        assert_eq!(deep_children[0].number, "1.2.1");
        assert_eq!(deep_children[0].content, "Rule 1.2.1 content");
        assert_eq!(deep_children[1].number, "1.2.2");
        assert_eq!(deep_children[1].content, "Rule 1.2.2 content");
        assert_eq!(deep_children[2].number, "1.2.10");
        assert_eq!(deep_children[2].content, "Rule 1.2.10 content");
    }

    #[test]
    fn test_rule_tree_mixed_hierarchy() {
        let rules = vec![
            create_test_rule_with_content("rule_10", "10", "rule-10", "Rule 10 content", None),
            create_test_rule_with_content("rule_2", "2", "rule-2", "Rule 2 content", None),
            create_test_rule_with_content("rule_2_10", "2.10", "rule-2-10", "Rule 2.10 content", Some("rule_2".to_string())),
            create_test_rule_with_content("rule_2_1", "2.1", "rule-2-1", "Rule 2.1 content", Some("rule_2".to_string())),
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
        ];

        let tree = build_rule_tree(rules);

        assert_eq!(tree.len(), 3);

        // Check root level sorting
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].content, "Rule 1 content");
        assert_eq!(tree[1].number, "2");
        assert_eq!(tree[1].content, "Rule 2 content");
        assert_eq!(tree[2].number, "10");
        assert_eq!(tree[2].content, "Rule 10 content");

        // Check rule 2 has sorted children
        assert_eq!(tree[1].children.len(), 2);
        assert_eq!(tree[1].children[0].number, "2.1");
        assert_eq!(tree[1].children[0].content, "Rule 2.1 content");
        assert_eq!(tree[1].children[1].number, "2.10");
        assert_eq!(tree[1].children[1].content, "Rule 2.10 content");
    }

    #[test]
    fn test_sort_rule_nodes_recursively() {
        let mut nodes = vec![
            RuleNode {
                number: "10".to_string(),
                slug: "rule-10".to_string(),
                content: "Rule 10 content".to_string(),
                content_html: "<p>Rule 10 content</p>\n".to_string(),
                children: vec![
                    RuleNode {
                        number: "10.10".to_string(),
                        slug: "rule-10-10".to_string(),
                        content: "Rule 10.10 content".to_string(),
                        content_html: "<p>Rule 10.10 content</p>\n".to_string(),
                        children: vec![],
                    },
                    RuleNode {
                        number: "10.2".to_string(),
                        slug: "rule-10-2".to_string(),
                        content: "Rule 10.2 content".to_string(),
                        content_html: "<p>Rule 10.2 content</p>\n".to_string(),
                        children: vec![],
                    },
                ],
            },
            RuleNode {
                number: "2".to_string(),
                slug: "rule-2".to_string(),
                content: "Rule 2 content".to_string(),
                content_html: "<p>Rule 2 content</p>\n".to_string(),
                children: vec![],
            },
        ];

        sort_rule_nodes_recursively(&mut nodes);

        assert_eq!(nodes[0].number, "2");
        assert_eq!(nodes[0].content, "Rule 2 content");
        assert_eq!(nodes[1].number, "10");
        assert_eq!(nodes[1].content, "Rule 10 content");
        assert_eq!(nodes[1].children[0].number, "10.2");
        assert_eq!(nodes[1].children[0].content, "Rule 10.2 content");
        assert_eq!(nodes[1].children[1].number, "10.10");
        assert_eq!(nodes[1].children[1].content, "Rule 10.10 content");
    }

    #[test]
    fn test_markdown_to_html_basic_features() {
        // Test headers
        assert_eq!(markdown_to_html("# Header 1"), "<h1>Header 1</h1>\n");
        assert_eq!(markdown_to_html("## Header 2"), "<h2>Header 2</h2>\n");
        
        // Test emphasis
        assert_eq!(markdown_to_html("*italic*"), "<p><em>italic</em></p>\n");
        assert_eq!(markdown_to_html("**bold**"), "<p><strong>bold</strong></p>\n");
        
        // Test links (ammonia adds rel attributes for security)
        assert_eq!(
            markdown_to_html("[link text](https://example.com)"),
            "<p><a href=\"https://example.com\" rel=\"noopener noreferrer\">link text</a></p>\n"
        );
        
        // Test lists
        assert_eq!(
            markdown_to_html("- Item 1\n- Item 2"),
            "<ul>\n<li>Item 1</li>\n<li>Item 2</li>\n</ul>\n"
        );
        
        // Test ordered lists
        assert_eq!(
            markdown_to_html("1. First\n2. Second"),
            "<ol>\n<li>First</li>\n<li>Second</li>\n</ol>\n"
        );
    }

    #[test]
    fn test_markdown_to_html_xss_protection() {
        // Test that script tags are completely removed by ammonia
        assert_eq!(
            markdown_to_html("<script>alert('xss')</script>"),
            ""
        );
        
        // Test that raw HTML is sanitized by ammonia (script tags removed)
        assert_eq!(
            markdown_to_html("Plain text with <script>"),
            "<p>Plain text with </p>"
        );
        
        // Test that dangerous links are sanitized (href removed, rel added)
        assert_eq!(
            markdown_to_html("[dangerous](javascript:alert('xss'))"),
            "<p><a rel=\"noopener noreferrer\">dangerous</a></p>\n"
        );
    }

    #[test]
    fn test_markdown_to_html_safe_html() {
        // Test that basic HTML tags are preserved when safe
        assert_eq!(
            markdown_to_html("Normal **bold** text"),
            "<p>Normal <strong>bold</strong> text</p>\n"
        );
        
        // Test paragraphs
        assert_eq!(
            markdown_to_html("First paragraph\n\nSecond paragraph"),
            "<p>First paragraph</p>\n<p>Second paragraph</p>\n"
        );
    }
}
