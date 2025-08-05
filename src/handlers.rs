use crate::{
    models::{NewQuizAttempt, Rule, RuleContent},
    repository::RuleRepository,
    AppError,
};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, Redirect},
};
use axum_extra::extract::CookieJar;
use minijinja::Environment;
use rand::seq::IndexedRandom;
use regelator::auth::{
    clear_admin_cookie, create_admin_cookie, verify_admin_cookie, ADMIN_COOKIE_NAME,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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

    // Get rule set info to build definition slug mapping
    let rule_sets = repo.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| eyre::eyre!("Rule set '{}' not found", rule_set_slug))?;

    // Build hierarchical tree structure
    let rule_tree = build_rule_tree(rules_with_content, &language, &rule_set_slug);

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

    // Get all rules with content for this version and build the full tree
    let all_rules_with_content = repo.get_rules_with_content_for_version(&version.id, &language)?;

    // Build slug -> number mapping for processing rule content
    let slug_to_number: HashMap<String, String> = all_rules_with_content
        .iter()
        .map(|(rule, _)| (rule.slug.clone(), rule.number.clone()))
        .collect();

    // Get rule set info to build definition slug mapping
    let rule_sets = repo.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| eyre::eyre!("Rule set '{}' not found", rule_set_slug))?;

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

    let full_tree = build_rule_tree(all_rules_with_content, &language, &rule_set_slug);

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
fn build_rule_tree(
    rules_with_content: Vec<(Rule, RuleContent)>,
    language: &str,
    rule_set_slug: &str,
) -> Vec<RuleNode> {
    let mut nodes: HashMap<String, RuleNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut root_ids: Vec<String> = Vec::new();

    // Build slug -> rule number mapping for reference processing
    let slug_to_number: HashMap<String, String> = rules_with_content
        .iter()
        .map(|(rule, _)| (rule.slug.clone(), rule.number.clone()))
        .collect();

    // Create all nodes and build children mapping
    for (rule, content) in &rules_with_content {
        let processed_content = content.content_markdown.clone();

        let node = RuleNode {
            number: rule.number.clone(),
            slug: rule.slug.clone(),
            content: processed_content,
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

    fn create_test_rule_with_content(
        id: &str,
        number: &str,
        slug: &str,
        content: &str,
        parent_id: Option<String>,
    ) -> (Rule, RuleContent) {
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

        let tree = build_rule_tree(rules, "en", "test-rules");

        assert_eq!(tree.len(), 3);
        assert_eq!(tree[0].number, "1");
        assert_eq!(tree[0].content, "Rule 1 content");
        assert_eq!(tree[1].number, "2");
        assert_eq!(tree[1].content, "Rule 2 content");
        assert_eq!(tree[2].number, "10");
        assert_eq!(tree[2].content, "Rule 10 content");
    }

    #[test]
    fn test_rule_tree_sorting_hierarchical() {
        let rules = vec![
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
            create_test_rule_with_content(
                "rule_1_10",
                "1.10",
                "rule-1-10",
                "Rule 1.10 content",
                Some("rule_1".to_string()),
            ),
            create_test_rule_with_content(
                "rule_1_2",
                "1.2",
                "rule-1-2",
                "Rule 1.2 content",
                Some("rule_1".to_string()),
            ),
            create_test_rule_with_content(
                "rule_1_1",
                "1.1",
                "rule-1-1",
                "Rule 1.1 content",
                Some("rule_1".to_string()),
            ),
        ];

        let tree = build_rule_tree(rules, "en", "test-rules");

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
            create_test_rule_with_content(
                "rule_1_2",
                "1.2",
                "rule-1-2",
                "Rule 1.2 content",
                Some("rule_1".to_string()),
            ),
            create_test_rule_with_content(
                "rule_1_2_10",
                "1.2.10",
                "rule-1-2-10",
                "Rule 1.2.10 content",
                Some("rule_1_2".to_string()),
            ),
            create_test_rule_with_content(
                "rule_1_2_2",
                "1.2.2",
                "rule-1-2-2",
                "Rule 1.2.2 content",
                Some("rule_1_2".to_string()),
            ),
            create_test_rule_with_content(
                "rule_1_2_1",
                "1.2.1",
                "rule-1-2-1",
                "Rule 1.2.1 content",
                Some("rule_1_2".to_string()),
            ),
        ];

        let tree = build_rule_tree(rules, "en", "test-rules");

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
            create_test_rule_with_content(
                "rule_2_10",
                "2.10",
                "rule-2-10",
                "Rule 2.10 content",
                Some("rule_2".to_string()),
            ),
            create_test_rule_with_content(
                "rule_2_1",
                "2.1",
                "rule-2-1",
                "Rule 2.1 content",
                Some("rule_2".to_string()),
            ),
            create_test_rule_with_content("rule_1", "1", "rule-1", "Rule 1 content", None),
        ];

        let tree = build_rule_tree(rules, "en", "test-rules");

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
                children: vec![
                    RuleNode {
                        number: "10.10".to_string(),
                        slug: "rule-10-10".to_string(),
                        content: "Rule 10.10 content".to_string(),
                        children: vec![],
                    },
                    RuleNode {
                        number: "10.2".to_string(),
                        slug: "rule-10-2".to_string(),
                        content: "Rule 10.2 content".to_string(),
                        children: vec![],
                    },
                ],
            },
            RuleNode {
                number: "2".to_string(),
                slug: "rule-2".to_string(),
                content: "Rule 2 content".to_string(),
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
}

/// Data structure for passing glossary terms to templates
#[derive(Serialize)]
struct DefinitionItem {
    term: String,
    slug: String,
    definition_html: String,
}

#[derive(Serialize)]
struct DefinitionsPageData {
    rule_set_name: String,
    rule_set_slug: String,
    version_name: String,
    definitions: Vec<DefinitionItem>,
}

/// Handler for displaying definitions/glossary page
pub async fn definitions_page(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
) -> Result<Html<String>, AppError> {
    // Get rule set info for display
    let rule_sets = repository.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| eyre::eyre!("Rule set '{}' not found", rule_set_slug))?;

    // Get the current version for this rule set
    let version = repository
        .get_current_version(&rule_set_slug)?
        .ok_or_else(|| eyre::eyre!("No current version found"))?;

    // Get all glossary terms for this rule set and version
    let glossary_terms = repository.get_glossary_terms(&rule_set.id, &version.id)?;

    // Convert to template data, sorting alphabetically by term
    let mut definitions: Vec<DefinitionItem> = glossary_terms
        .into_iter()
        .map(|(term, content)| DefinitionItem {
            term: content.term.clone(),
            slug: term.slug.clone(),
            definition_html: content.definition_markdown.clone(),
        })
        .collect();

    // Sort alphabetically by term (case-insensitive)
    definitions.sort_by(|a, b| a.term.to_lowercase().cmp(&b.term.to_lowercase()));

    let template_data = DefinitionsPageData {
        rule_set_name: rule_set.name.clone(),
        rule_set_slug: rule_set.slug.clone(),
        version_name: version.version_name.clone(),
        definitions,
    };

    let template = template_env.get_template("definitions.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

// Quiz handlers

#[derive(Serialize)]
struct QuizLandingData {
    language: String,
    rule_set_slug: String,
}

#[derive(Serialize)]
struct QuizQuestionData {
    question_id: String,
    question_text: String,
    difficulty_level: String,
    answers: Vec<QuizAnswerData>,
    session_id: String,
    rule_set_slug: String,
    language: String,
}

#[derive(Serialize)]
struct QuizAnswerData {
    id: String,
    answer_text: String,
}

#[derive(Serialize)]
struct QuizResultData {
    question_id: String,
    question_text: String,
    difficulty_level: String,
    answers: Vec<QuizAnswerWithResult>,
    selected_answer_id: String,
    is_correct: bool,
    explanation: String,
    session_id: String,
    session_stats: SessionStatsView,
    total_questions_available: usize,
    questions_attempted: usize,
    language: String,
    rule_set_slug: String,
}

#[derive(Serialize)]
struct QuizSessionCompleteData {
    session_id: String,
    stats: SessionStatsView,
    missed_questions: Vec<MissedQuestionView>,
}

#[derive(Serialize)]
struct SessionStatsView {
    total_questions: usize,
    correct_answers: usize,
    accuracy_percentage: u32,
    current_streak: usize,
}

#[derive(Serialize)]
struct MissedQuestionView {
    question_text: String,
    difficulty_level: String,
    explanation: String,
}

#[derive(Serialize)]
struct QuizAnswerWithResult {
    id: String,
    answer_text: String,
    is_correct: bool,
    was_selected: bool,
}

#[derive(Deserialize)]
pub struct QuizSubmission {
    question_id: String,
    answer_id: String,
    session_id: String,
    rule_set_slug: String,
    language: String,
}

/// Quiz landing page
pub async fn quiz_landing(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(template_env): State<Arc<Environment<'static>>>,
) -> Result<Html<String>, AppError> {
    let template_data = QuizLandingData {
        language,
        rule_set_slug,
    };

    let template = template_env.get_template("quiz_landing.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Start a new quiz session
pub async fn start_quiz_session(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
) -> Result<Html<String>, AppError> {
    let session_id = uuid::Uuid::now_v7().to_string();

    // Redirect to first question with this session
    get_quiz_question_for_session(
        repository,
        template_env,
        session_id,
        language,
        rule_set_slug,
    )
    .await
}

/// Get a random quiz question (for form redirects)
pub async fn random_quiz_question(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
    Form(form_data): Form<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    let session_id = form_data
        .get("session_id")
        .ok_or_else(|| AppError(eyre::eyre!("Session ID required")))?
        .clone();

    get_quiz_question_for_session(
        repository,
        template_env,
        session_id,
        language,
        rule_set_slug,
    )
    .await
}

/// Helper function to get quiz question for a session
async fn get_quiz_question_for_session(
    repository: RuleRepository,
    template_env: Arc<Environment<'static>>,
    session_id: String,
    language: String,
    rule_set_slug: String,
) -> Result<Html<String>, AppError> {
    let rule_sets = repository.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| AppError(eyre::eyre!("Rule set not found")))?;

    let version = repository
        .get_current_version(&rule_set_slug)?
        .ok_or_else(|| AppError(eyre::eyre!("No current version found")))?;

    // Get questions not yet attempted in this session
    let questions =
        repository.get_unattempted_questions_for_session(&session_id, &rule_set.id, &version.id)?;

    if questions.is_empty() {
        // All questions have been attempted - show session complete
        return show_session_complete(
            repository,
            template_env,
            session_id,
            language,
            rule_set_slug,
        )
        .await;
    }

    // Select random question (simple approach for now)
    let mut rng = rand::rng();
    let selected_question = questions
        .choose(&mut rng)
        .ok_or_else(|| AppError(eyre::eyre!("Failed to select random question")))?;

    // Get answers for this question
    let db_answers = repository.get_quiz_answers(&selected_question.id)?;

    // Convert to handler-specific structs
    let answers = db_answers
        .iter()
        .map(|a| QuizAnswerData {
            id: a.id.clone(),
            answer_text: a.answer_text.clone(),
        })
        .collect();

    let template_data = QuizQuestionData {
        question_id: selected_question.id.clone(),
        question_text: selected_question.question_text.clone(),
        difficulty_level: selected_question.difficulty_level.clone(),
        answers,
        session_id,
        rule_set_slug,
        language,
    };

    let template = template_env.get_template("quiz_question.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Show session complete page with statistics
async fn show_session_complete(
    repository: RuleRepository,
    template_env: Arc<Environment<'static>>,
    session_id: String,
    _language: String,
    _rule_set_slug: String,
) -> Result<Html<String>, AppError> {
    // Get session statistics
    let db_stats = repository.get_session_statistics(&session_id)?;
    let stats = SessionStatsView {
        total_questions: db_stats.total_questions,
        correct_answers: db_stats.correct_answers,
        accuracy_percentage: db_stats.accuracy_percentage,
        current_streak: db_stats.current_streak,
    };

    // Get missed questions for review and convert to view models
    let db_missed = repository.get_session_missed_questions(&session_id)?;
    let missed_questions: Vec<MissedQuestionView> = db_missed
        .into_iter()
        .map(|(question, _attempt)| MissedQuestionView {
            question_text: question.question_text,
            difficulty_level: question.difficulty_level,
            explanation: question.explanation,
        })
        .collect();

    let template_data = QuizSessionCompleteData {
        session_id,
        stats,
        missed_questions,
    };

    let template = template_env.get_template("quiz_session_complete.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Submit quiz answer and show results
pub async fn submit_quiz_answer(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
    Form(submission): Form<QuizSubmission>,
) -> Result<Html<String>, AppError> {
    // Get the question by ID
    let question = repository
        .get_quiz_question_by_id(&submission.question_id)?
        .ok_or_else(|| AppError(eyre::eyre!("Question not found")))?;

    let answers = repository.get_quiz_answers(&submission.question_id)?;

    // Find selected answer and check if correct
    let selected_answer = answers
        .iter()
        .find(|a| a.id == submission.answer_id)
        .ok_or_else(|| AppError(eyre::eyre!("Answer not found")))?;

    let is_correct = selected_answer.is_correct;

    // Record the attempt
    let attempt = NewQuizAttempt::new(
        submission.session_id.clone(),
        submission.question_id.clone(),
        Some(submission.answer_id.clone()),
        Some(is_correct),
        None, // No timing for now
    );

    repository.create_quiz_attempt(&attempt)?;

    // Get session statistics after recording the attempt
    let db_stats = repository.get_session_statistics(&submission.session_id)?;
    let session_stats = SessionStatsView {
        total_questions: db_stats.total_questions,
        correct_answers: db_stats.correct_answers,
        accuracy_percentage: db_stats.accuracy_percentage,
        current_streak: db_stats.current_streak,
    };

    // Get total available questions for this rule set using form data
    let rule_sets = repository.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == submission.rule_set_slug)
        .ok_or_else(|| AppError(eyre::eyre!("Rule set not found")))?;
    let version = repository
        .get_current_version(&submission.rule_set_slug)?
        .ok_or_else(|| AppError(eyre::eyre!("No current version found")))?;

    let all_questions = repository.get_quiz_questions(&rule_set.id, &version.id)?;
    let total_questions_available = all_questions.len();
    let questions_attempted = db_stats.total_questions;

    // Prepare answer data with selection markers
    let answers_with_result: Vec<QuizAnswerWithResult> = answers
        .iter()
        .map(|a| QuizAnswerWithResult {
            id: a.id.clone(),
            answer_text: a.answer_text.clone(),
            is_correct: a.is_correct,
            was_selected: a.id == submission.answer_id,
        })
        .collect();

    let template_data = QuizResultData {
        question_id: question.id.clone(),
        question_text: question.question_text.clone(),
        difficulty_level: question.difficulty_level.clone(),
        answers: answers_with_result,
        selected_answer_id: submission.answer_id,
        is_correct,
        explanation: question.explanation.clone(),
        session_id: submission.session_id,
        session_stats,
        total_questions_available,
        questions_attempted,
        language: submission.language,
        rule_set_slug: submission.rule_set_slug,
    };

    let template = template_env.get_template("quiz_result.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Clear session data for privacy
pub async fn clear_session_data(
    State(repository): State<RuleRepository>,
    Path(session_id): Path<String>,
) -> Result<Redirect, AppError> {
    // Delete all attempts for this session
    repository.clear_session_attempts(&session_id)?;

    // Redirect back to quiz home
    Ok(Redirect::to("/quiz"))
}

// Admin authentication structures
#[derive(Deserialize)]
pub struct AdminLoginForm {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AdminLoginContext {
    error: Option<String>,
}

#[derive(Serialize)]
struct AdminDashboardContext {
    username: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

#[derive(Serialize)]
struct ChangePasswordContext {
    error: Option<String>,
    success: Option<String>,
}

/// Show admin login form
pub async fn admin_login_form(
    State(templates): State<Arc<Environment<'static>>>,
) -> Result<Html<String>, AppError> {
    let context = AdminLoginContext { error: None };
    let tmpl = templates.get_template("admin_login.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Process admin login
pub async fn admin_login_submit(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    jar: CookieJar,
    Form(form_data): Form<AdminLoginForm>,
) -> Result<(CookieJar, Html<String>), AppError> {
    // Find admin by username
    let admin = match repository.find_admin_by_username(&form_data.username)? {
        Some(admin) => admin,
        None => {
            // Show login form with error
            let context = AdminLoginContext {
                error: Some("Invalid username or password".to_string()),
            };
            let tmpl = templates.get_template("admin_login.html")?;
            let rendered = tmpl.render(context)?;
            return Ok((jar, Html(rendered)));
        }
    };

    // Verify password using Argon2
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&admin.password_hash)
        .map_err(|e| AppError(eyre::eyre!("Failed to parse password hash: {}", e)))?;

    if argon2
        .verify_password(form_data.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        // Password is correct - update last login and create signed cookie
        repository.update_admin_last_login(&admin.id)?;

        let cookie = create_admin_cookie(admin.id, admin.username.clone())
            .map_err(|e| AppError(eyre::eyre!("Failed to create admin cookie: {}", e)))?;

        // Show admin dashboard
        let context = AdminDashboardContext {
            username: admin.username,
        };
        let tmpl = templates.get_template("admin_dashboard.html")?;
        let rendered = tmpl.render(context)?;
        Ok((jar.add(cookie), Html(rendered)))
    } else {
        // Show login form with error
        let context = AdminLoginContext {
            error: Some("Invalid username or password".to_string()),
        };
        let tmpl = templates.get_template("admin_login.html")?;
        let rendered = tmpl.render(context)?;
        Ok((jar, Html(rendered)))
    }
}

/// Show admin dashboard (protected route)
pub async fn admin_dashboard(
    State(templates): State<Arc<Environment<'static>>>,
    jar: CookieJar,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    let claims = verify_admin_cookie(cookie.value())
        .map_err(|e| AppError(eyre::eyre!("Invalid admin session: {}", e)))?;

    let context = AdminDashboardContext {
        username: claims.username,
    };
    let tmpl = templates.get_template("admin_dashboard.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Admin logout (clears cookie and redirects to login)
pub async fn admin_logout(jar: CookieJar) -> Result<(CookieJar, Redirect), AppError> {
    let clear_cookie = clear_admin_cookie();
    Ok((jar.add(clear_cookie), Redirect::to("/admin/login")))
}

/// Show password change form (protected route)
pub async fn admin_change_password_form(
    State(templates): State<Arc<Environment<'static>>>,
    jar: CookieJar,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    verify_admin_cookie(cookie.value())
        .map_err(|e| AppError(eyre::eyre!("Invalid admin session: {}", e)))?;

    let context = ChangePasswordContext {
        error: None,
        success: None,
    };
    let tmpl = templates.get_template("admin_change_password.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Process password change (protected route)
pub async fn admin_change_password_submit(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    jar: CookieJar,
    Form(form_data): Form<ChangePasswordForm>,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie and get admin info
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    let claims = verify_admin_cookie(cookie.value())
        .map_err(|e| AppError(eyre::eyre!("Invalid admin session: {}", e)))?;

    // Validate form data
    if form_data.new_password != form_data.confirm_password {
        let context = ChangePasswordContext {
            error: Some("New passwords do not match".to_string()),
            success: None,
        };
        let tmpl = templates.get_template("admin_change_password.html")?;
        let rendered = tmpl.render(context)?;
        return Ok(Html(rendered));
    }

    if form_data.new_password.len() < 8 {
        let context = ChangePasswordContext {
            error: Some("New password must be at least 8 characters".to_string()),
            success: None,
        };
        let tmpl = templates.get_template("admin_change_password.html")?;
        let rendered = tmpl.render(context)?;
        return Ok(Html(rendered));
    }

    // Get current admin record to verify current password
    let admin = repository
        .find_admin_by_username(&claims.username)?
        .ok_or_else(|| AppError(eyre::eyre!("Admin not found")))?;

    // Verify current password
    let argon2 = Argon2::default();
    let current_parsed_hash = PasswordHash::new(&admin.password_hash)
        .map_err(|e| AppError(eyre::eyre!("Failed to parse current password hash: {}", e)))?;

    if argon2
        .verify_password(form_data.current_password.as_bytes(), &current_parsed_hash)
        .is_err()
    {
        let context = ChangePasswordContext {
            error: Some("Current password is incorrect".to_string()),
            success: None,
        };
        let tmpl = templates.get_template("admin_change_password.html")?;
        let rendered = tmpl.render(context)?;
        return Ok(Html(rendered));
    }

    // Hash new password
    let salt = SaltString::generate(&mut OsRng);
    let new_password_hash = argon2
        .hash_password(form_data.new_password.as_bytes(), &salt)
        .map_err(|e| AppError(eyre::eyre!("Failed to hash new password: {}", e)))?
        .to_string();

    // Update password in database (with current hash verification for extra security)
    repository.update_admin_password(&admin.id, &admin.password_hash, &new_password_hash)?;

    // Show success message
    let context = ChangePasswordContext {
        error: None,
        success: Some("Password changed successfully".to_string()),
    };
    let tmpl = templates.get_template("admin_change_password.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}
