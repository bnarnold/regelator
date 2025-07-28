use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use eyre::{Result, WrapErr};

use crate::models::*;

type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct RuleRepository {
    pool: DbPool,
}

impl RuleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get all rule sets
    pub fn get_rule_sets(&self) -> Result<Vec<RuleSet>> {
        use crate::schema::rule_sets::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = rule_sets
            .select(RuleSet::as_select())
            .load(&mut conn)
            .wrap_err("Failed to load rule sets")?;

        Ok(results)
    }

    /// Get current version for a rule set
    pub fn get_current_version(&self, rule_set_slug: &str) -> Result<Option<Version>> {
        use crate::schema::rule_sets::dsl as rs_dsl;
        use crate::schema::versions::dsl as v_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = v_dsl::versions
            .inner_join(rs_dsl::rule_sets)
            .filter(rs_dsl::slug.eq(rule_set_slug))
            .filter(v_dsl::is_current.eq(true))
            .select(Version::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load current version")?;

        Ok(result)
    }

    /// Get version by name for a rule set
    pub fn get_version_by_name(
        &self,
        rule_set_slug: &str,
        version_name: &str,
    ) -> Result<Option<Version>> {
        use crate::schema::rule_sets::dsl as rs_dsl;
        use crate::schema::versions::dsl as v_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = v_dsl::versions
            .inner_join(rs_dsl::rule_sets)
            .filter(rs_dsl::slug.eq(rule_set_slug))
            .filter(v_dsl::version_name.eq(version_name))
            .select(Version::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load version by name")?;

        Ok(result)
    }

    /// Get rules for a specific version, ordered by number
    pub fn get_rules_for_version(&self, version_id_param: &str) -> Result<Vec<Rule>> {
        use crate::schema::rules::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = rules
            .filter(version_id.eq(version_id_param))
            .select(Rule::as_select())
            .order(number.asc())
            .load(&mut conn)
            .wrap_err("Failed to load rules for version")?;

        Ok(results)
    }

    /// Get rules with their content for a specific version and language
    pub fn get_rules_with_content_for_version(
        &self,
        version_id_param: &str,
        language_param: &str,
    ) -> Result<Vec<(Rule, RuleContent)>> {
        use crate::schema::rule_content::dsl as content_dsl;
        use crate::schema::rules::dsl as rules_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = rules_dsl::rules
            .inner_join(
                content_dsl::rule_content.on(
                    rules_dsl::id.eq(content_dsl::rule_id)
                        .and(content_dsl::language.eq(language_param))
                )
            )
            .filter(rules_dsl::version_id.eq(version_id_param))
            .select((Rule::as_select(), RuleContent::as_select()))
            .load::<(Rule, RuleContent)>(&mut conn)
            .wrap_err("Failed to load rules with content")?;

        Ok(results)
    }

    /// Get rule by ID
    pub fn get_rule_by_id(&self, rule_id_param: &str) -> Result<Option<Rule>> {
        use crate::schema::rules::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = rules
            .filter(id.eq(rule_id_param))
            .select(Rule::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load rule by id")?;

        Ok(result)
    }

    /// Get rule by slug and version
    pub fn get_rule_by_slug(
        &self,
        rule_slug: &str,
        version_id_param: &str,
    ) -> Result<Option<Rule>> {
        use crate::schema::rules::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = rules
            .filter(slug.eq(rule_slug))
            .filter(version_id.eq(version_id_param))
            .select(Rule::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load rule by slug")?;

        Ok(result)
    }

    /// Get rule content for a rule in a specific language (with fallback to English)
    pub fn get_rule_content(
        &self,
        rule_id_param: &str,
        language_param: &str,
    ) -> Result<Option<RuleContent>> {
        use crate::schema::rule_content::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        // Try to get content in requested language first
        let result = rule_content
            .filter(rule_id.eq(rule_id_param))
            .filter(language.eq(language_param))
            .select(RuleContent::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load rule content")?;

        // If no content in requested language, fallback to English
        if result.is_none() && language_param != "en" {
            let en_result = rule_content
                .filter(rule_id.eq(rule_id_param))
                .filter(language.eq("en"))
                .select(RuleContent::as_select())
                .first(&mut conn)
                .optional()
                .wrap_err("Failed to load English fallback content")?;

            return Ok(en_result);
        }

        Ok(result)
    }


    /// Get child rules for a parent rule
    pub fn get_child_rules(&self, parent_id: &str) -> Result<Vec<Rule>> {
        use crate::schema::rules::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = rules
            .filter(parent_rule_id.eq(parent_id))
            .select(Rule::as_select())
            .order(number.asc())
            .load(&mut conn)
            .wrap_err("Failed to load child rules")?;

        Ok(results)
    }

    /// Create a new rule set
    pub fn create_rule_set(&self, new_rule_set: NewRuleSet) -> Result<RuleSet> {
        use crate::schema::rule_sets::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(rule_sets)
            .values(&new_rule_set)
            .execute(&mut conn)
            .wrap_err("Failed to create rule set")?;

        // Return the created rule set
        let created = rule_sets
            .filter(id.eq(&new_rule_set.id))
            .select(RuleSet::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created rule set")?;

        Ok(created)
    }

    /// Create a new version
    pub fn create_version(&self, new_version: NewVersion) -> Result<Version> {
        use crate::schema::versions::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(versions)
            .values(&new_version)
            .execute(&mut conn)
            .wrap_err("Failed to create version")?;

        // Return the created version
        let created = versions
            .filter(id.eq(&new_version.id))
            .select(Version::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created version")?;

        Ok(created)
    }

    /// Create a new rule
    pub fn create_rule(&self, new_rule: NewRule) -> Result<Rule> {
        use crate::schema::rules::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(rules)
            .values(&new_rule)
            .execute(&mut conn)
            .wrap_err("Failed to create rule")?;

        // Return the created rule
        let created = rules
            .filter(id.eq(&new_rule.id))
            .select(Rule::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created rule")?;

        Ok(created)
    }

    /// Create rule content
    pub fn create_rule_content(&self, new_content: NewRuleContent) -> Result<RuleContent> {
        use crate::schema::rule_content::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(rule_content)
            .values(&new_content)
            .execute(&mut conn)
            .wrap_err("Failed to create rule content")?;

        // Return the created content
        let created = rule_content
            .filter(id.eq(&new_content.id))
            .select(RuleContent::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created rule content")?;

        Ok(created)
    }

    /// Create a new glossary term
    pub fn create_glossary_term(&self, new_term: NewGlossaryTerm) -> Result<GlossaryTerm> {
        use crate::schema::glossary_terms::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(glossary_terms)
            .values(&new_term)
            .execute(&mut conn)
            .wrap_err("Failed to create glossary term")?;

        // Return the created term
        let created = glossary_terms
            .filter(id.eq(&new_term.id))
            .select(GlossaryTerm::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created glossary term")?;

        Ok(created)
    }

    /// Create glossary content for a term
    pub fn create_glossary_content(&self, new_content: NewGlossaryContent) -> Result<GlossaryContent> {
        use crate::schema::glossary_content::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(glossary_content)
            .values(&new_content)
            .execute(&mut conn)
            .wrap_err("Failed to create glossary content")?;

        // Return the created content
        let created = glossary_content
            .filter(id.eq(&new_content.id))
            .select(GlossaryContent::as_select())
            .first(&mut conn)
            .wrap_err("Failed to retrieve created glossary content")?;

        Ok(created)
    }

    /// Get all glossary terms for a rule set and version
    pub fn get_glossary_terms(&self, rule_set_id_param: &str, version_id_param: &str) -> Result<Vec<(GlossaryTerm, GlossaryContent)>> {
        use crate::schema::glossary_terms::dsl as terms_dsl;
        use crate::schema::glossary_content::dsl as content_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = terms_dsl::glossary_terms
            .inner_join(content_dsl::glossary_content.on(content_dsl::term_id.eq(terms_dsl::id)))
            .filter(terms_dsl::rule_set_id.eq(rule_set_id_param))
            .filter(terms_dsl::version_id.eq(version_id_param))
            .filter(content_dsl::language.eq("en"))
            .select((GlossaryTerm::as_select(), GlossaryContent::as_select()))
            .load(&mut conn)
            .wrap_err("Failed to load glossary terms")?;

        Ok(results)
    }

    /// Find a glossary term by slug
    pub fn find_glossary_term_by_slug(&self, rule_set_id_param: &str, version_id_param: &str, slug_param: &str) -> Result<Option<(GlossaryTerm, GlossaryContent)>> {
        use crate::schema::glossary_terms::dsl as terms_dsl;
        use crate::schema::glossary_content::dsl as content_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = terms_dsl::glossary_terms
            .inner_join(content_dsl::glossary_content.on(content_dsl::term_id.eq(terms_dsl::id)))
            .filter(terms_dsl::rule_set_id.eq(rule_set_id_param))
            .filter(terms_dsl::version_id.eq(version_id_param))
            .filter(terms_dsl::slug.eq(slug_param))
            .filter(content_dsl::language.eq("en"))
            .select((GlossaryTerm::as_select(), GlossaryContent::as_select()))
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to find glossary term")?;

        Ok(result)
    }

    // Quiz repository methods

    /// Create a complete quiz question with answers and rule links in a transaction
    pub fn create_quiz_question_complete(&self, question_data: &crate::models::QuizQuestionData) -> Result<QuizQuestion> {
        use crate::schema::quiz_answers::dsl as qa_dsl;
        use crate::schema::quiz_question_rules::dsl as qqr_dsl;
        use crate::schema::quiz_questions::dsl as qq_dsl;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let (question, answers, rule_links) = question_data.to_database_entities();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // Insert the question
            diesel::insert_into(qq_dsl::quiz_questions)
                .values(&question)
                .execute(conn)?;

            // Insert all answers
            if !answers.is_empty() {
                diesel::insert_into(qa_dsl::quiz_answers)
                    .values(&answers)
                    .execute(conn)?;
            }

            // Insert rule links
            if !rule_links.is_empty() {
                diesel::insert_into(qqr_dsl::quiz_question_rules)
                    .values(&rule_links)
                    .execute(conn)?;
            }

            Ok(())
        })
        .wrap_err("Failed to create quiz question with transaction")?;

        // Return the created question
        let result = qq_dsl::quiz_questions
            .filter(qq_dsl::id.eq(&question.id))
            .select(QuizQuestion::as_select())
            .first(&mut conn)
            .wrap_err("Failed to load created quiz question")?;

        Ok(result)
    }

    /// Get quiz questions for a rule set and version
    pub fn get_quiz_questions(&self, rule_set_id_param: &str, version_id_param: &str) -> Result<Vec<QuizQuestion>> {
        use crate::schema::quiz_questions::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = quiz_questions
            .filter(rule_set_id.eq(rule_set_id_param))
            .filter(version_id.eq(version_id_param))
            .select(QuizQuestion::as_select())
            .load(&mut conn)
            .wrap_err("Failed to load quiz questions")?;

        Ok(results)
    }

    /// Get a specific quiz question by ID
    pub fn get_quiz_question_by_id(&self, question_id_param: &str) -> Result<Option<QuizQuestion>> {
        use crate::schema::quiz_questions::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let result = quiz_questions
            .filter(id.eq(question_id_param))
            .select(QuizQuestion::as_select())
            .first(&mut conn)
            .optional()
            .wrap_err("Failed to load quiz question")?;

        Ok(result)
    }

    /// Get quiz answers for a question
    pub fn get_quiz_answers(&self, question_id_param: &str) -> Result<Vec<QuizAnswer>> {
        use crate::schema::quiz_answers::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = quiz_answers
            .filter(question_id.eq(question_id_param))
            .order(sort_order.asc())
            .select(QuizAnswer::as_select())
            .load(&mut conn)
            .wrap_err("Failed to load quiz answers")?;

        Ok(results)
    }

    /// Record a quiz attempt
    pub fn create_quiz_attempt(&self, attempt: &NewQuizAttempt) -> Result<QuizAttempt> {
        use crate::schema::quiz_attempts::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        diesel::insert_into(quiz_attempts)
            .values(attempt)
            .execute(&mut conn)
            .wrap_err("Failed to create quiz attempt")?;

        let result = quiz_attempts
            .filter(id.eq(&attempt.id))
            .select(QuizAttempt::as_select())
            .first(&mut conn)
            .wrap_err("Failed to load created quiz attempt")?;

        Ok(result)
    }

    /// Get quiz attempts for a session
    pub fn get_session_attempts(&self, session_id_param: &str) -> Result<Vec<QuizAttempt>> {
        use crate::schema::quiz_attempts::dsl::*;

        let mut conn = self
            .pool
            .get()
            .wrap_err("Failed to get database connection")?;

        let results = quiz_attempts
            .filter(session_id.eq(session_id_param))
            .order(created_at.desc())
            .select(QuizAttempt::as_select())
            .load(&mut conn)
            .wrap_err("Failed to load session attempts")?;

        Ok(results)
    }
}
