use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use eyre::{Result, WrapErr};
use regex::Regex;
use std::io::{self, BufRead};

use regelator::models::*;
use regelator::repository::RuleRepository;
use std::collections::HashMap;

#[derive(Debug)]
struct DefinitionData {
    term: String,
    slug: String,
    definition: String,
}

/// Generate a slug from a term (e.g., "Affect the play" -> "affect-the-play")
fn generate_slug(term: &str) -> String {
    term.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Read definitions from stdin, handling multi-line definitions
/// New definitions start with pattern: ^[^.:]+: (anything except dots and colons, then colon)
fn read_definitions_from_stdin() -> Result<Vec<DefinitionData>> {
    let stdin = io::stdin();
    let term_start_pattern = Regex::new(r"^([^.:]+?):\s*(.*)$").unwrap();
    let mut definitions = Vec::new();
    let mut current_term: Option<String> = None;
    let mut current_definition = String::new();

    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();
        
        // Check if this line starts a new definition
        if let Some(caps) = term_start_pattern.captures(line) {
            // Save the previous definition if we have one
            if let Some(term) = current_term.take() {
                if !current_definition.trim().is_empty() {
                    let slug = generate_slug(&term);
                    definitions.push(DefinitionData {
                        term,
                        slug,
                        definition: current_definition.trim().to_string(),
                    });
                }
            }
            
            // Start new definition
            current_term = Some(caps.get(1).unwrap().as_str().trim().to_string());
            current_definition = caps.get(2).unwrap().as_str().to_string();
        } else if current_term.is_some() {
            // Continue current definition (including empty lines for paragraph breaks)
            if !current_definition.is_empty() {
                current_definition.push('\n');
            }
            current_definition.push_str(line);
        } else {
            eprintln!("Warning: Skipping line outside definition: {}", line);
        }
    }
    
    // Don't forget the last definition
    if let Some(term) = current_term {
        if !current_definition.trim().is_empty() {
            let slug = generate_slug(&term);
            definitions.push(DefinitionData {
                term,
                slug,
                definition: current_definition.trim().to_string(),
            });
        }
    }

    Ok(definitions)
}

/// Process rule content to add {{definition:slug}} references for term mentions
fn process_definition_references(
    content: &str,
    term_to_slug: &HashMap<String, String>,
) -> String {
    if term_to_slug.is_empty() {
        return content.to_string();
    }
    
    // Build a single regex with all terms as alternatives, sorted by length (longest first)
    let mut sorted_terms: Vec<(&String, &String)> = term_to_slug.iter().collect();
    sorted_terms.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    
    let pattern_parts: Vec<String> = sorted_terms
        .iter()
        .map(|(term, _)| format!(r"\b{}\b", regex::escape(term)))
        .collect();
    
    let combined_pattern = format!("(?i)({})", pattern_parts.join("|"));
    let regex = match Regex::new(&combined_pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to compile regex: {}", e);
            return content.to_string();
        }
    };
    
    // Find all non-overlapping matches and build replacement map
    let mut replacements = Vec::new();
    
    for mat in regex.find_iter(content) {
        let matched_text = mat.as_str();
        
        // Find which term this match corresponds to (case-insensitive)
        if let Some((_, slug)) = sorted_terms.iter().find(|(term, _)| {
            term.to_lowercase() == matched_text.to_lowercase()
        }) {
            replacements.push((mat.start(), mat.end(), format!("[{}](definition:{})", matched_text, slug)));
            println!("    Found term '{}' -> [{}](definition:{})", matched_text, matched_text, slug);
        }
    }
    
    // Apply replacements from end to start to preserve indices
    let mut result = content.to_string();
    for (start, end, replacement) in replacements.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }
    
    result
}

/// Import definitions into the database
fn import_definitions(definitions: Vec<DefinitionData>) -> Result<()> {
    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://db/regelator.db".to_string());
    
    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);
    let pool = Pool::builder()
        .build(manager)
        .wrap_err("Failed to create connection pool")?;

    let repo = RuleRepository::new(pool);

    // Use the same hardcoded rule set as the rules import script
    // TODO: This should be passed as CLI parameters or discovered dynamically
    let rule_set_slug = "wfdf-ultimate";
    let version_name = "2025-2028";
    
    // Find the rule set by slug
    let rule_sets = repo.get_rule_sets()?;
    let rule_set = rule_sets.iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| eyre::eyre!("Rule set '{}' not found. Please import rules first.", rule_set_slug))?;
    
    let version = repo.get_current_version(&rule_set.slug)?
        .ok_or_else(|| eyre::eyre!("No current version found for rule set '{}'", rule_set_slug))?;

    // Build term name -> slug mapping for cross-reference processing
    let term_to_slug: HashMap<String, String> = definitions
        .iter()
        .map(|def| (def.term.clone(), def.slug.clone()))
        .collect();

    let definitions_count = definitions.len();
    println!("Importing {} definitions into rule set '{}' version '{}'", 
             definitions_count, rule_set.name, version.version_name);

    for mut definition in definitions {
        println!("  Importing: {} -> {}", definition.term, definition.slug);
        
        // Process definition content to add cross-references to other terms
        definition.definition = process_definition_references(&definition.definition, &term_to_slug);
        
        // Create glossary term
        let new_term = NewGlossaryTerm::new(
            rule_set.id.clone(),
            version.id.clone(),
            definition.slug.clone(),
        );
        
        let term = repo.create_glossary_term(new_term)?;
        
        // Create glossary content
        let new_content = NewGlossaryContent::new(
            term.id,
            "en".to_string(),
            definition.term,
            definition.definition,
        );
        
        repo.create_glossary_content(new_content)?;
    }

    println!("Successfully imported {} definitions", definitions_count);
    Ok(())
}

fn main() -> Result<()> {
    let definitions = read_definitions_from_stdin()?;
    if definitions.is_empty() {
        println!("No definitions provided");
        return Ok(());
    }
    import_definitions(definitions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Affect the play"), "affect-the-play");
        assert_eq!(generate_slug("Act of throwing"), "act-of-throwing");
        assert_eq!(generate_slug("Ultimate Frisbee"), "ultimate-frisbee");
        assert_eq!(generate_slug("Non-contact sport"), "non-contact-sport");
    }

    // Helper function to simulate stdin input for testing
    fn test_parse_definitions(input: &str) -> Result<Vec<DefinitionData>> {
        use std::io::Cursor;
        
        let cursor = Cursor::new(input);
        let term_start_pattern = Regex::new(r"^([^.:]+?):\s*(.*)$").unwrap();
        let mut definitions = Vec::new();
        let mut current_term: Option<String> = None;
        let mut current_definition = String::new();

        for line in cursor.lines() {
            let line = line?;
            let line = line.trim();
            
            if let Some(caps) = term_start_pattern.captures(line) {
                if let Some(term) = current_term.take() {
                    if !current_definition.trim().is_empty() {
                        let slug = generate_slug(&term);
                        definitions.push(DefinitionData {
                            term,
                            slug,
                            definition: current_definition.trim().to_string(),
                        });
                    }
                }
                
                current_term = Some(caps.get(1).unwrap().as_str().trim().to_string());
                current_definition = caps.get(2).unwrap().as_str().to_string();
            } else if current_term.is_some() {
                // Continue current definition (including empty lines for paragraph breaks)
                if !current_definition.is_empty() {
                    current_definition.push('\n');
                }
                current_definition.push_str(line);
            }
        }
        
        if let Some(term) = current_term {
            if !current_definition.trim().is_empty() {
                let slug = generate_slug(&term);
                definitions.push(DefinitionData {
                    term,
                    slug,
                    definition: current_definition.trim().to_string(),
                });
            }
        }

        Ok(definitions)
    }

    #[test]
    fn test_single_line_definition() {
        let input = "Affect the play: A breach or call affects the play if it is reasonable to assume that the outcome of the specific play may have been meaningfully different had the breach or call not occurred.";
        
        let definitions = test_parse_definitions(input).unwrap();
        
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Affect the play");
        assert_eq!(definitions[0].slug, "affect-the-play");
        assert_eq!(definitions[0].definition, "A breach or call affects the play if it is reasonable to assume that the outcome of the specific play may have been meaningfully different had the breach or call not occurred.");
    }

    #[test]
    fn test_multi_line_definition() {
        let input = "Ultimate Frisbee: A sport played by two teams\nof seven players each on a rectangular field\nwith end zones at each end.";
        
        let definitions = test_parse_definitions(input).unwrap();
        
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Ultimate Frisbee");
        assert_eq!(definitions[0].slug, "ultimate-frisbee");
        assert_eq!(definitions[0].definition, "A sport played by two teams\nof seven players each on a rectangular field\nwith end zones at each end.");
    }

    #[test]
    fn test_multiple_definitions() {
        let input = "Act of throwing: The act of releasing the disc.\n\nFoul: A violation of the rules.\nCalling fouls is important for fair play.";
        
        let definitions = test_parse_definitions(input).unwrap();
        
        assert_eq!(definitions.len(), 2);
        
        assert_eq!(definitions[0].term, "Act of throwing");
        assert_eq!(definitions[0].slug, "act-of-throwing");
        assert_eq!(definitions[0].definition, "The act of releasing the disc.");
        
        assert_eq!(definitions[1].term, "Foul");
        assert_eq!(definitions[1].slug, "foul");
        assert_eq!(definitions[1].definition, "A violation of the rules.\nCalling fouls is important for fair play.");
    }

    #[test]
    fn test_parentheses_and_abbreviations() {
        let input = "Out-of-bounds (OB): Everything that is not part of the playing field, including the perimeter lines.";
        
        let definitions = test_parse_definitions(input).unwrap();
        
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Out-of-bounds (OB)");
        assert_eq!(definitions[0].slug, "out-of-bounds-ob");
        assert_eq!(definitions[0].definition, "Everything that is not part of the playing field, including the perimeter lines.");
    }

    #[test]
    fn test_invalid_term_start_ignored() {
        let input = "1.2.3. this should be ignored\nAct of throwing: The act of releasing the disc.";
        
        let definitions = test_parse_definitions(input).unwrap();
        
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Act of throwing");
    }

    #[test]
    fn test_process_definition_references() {
        let mut term_to_slug = HashMap::new();
        term_to_slug.insert("pass".to_string(), "pass".to_string());
        term_to_slug.insert("foul".to_string(), "foul".to_string());
        term_to_slug.insert("Ultimate Frisbee".to_string(), "ultimate-frisbee".to_string());
        
        let content = "A pass in Ultimate Frisbee can be intercepted. A foul stops play.";
        let processed = process_definition_references(content, &term_to_slug);
        
        let expected = "A [pass](definition:pass) in [Ultimate Frisbee](definition:ultimate-frisbee) can be intercepted. A [foul](definition:foul) stops play.";
        assert_eq!(processed, expected);
    }

    #[test]
    fn test_process_definition_references_case_insensitive() {
        let mut term_to_slug = HashMap::new();
        term_to_slug.insert("Pass".to_string(), "pass".to_string());
        
        let content = "The pass was incomplete. A PASS requires catching. Lower case pass works too.";
        let processed = process_definition_references(content, &term_to_slug);
        
        let expected = "The [pass](definition:pass) was incomplete. A [PASS](definition:pass) requires catching. Lower case [pass](definition:pass) works too.";
        assert_eq!(processed, expected);
    }

    #[test]
    fn test_process_definition_references_word_boundaries() {
        let mut term_to_slug = HashMap::new();
        term_to_slug.insert("pass".to_string(), "pass".to_string());
        
        let content = "A pass is good, but passage is different. Passing and bypass contain pass.";
        let processed = process_definition_references(content, &term_to_slug);
        
        // Only the standalone word "pass" should be replaced, not parts of other words
        let expected = "A [pass](definition:pass) is good, but passage is different. Passing and bypass contain [pass](definition:pass).";
        assert_eq!(processed, expected);
    }

    #[test]
    fn test_process_definition_references_overlapping_terms() {
        let mut term_to_slug = HashMap::new();
        term_to_slug.insert("player".to_string(), "player".to_string());
        term_to_slug.insert("offensive player".to_string(), "offensive-player".to_string());
        term_to_slug.insert("possession of the disc".to_string(), "possession-of-the-disc".to_string());
        term_to_slug.insert("throw".to_string(), "throw".to_string());
        
        let content = "The offensive player in possession of the disc, or the player who has just thrown the disc prior to when the result of the throw has been determined.";
        let processed = process_definition_references(content, &term_to_slug);
        
        // Should prefer longer matches: "offensive player" over "player", "possession of the disc" over parts
        let expected = "The [offensive player](definition:offensive-player) in [possession of the disc](definition:possession-of-the-disc), or the [player](definition:player) who has just thrown the disc prior to when the result of the [throw](definition:throw) has been determined.";
        assert_eq!(processed, expected);
    }
}