use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use eyre::{Result, WrapErr};
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead};

use regelator::config::Config;
use regelator::models::*;
use regelator::repository::RuleRepository;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
struct QuizQuestionImport {
    question_text: String,
    difficulty: String,
    answers: Vec<QuizAnswerImport>,
    explanation: String,
    rule_references: Vec<String>, // Rule numbers mentioned in REF lines
}

#[derive(Debug)]
struct QuizAnswerImport {
    text: String,
    is_correct: bool,
}

fn main() -> Result<()> {
    // Configuration constants
    // Load configuration
    let config = Config::load().wrap_err("Failed to load configuration")?;
    
    let rule_set_slug = &config.import.rule_set_slug;
    let version_name = &config.import.version_name;
    let language = "en";

    let manager = ConnectionManager::<SqliteConnection>::new(&config.database.url);
    let pool = Pool::builder()
        .build(manager)
        .wrap_err("Failed to create connection pool")?;

    let repository = RuleRepository::new(pool);

    // Read questions from stdin
    let stdin = io::stdin();
    let lines: Vec<String> = stdin.lock().lines().collect::<Result<_, _>>()?;

    let questions = parse_quiz_questions(&lines)?;
    println!("Parsed {} quiz questions", questions.len());
    
    let rule_sets = repository.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == *rule_set_slug)
        .ok_or_else(|| eyre::eyre!("Rule set '{}' not found", rule_set_slug))?;

    let version = repository.get_current_version(rule_set_slug)?
        .ok_or_else(|| eyre::eyre!("No current version found for rule set '{}'", rule_set_slug))?;

    // Get all rules to build number-to-slug and number-to-id mappings
    let all_rules_with_content = repository.get_rules_with_content_for_version(&version.id, language)?;
    
    let number_to_slug: HashMap<String, String> = all_rules_with_content
        .iter()
        .map(|(rule, _)| (rule.number.clone(), rule.slug.clone()))
        .collect();

    let number_to_id: HashMap<String, String> = all_rules_with_content
        .iter()
        .map(|(rule, _)| (rule.number.clone(), rule.id.clone()))
        .collect();

    // Process and import each question
    for question_import in questions {
        println!("Importing question: {}", &question_import.question_text[..50.min(question_import.question_text.len())]);
        
        // Process rule references in explanation
        let (processed_explanation, _broken_refs) = process_number_references(
            &question_import.explanation,
            &number_to_slug,
        );

        // Convert answers to business layer format
        let answers: Vec<QuizAnswerData> = question_import.answers
            .into_iter()
            .map(|a| QuizAnswerData {
                answer_text: a.text,
                is_correct: a.is_correct,
            })
            .collect();

        // Get rule IDs for the question based on rule references
        let rule_ids: Vec<String> = question_import.rule_references
            .iter()
            .filter_map(|rule_num| number_to_id.get(rule_num))
            .cloned()
            .collect();

        // Create business layer question data
        let question_data = QuizQuestionData::new(
            rule_set.id.clone(),
            version.id.clone(),
            question_import.question_text,
            processed_explanation,
            question_import.difficulty,
            answers,
            rule_ids,
        );

        // Import the complete question
        let created_question = repository.create_quiz_question_complete(&question_data)?;
        println!("Created question with ID: {}", created_question.id);
    }

    println!("Import completed successfully!");
    Ok(())
}

/// Parse quiz questions from input lines
fn parse_quiz_questions(lines: &[String]) -> Result<Vec<QuizQuestionImport>> {
    let mut questions = Vec::new();
    let mut current_question: Option<QuizQuestionImport> = None;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("Q: ") {
            // Save previous question if exists
            if let Some(question) = current_question.take() {
                questions.push(question);
            }

            // Parse question with difficulty
            let question_line = &line[3..]; // Remove "Q: "
            let (difficulty, question_text) = parse_question_with_difficulty(question_line)?;

            current_question = Some(QuizQuestionImport {
                question_text: question_text.to_string(),
                difficulty,
                answers: Vec::new(),
                explanation: String::new(),
                rule_references: Vec::new(),
            });
        } else if line.starts_with("REF: ") {
            // Parse rule references
            if let Some(ref mut question) = current_question {
                let refs = &line[5..]; // Remove "REF: "
                for rule_ref in refs.split(',') {
                    question.rule_references.push(rule_ref.trim().to_string());
                }
            }
        } else if line.starts_with("A: ") {
            // Parse answer
            if let Some(ref mut question) = current_question {
                let answer_line = &line[3..]; // Remove "A: "
                let (is_correct, answer_text) = parse_answer(answer_line);
                question.answers.push(QuizAnswerImport {
                    text: answer_text.to_string(),
                    is_correct,
                });
            }
        } else if line.starts_with("EXPLAIN: ") {
            // Parse explanation
            if let Some(ref mut question) = current_question {
                let mut explanation = line[9..].to_string(); // Remove "EXPLAIN: "
                
                // Check for multi-line explanations
                i += 1;
                while i < lines.len() && !lines[i].trim().is_empty() && !lines[i].starts_with("Q: ") {
                    explanation.push('\n');
                    explanation.push_str(lines[i].trim());
                    i += 1;
                }
                i -= 1; // Back up one since we'll increment at end of loop
                
                question.explanation = explanation;
            }
        }

        i += 1;
    }

    // Don't forget the last question
    if let Some(question) = current_question {
        questions.push(question);
    }

    Ok(questions)
}

/// Parse question line with difficulty level in brackets
fn parse_question_with_difficulty(line: &str) -> Result<(String, &str)> {
    let difficulty_regex = Regex::new(r"^\[(\w+)\]\s*(.+)$").unwrap();
    
    if let Some(captures) = difficulty_regex.captures(line) {
        let difficulty = captures.get(1).unwrap().as_str().to_lowercase();
        let question_text = captures.get(2).unwrap().as_str();
        
        // Validate difficulty level
        match difficulty.as_str() {
            "beginner" | "intermediate" | "advanced" => Ok((difficulty, question_text)),
            _ => Err(eyre::eyre!("Invalid difficulty level: {}. Must be beginner, intermediate, or advanced", difficulty)),
        }
    } else {
        Err(eyre::eyre!("Question must start with difficulty level in brackets: [BEGINNER], [INTERMEDIATE], or [ADVANCED]"))
    }
}

/// Parse answer line, checking for [CORRECT] flag
fn parse_answer(line: &str) -> (bool, &str) {
    if line.ends_with(" [CORRECT]") {
        (true, &line[..line.len() - 10]) // Remove " [CORRECT]"
    } else {
        (false, line)
    }
}

/// Process rule number references in content and replace with [text](rule:slug) links
/// Reuses the same logic as the rule import script
fn process_number_references(
    content: &str,
    number_to_slug: &HashMap<String, String>,
) -> (String, Vec<String>) {
    // Match rule references: numbers with dots OR numbers prefixed by "Section"
    let reference_pattern = Regex::new(r"\b(?:Section\s+(\d+(?:\.\d+)*)|(\d+\.\d+(?:\.\d+)*))\b").unwrap();
    let mut processed_content = content.to_string();
    let mut broken_references = Vec::new();
    
    // Find all rule reference patterns and collect replacements
    let mut replacements = Vec::new();
    
    for mat in reference_pattern.find_iter(content) {
        let full_match = mat.as_str();
        let match_start = mat.start();
        let match_end = mat.end();
        
        // Extract the rule number (handle both "Section X" and "X.Y" patterns)
        let rule_number = if full_match.starts_with("Section ") {
            &full_match[8..] // Remove "Section "
        } else {
            full_match
        };
        
        if let Some(slug) = number_to_slug.get(rule_number) {
            let replacement = if full_match.starts_with("Section ") {
                format!("[Section {}](rule:{})", rule_number, slug)
            } else {
                format!("[{}](rule:{})", rule_number, slug)
            };
            replacements.push((match_start, match_end, replacement));
        } else {
            broken_references.push(rule_number.to_string());
        }
    }
    
    // Apply replacements in reverse order to maintain correct positions
    replacements.reverse();
    for (start, end, replacement) in replacements {
        processed_content.replace_range(start..end, &replacement);
    }
    
    (processed_content, broken_references)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_question_with_difficulty() {
        let result = parse_question_with_difficulty("[BEGINNER] What is the objective of Ultimate?");
        assert!(result.is_ok());
        let (difficulty, question) = result.unwrap();
        assert_eq!(difficulty, "beginner");
        assert_eq!(question, "What is the objective of Ultimate?");
    }

    #[test]
    fn test_parse_answer_correct() {
        let (is_correct, text) = parse_answer("Score points by catching the disc in the end zone [CORRECT]");
        assert!(is_correct);
        assert_eq!(text, "Score points by catching the disc in the end zone");
    }

    #[test]
    fn test_parse_answer_incorrect() {
        let (is_correct, text) = parse_answer("Keep possession for as long as possible");
        assert!(!is_correct);
        assert_eq!(text, "Keep possession for as long as possible");
    }

    #[test]
    fn test_process_number_references() {
        let mut number_to_slug = HashMap::new();
        number_to_slug.insert("16.3".to_string(), "handling-contested-calls".to_string());
        number_to_slug.insert("1".to_string(), "spirit-of-the-game".to_string());
        
        let content = "According to 16.3 and Section 1, the disc returns to the thrower.";
        let (processed, broken_refs) = process_number_references(content, &number_to_slug);
        
        let expected = "According to [16.3](rule:handling-contested-calls) and [Section 1](rule:spirit-of-the-game), the disc returns to the thrower.";
        assert_eq!(processed, expected);
        assert!(broken_refs.is_empty());
    }
}