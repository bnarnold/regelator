use crate::models::QuestionStatus;
use crate::{repository::RuleRepository, AppError};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use axum_extra::extract::{CookieJar, Form, Query};
use chrono::{Duration, Utc};
use minijinja::Environment;
use regelator::auth::{clear_admin_cookie, create_admin_cookie, AdminToken};
use regelator::config::Config;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{instrument, Span};

// Admin authentication structures
#[derive(Deserialize)]
pub struct AdminLoginForm {
    pub username: String,
    pub password: String,
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
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Serialize)]
struct ChangePasswordContext {
    error: Option<String>,
    success: Option<String>,
}

/// Show admin login form
#[instrument(skip(templates))]
pub async fn admin_login_form(
    State(templates): State<Arc<Environment<'static>>>,
) -> Result<Html<String>, AppError> {
    let context = AdminLoginContext { error: None };
    let tmpl = templates.get_template("admin_login.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Process admin login
#[instrument(skip(templates, repository, config, jar, form_data), fields(username))]
pub async fn admin_login_submit(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    State(config): State<Config>,
    jar: CookieJar,
    Form(form_data): Form<AdminLoginForm>,
) -> Result<(CookieJar, Html<String>), AppError> {
    // Record username in span (but not password for security)
    Span::current().record("username", &form_data.username);

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
    let parsed_hash = PasswordHash::new(&admin.password_hash).map_err(|e| {
        AppError(color_eyre::eyre::eyre!(
            "Failed to parse password hash: {}",
            e
        ))
    })?;

    if argon2
        .verify_password(form_data.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        // Password is correct - update last login and create signed cookie
        repository.update_admin_last_login(&admin.id)?;

        let cookie = create_admin_cookie(
            admin.id,
            admin.username.clone(),
            &config.security.jwt_secret,
            config.session_duration(),
        )
        .map_err(|e| {
            AppError(color_eyre::eyre::eyre!(
                "Failed to create admin cookie: {}",
                e
            ))
        })?;

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
#[instrument(skip(templates, admin), fields(admin_username = %admin.username()))]
pub async fn admin_dashboard(
    State(templates): State<Arc<Environment<'static>>>,
    admin: AdminToken,
) -> Result<Html<String>, AppError> {
    let context = AdminDashboardContext {
        username: admin.username().to_string(),
    };
    let tmpl = templates.get_template("admin_dashboard.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Admin logout (clears cookie and redirects to login)
#[instrument(skip(jar))]
pub async fn admin_logout(jar: CookieJar) -> Result<(CookieJar, Redirect), AppError> {
    let clear_cookie = clear_admin_cookie();
    Ok((jar.add(clear_cookie), Redirect::to("/admin/login")))
}

/// Show password change form (protected route)
#[instrument(skip(templates, _admin), fields(admin_username = %_admin.username()))]
pub async fn admin_change_password_form(
    State(templates): State<Arc<Environment<'static>>>,
    _admin: AdminToken,
) -> Result<Html<String>, AppError> {
    let context = ChangePasswordContext {
        error: None,
        success: None,
    };
    let tmpl = templates.get_template("admin_change_password.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Process password change (protected route)
#[instrument(skip(templates, repository, admin, form_data), fields(admin_username = %admin.username()))]
pub async fn admin_change_password_submit(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    admin: AdminToken,
    Form(form_data): Form<ChangePasswordForm>,
) -> Result<Html<String>, AppError> {
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
    let admin_record = repository
        .find_admin_by_username(admin.username())?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Admin not found")))?;

    // Verify current password
    let argon2 = Argon2::default();
    let current_parsed_hash = PasswordHash::new(&admin_record.password_hash).map_err(|e| {
        AppError(color_eyre::eyre::eyre!(
            "Failed to parse current password hash: {}",
            e
        ))
    })?;

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
        .map_err(|e| {
            AppError(color_eyre::eyre::eyre!(
                "Failed to hash new password: {}",
                e
            ))
        })?
        .to_string();

    // Update password in database (with current hash verification for extra security)
    repository.update_admin_password(
        &admin_record.id,
        &admin_record.password_hash,
        &new_password_hash,
    )?;

    // Show success message
    let context = ChangePasswordContext {
        error: None,
        success: Some("Password changed successfully".to_string()),
    };
    let tmpl = templates.get_template("admin_change_password.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

// Question Management Handlers

#[derive(Deserialize)]
pub struct QuestionFilters {
    pub search: Option<String>,
    pub status: Option<QuestionStatus>,
    pub difficulty: Option<String>,
}

#[derive(Deserialize)]
pub struct QuestionForm {
    pub status: String,
    pub difficulty_level: String,
    pub question_text: String,
    pub explanation: String,
    pub answers: Vec<String>,
    pub correct_answer: usize,
    pub action: Option<String>,
    // TODO: there is an input for rule references, we don't store these in the database yet though
}

#[derive(Serialize)]
struct QuestionsListContext {
    questions: Vec<QuestionWithAnswers>,
    search_query: Option<String>,
    status_filter: Option<QuestionStatus>,
    difficulty_filter: Option<String>,
    rule_set_name: String,
}

#[derive(Serialize)]
struct QuestionWithAnswers {
    question: crate::models::quiz::QuizQuestion,
    answers: Vec<crate::models::quiz::QuizAnswer>,
}

#[derive(Serialize)]
struct QuestionFormContext {
    question: Option<crate::models::quiz::QuizQuestion>,
    answers: Option<Vec<crate::models::quiz::QuizAnswer>>,
    errors: Option<Vec<String>>,
}

#[derive(Serialize)]
struct QuestionPreviewContext {
    question: crate::models::quiz::QuizQuestion,
    answers: Vec<crate::models::quiz::QuizAnswer>,
}

/// Show questions list with filtering
#[instrument(skip(templates, repository, _admin, filters), fields(admin_username = %_admin.username(), search = ?filters.search, status = ?filters.status, difficulty = ?filters.difficulty))]
pub async fn questions_list(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Query(filters): Query<QuestionFilters>,
) -> Result<Html<String>, AppError> {
    // Parse status filter
    let status_filter = filters.status;

    // Get questions with filters
    let questions = repository.get_questions_filtered(
        status_filter,
        filters.difficulty.as_deref(),
        filters.search.as_deref(),
        Some(50), // limit
        None,     // offset - TODO: implement pagination
    )?;

    // Get answers for each question (simplified for list view)
    let questions_with_answers: Vec<QuestionWithAnswers> = questions
        .into_iter()
        .map(|question| QuestionWithAnswers {
            question,
            answers: vec![], // We don't need full answers for the list view
        })
        .collect();

    let context = QuestionsListContext {
        questions: questions_with_answers,
        search_query: filters.search,
        status_filter: filters.status,
        difficulty_filter: filters.difficulty,
        rule_set_name: "Ultimate Frisbee Rules".to_string(),
    };

    let tmpl = templates.get_template("admin_questions_list.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Show new question form
#[instrument(skip(templates, _admin), fields(admin_username = %_admin.username()))]
pub async fn new_question_form(
    State(templates): State<Arc<Environment<'static>>>,
    _admin: AdminToken,
) -> Result<Html<String>, AppError> {
    let context = QuestionFormContext {
        question: None,
        answers: None,
        errors: None,
    };

    let tmpl = templates.get_template("admin_question_form.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Create new question
#[instrument(skip(repository, _admin, form_data), fields(admin_username = %_admin.username(), question_text, difficulty = %form_data.difficulty_level))]
pub async fn create_question(
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Form(form_data): Form<QuestionForm>,
) -> Result<Redirect, AppError> {
    // Basic validation
    let mut errors = Vec::new();

    if form_data.question_text.trim().is_empty() {
        errors.push("Question text is required".to_string());
    }

    if form_data.explanation.trim().is_empty() {
        errors.push("Explanation is required".to_string());
    }

    // Filter out empty answers
    let valid_answers: Vec<String> = form_data
        .answers
        .into_iter()
        .filter(|answer| !answer.trim().is_empty())
        .collect();

    if valid_answers.len() < 2 {
        errors.push("At least 2 answer options are required".to_string());
    }

    if form_data.correct_answer >= valid_answers.len() {
        errors.push("Invalid correct answer selection".to_string());
    }

    // For now, if there are validation errors, just redirect back
    // TODO: Implement proper error handling with form state preservation
    if !errors.is_empty() {
        return Ok(Redirect::to("/admin/questions/new"));
    }

    // Parse status
    let status = crate::models::quiz::QuestionStatus::from_str(&form_data.status)
        .unwrap_or(crate::models::quiz::QuestionStatus::Draft);

    // Create question
    let new_question = crate::models::quiz::NewQuizQuestion::new(
        "default-rule-set-id".to_string(), // TODO: Get from context
        "default-version-id".to_string(),  // TODO: Get from context
        form_data.question_text,
        form_data.explanation,
        form_data.difficulty_level,
        status,
    );

    // Create answers
    let new_answers: Vec<crate::models::quiz::NewQuizAnswer> = valid_answers
        .into_iter()
        .enumerate()
        .map(|(index, answer_text)| {
            crate::models::quiz::NewQuizAnswer::new(
                new_question.id.clone(),
                answer_text,
                index == form_data.correct_answer,
                index as i32,
            )
        })
        .collect();

    // Save to database
    let _created_question = repository.create_question_with_answers(new_question, new_answers)?;

    Ok(Redirect::to("/admin/questions"))
}

/// Show edit question form
#[instrument(skip(templates, repository, _admin), fields(admin_username = %_admin.username(), question_id = %question_id))]
pub async fn edit_question_form(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Path(question_id): Path<String>,
) -> Result<Html<String>, AppError> {
    // Get question with answers
    let (question, answers) = repository
        .get_question_with_answers(&question_id)?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Question not found")))?;

    let context = QuestionFormContext {
        question: Some(question),
        answers: Some(answers),
        errors: None,
    };

    let tmpl = templates.get_template("admin_question_form.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Show question preview
#[instrument(skip(templates, repository, _admin), fields(admin_username = %_admin.username(), question_id = %question_id))]
pub async fn preview_question(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Path(question_id): Path<String>,
) -> Result<Html<String>, AppError> {
    // Get question with answers
    let (question, answers) = repository
        .get_question_with_answers(&question_id)?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Question not found")))?;

    let context = QuestionPreviewContext { question, answers };

    let tmpl = templates.get_template("admin_question_preview.html")?;
    let rendered = tmpl.render(context)?;
    Ok(Html(rendered))
}

/// Update question
#[instrument(skip(repository, _admin, form_data), fields(admin_username = %_admin.username(), question_id = %question_id, action = ?form_data.action, difficulty = %form_data.difficulty_level))]
pub async fn update_question(
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Path(question_id): Path<String>,
    Form(form_data): Form<QuestionForm>,
) -> Result<Redirect, AppError> {
    // Handle delete action
    if form_data.action.as_deref() == Some("delete") {
        repository.delete_question(&question_id)?;
        return Ok(Redirect::to("/admin/questions"));
    }

    // Parse status
    let status = crate::models::quiz::QuestionStatus::from_str(&form_data.status)
        .unwrap_or(crate::models::quiz::QuestionStatus::Draft);

    // Update question
    repository.update_question(
        &question_id,
        &form_data.question_text,
        &form_data.explanation,
        &form_data.difficulty_level,
        status,
    )?;

    // Update answers
    let valid_answers: Vec<String> = form_data
        .answers
        .into_iter()
        .filter(|answer| !answer.trim().is_empty())
        .collect();

    if !valid_answers.is_empty() && form_data.correct_answer < valid_answers.len() {
        let new_answers: Vec<crate::models::quiz::NewQuizAnswer> = valid_answers
            .into_iter()
            .enumerate()
            .map(|(index, answer_text)| {
                crate::models::quiz::NewQuizAnswer::new(
                    question_id.clone(),
                    answer_text,
                    index == form_data.correct_answer,
                    index as i32,
                )
            })
            .collect();

        repository.update_question_answers(&question_id, new_answers)?;
    }

    Ok(Redirect::to("/admin/questions"))
}

// Analytics handlers

#[derive(Serialize)]
pub struct AdminStatsContext {
    pub aggregate_stats: crate::models::quiz::AggregateStatistics,
    pub question_stats: Vec<crate::models::quiz::QuestionStatistics>,
    pub current_filter: String,
    pub current_start_date: String,
    pub current_end_date: String,
    pub current_filter_value: String,
}

#[derive(Deserialize, Debug)]
pub struct StatsQueryParams {
    pub filter: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Show admin statistics dashboard
#[instrument(skip(templates, repository, _admin), fields(admin_username = %_admin.username()))]
pub async fn admin_stats_dashboard(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Query(params): Query<StatsQueryParams>,
) -> Result<Html<String>, AppError> {
    // Determine date range based on filter
    let (start_date, end_date, filter_name, filter_value) = match params.filter.as_deref() {
        Some("7days") => {
            let end = Utc::now();
            let start = end - Duration::days(7);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
                "Last 7 Days".to_string(),
                "7days".to_string(),
            )
        }
        Some("30days") => {
            let end = Utc::now();
            let start = end - Duration::days(30);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
                "Last 30 Days".to_string(),
                "30days".to_string(),
            )
        }
        Some("custom") => (
            params.start_date.clone(),
            params.end_date.clone(),
            "Custom Range".to_string(),
            "custom".to_string(),
        ),
        _ => (None, None, "All Time".to_string(), "all".to_string()),
    };

    // Get statistics
    let aggregate_stats =
        repository.get_aggregate_quiz_statistics(start_date.as_deref(), end_date.as_deref())?;

    let question_stats = repository.get_question_statistics(
        start_date.as_deref(),
        end_date.as_deref(),
        None, // No limit
        None, // No offset
    )?;

    let context = AdminStatsContext {
        aggregate_stats,
        question_stats,
        current_filter: filter_name,
        current_start_date: params.start_date.unwrap_or_default(),
        current_end_date: params.end_date.unwrap_or_default(),
        current_filter_value: filter_value,
    };

    let template = templates.get_template("admin_stats.html")?;
    let rendered = template.render(&context)?;

    Ok(Html(rendered))
}

/// Show detailed question statistics
#[derive(Serialize)]
struct QuestionDetailStatsContext {
    pub question_detail_stats: crate::models::quiz::QuestionDetailStats,
    pub current_filter: String,
}

#[instrument(skip(templates, repository, _admin), fields(admin_username = %_admin.username(), question_id = %question_id))]
pub async fn admin_question_detail_stats(
    State(templates): State<Arc<Environment<'static>>>,
    State(repository): State<RuleRepository>,
    _admin: AdminToken,
    Path(question_id): Path<String>,
    Query(params): Query<StatsQueryParams>,
) -> Result<Html<String>, AppError> {
    // Determine date range based on filter (same logic as main stats dashboard)
    let (start_date, end_date, filter_name) = match params.filter.as_deref() {
        Some("7days") => {
            let end = Utc::now();
            let start = end - Duration::days(7);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
                "Last 7 Days".to_string(),
            )
        }
        Some("30days") => {
            let end = Utc::now();
            let start = end - Duration::days(30);
            (
                Some(start.format("%Y-%m-%d").to_string()),
                Some(end.format("%Y-%m-%d").to_string()),
                "Last 30 Days".to_string(),
            )
        }
        Some("custom") => (
            params.start_date.clone(),
            params.end_date.clone(),
            "Custom Range".to_string(),
        ),
        _ => (None, None, "All Time".to_string()),
    };

    // Get detailed question statistics
    let question_detail_stats = repository
        .get_question_detail_statistics(&question_id, start_date.as_deref(), end_date.as_deref())?
        .ok_or_else(|| AppError(color_eyre::eyre::eyre!("Question not found")))?;

    let context = QuestionDetailStatsContext {
        question_detail_stats,
        current_filter: filter_name,
    };

    let template = templates.get_template("admin_question_detail_stats.html")?;
    let rendered = template.render(&context)?;

    Ok(Html(rendered))
}
