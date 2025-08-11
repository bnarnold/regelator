use crate::{
    AppError,
    models::NewQuizAttempt,
    quiz_session::{QuizSession, clear_quiz_session_cookie},
    repository::RuleRepository,
};
use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use minijinja::Environment;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{Span, instrument};

#[derive(Serialize)]
pub struct QuizLandingData {
    pub language: String,
    pub rule_set_slug: String,
    pub has_progress: bool,
    pub questions_attempted: usize,
    pub total_questions: usize,
}

#[derive(Serialize)]
pub struct QuizQuestionData {
    pub question_id: String,
    pub question_text: String,
    pub difficulty_level: String,
    pub answers: Vec<QuizAnswerData>,
    pub session_id: String,
    pub rule_set_slug: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct QuizAnswerData {
    pub id: String,
    pub answer_text: String,
}

#[derive(Serialize)]
pub struct QuizResultData {
    pub question_id: String,
    pub question_text: String,
    pub difficulty_level: String,
    pub answers: Vec<QuizAnswerWithResult>,
    pub selected_answer_id: String,
    pub is_correct: bool,
    pub explanation: String,
    pub session_id: String,
    pub session_stats: SessionStatsView,
    pub total_questions_available: usize,
    pub questions_attempted: usize,
    pub language: String,
    pub rule_set_slug: String,
}

#[derive(Serialize)]
pub struct QuizSessionCompleteData {
    pub session_id: String,
    pub stats: SessionStatsView,
    pub missed_questions: Vec<MissedQuestionView>,
}

#[derive(Serialize)]
pub struct SessionStatsView {
    pub total_questions: usize,
    pub correct_answers: usize,
    pub accuracy_percentage: u32,
    pub current_streak: usize,
}

#[derive(Serialize)]
pub struct MissedQuestionView {
    pub question_text: String,
    pub difficulty_level: String,
    pub explanation: String,
}

#[derive(Serialize)]
pub struct QuizAnswerWithResult {
    pub id: String,
    pub answer_text: String,
    pub is_correct: bool,
    pub was_selected: bool,
}

#[derive(Deserialize)]
pub struct QuizSubmission {
    pub question_id: String,
    pub answer_id: String,
    // session_id now comes from QuizSession extractor
    // rule_set_slug and language come from path parameters
}

/// Quiz landing page
#[instrument(skip(template_env, repository, quiz_session), fields(language = %language, rule_set_slug = %rule_set_slug, session_id = %quiz_session.session_id()))]
pub async fn quiz_landing(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(template_env): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    quiz_session: QuizSession,
) -> Result<Html<String>, AppError> {
    let session_id = quiz_session.session_id();

    // Check if there's existing progress for this session
    let session_stats = repository.get_session_statistics(session_id).ok();
    let has_progress = session_stats
        .as_ref()
        .is_some_and(|stats| stats.total_questions > 0);

    // Get total available questions for progress calculation
    let total_questions = if has_progress {
        // Get the rule set and version to calculate total questions
        let rule_sets = repository.get_rule_sets()?;
        let rule_set = rule_sets
            .iter()
            .find(|rs| rs.slug == rule_set_slug)
            .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Rule set not found")))?;

        let version = repository
            .get_current_version(&rule_set_slug)?
            .ok_or_else(|| AppError(color_eyre::eyre::eyre!("No current version found")))?;

        let all_questions = repository.get_quiz_questions(&rule_set.id, &version.id)?;
        all_questions.len()
    } else {
        0
    };

    let template_data = QuizLandingData {
        language,
        rule_set_slug,
        has_progress,
        questions_attempted: session_stats
            .as_ref()
            .map_or(0, |stats| stats.total_questions),
        total_questions,
    };

    let template = template_env.get_template("quiz_landing.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Start a new quiz session
#[instrument(skip(repository, template_env, quiz_session), fields(language = %language, rule_set_slug = %rule_set_slug, session_id = %quiz_session.session_id()))]
pub async fn start_quiz_session(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
    quiz_session: QuizSession,
) -> Result<Html<String>, AppError> {
    let session_id = quiz_session.session_id().to_string();

    // Get first question for this session (middleware handles cookies)
    get_quiz_question_for_session(
        repository,
        template_env,
        session_id,
        language,
        rule_set_slug,
    )
    .await
}

/// Get a random quiz question (for next question flow)
#[instrument(skip(repository, template_env, quiz_session), fields(language = %language, rule_set_slug = %rule_set_slug, session_id = %quiz_session.session_id()))]
pub async fn random_quiz_question(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
    quiz_session: QuizSession,
) -> Result<Html<String>, AppError> {
    let session_id = quiz_session.session_id().to_string();

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
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Rule set not found")))?;

    let version = repository
        .get_current_version(&rule_set_slug)?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("No current version found")))?;

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
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Failed to select random question")))?;

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
#[instrument(skip(repository, template_env, quiz_session, submission), fields(language = %language, rule_set_slug = %rule_set_slug, session_id = %quiz_session.session_id(), question_id, answer_id))]
pub async fn submit_quiz_answer(
    Path((language, rule_set_slug)): Path<(String, String)>,
    State(repository): State<RuleRepository>,
    State(template_env): State<Arc<Environment<'static>>>,
    quiz_session: QuizSession,
    Form(submission): Form<QuizSubmission>,
) -> Result<Html<String>, AppError> {
    let session_id = quiz_session.session_id().to_string();

    // Record form data in span
    Span::current().record("question_id", &submission.question_id);
    Span::current().record("answer_id", &submission.answer_id);

    // Get the question by ID
    let question = repository
        .get_quiz_question_by_id(&submission.question_id)?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Question not found")))?;

    let answers = repository.get_quiz_answers(&submission.question_id)?;

    // Find selected answer and check if correct
    let selected_answer = answers
        .iter()
        .find(|a| a.id == submission.answer_id)
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Answer not found")))?;

    let is_correct = selected_answer.is_correct;

    // Record the attempt
    let attempt = NewQuizAttempt::new(
        session_id.clone(),
        submission.question_id.clone(),
        Some(submission.answer_id.clone()),
        Some(is_correct),
        None, // No timing for now
    );

    repository.create_quiz_attempt(&attempt)?;

    // Get session statistics after recording the attempt
    let db_stats = repository.get_session_statistics(&session_id)?;
    let session_stats = SessionStatsView {
        total_questions: db_stats.total_questions,
        correct_answers: db_stats.correct_answers,
        accuracy_percentage: db_stats.accuracy_percentage,
        current_streak: db_stats.current_streak,
    };

    // Get total available questions for this rule set using path parameter
    let rule_sets = repository.get_rule_sets()?;
    let rule_set = rule_sets
        .iter()
        .find(|rs| rs.slug == rule_set_slug)
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Rule set not found")))?;
    let version = repository
        .get_current_version(&rule_set_slug)?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("No current version found")))?;

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
        session_id: session_id.clone(),
        session_stats,
        total_questions_available,
        questions_attempted,
        language,
        rule_set_slug,
    };

    let template = template_env.get_template("quiz_result.html")?;
    let response = template.render(&template_data)?;

    Ok(Html(response))
}

/// Clear session data for privacy
#[instrument(skip(quiz_session, jar), fields(session_id = %quiz_session.session_id()))]
pub async fn clear_session_data(
    Path((language, rule_set_slug)): Path<(String, String)>,
    quiz_session: QuizSession,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    let jar = jar.add(clear_quiz_session_cookie());

    // Redirect back to quiz home for this rule set
    Ok((
        jar,
        axum::response::Redirect::to(&format!("/{language}/quiz/{rule_set_slug}",)),
    ))
}
