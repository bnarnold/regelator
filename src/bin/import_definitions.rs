use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use eyre::{Result, WrapErr};
use regex::Regex;
use std::io::{self, BufRead};

use regelator::models::*;
use regelator::repository::RuleRepository;

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
/// New definitions start with pattern: ^[^.:\[\]]+: (anything except dots, colons, and brackets, then colon)
fn read_definitions_from_stdin() -> Result<Vec<DefinitionData>> {
    let stdin = io::stdin();
    let term_start_pattern = Regex::new(r"^([^.:\[\]]+?):\s*(.*)$").unwrap();
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

/// Import definitions into the database
fn import_definitions(definitions: Vec<DefinitionData>) -> Result<()> {
    // Database setup
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db/regelator.db".to_string());

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
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| {
            eyre::eyre!(
                "Rule set '{}' not found. Please import rules first.",
                rule_set_slug
            )
        })?;

    let version = repo
        .get_current_version(&rule_set.slug)?
        .ok_or_else(|| eyre::eyre!("No current version found for rule set '{}'", rule_set_slug))?;

    let definitions_count = definitions.len();
    println!(
        "Importing {} definitions into rule set '{}' version '{}'",
        definitions_count, rule_set.name, version.version_name
    );

    for definition in definitions {
        println!("  Importing: {} -> {}", definition.term, definition.slug);

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
        let term_start_pattern = Regex::new(r"^([^.:\[\]]+?):\s*(.*)$").unwrap();
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
        assert_eq!(
            definitions[1].definition,
            "A violation of the rules.\nCalling fouls is important for fair play."
        );
    }

    #[test]
    fn test_parentheses_and_abbreviations() {
        let input = "Out-of-bounds (OB): Everything that is not part of the playing field, including the perimeter lines.";

        let definitions = test_parse_definitions(input).unwrap();

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Out-of-bounds (OB)");
        assert_eq!(definitions[0].slug, "out-of-bounds-ob");
        assert_eq!(
            definitions[0].definition,
            "Everything that is not part of the playing field, including the perimeter lines."
        );
    }

    #[test]
    fn test_invalid_term_start_ignored() {
        let input =
            "1.2.3. this should be ignored\nAct of throwing: The act of releasing the disc.";

        let definitions = test_parse_definitions(input).unwrap();

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Act of throwing");
    }

    #[test]
    fn test_definition_with_links() {
        let input = "Affect the play: A [breach](definition:breach) or [call](definition:call) affects the play if it is reasonable to assume that the outcome of the specific play may have been meaningfully different.";

        let definitions = test_parse_definitions(input).unwrap();

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].term, "Affect the play");
        assert_eq!(definitions[0].slug, "affect-the-play");
        assert_eq!(
            definitions[0].definition,
            "A [breach](definition:breach) or [call](definition:call) affects the play if it is reasonable to assume that the outcome of the specific play may have been meaningfully different."
        );
    }
}

