use crate::{repository::RuleRepository, AppError};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Form, State},
    response::{Html, Redirect},
};
use axum_extra::extract::CookieJar;
use minijinja::Environment;
use regelator::auth::{
    clear_admin_cookie, create_admin_cookie, verify_admin_cookie, ADMIN_COOKIE_NAME,
};
use regelator::config::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    State(config): State<Config>,
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

        let cookie = create_admin_cookie(
            admin.id, 
            admin.username.clone(),
            &config.security.jwt_secret,
            config.session_duration(),
        ).map_err(|e| AppError(eyre::eyre!("Failed to create admin cookie: {}", e)))?;

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
    State(config): State<Config>,
    jar: CookieJar,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    let claims = verify_admin_cookie(cookie.value(), &config.security.jwt_secret)
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
    State(config): State<Config>,
    jar: CookieJar,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    verify_admin_cookie(cookie.value(), &config.security.jwt_secret)
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
    State(config): State<Config>,
    jar: CookieJar,
    Form(form_data): Form<ChangePasswordForm>,
) -> Result<Html<String>, AppError> {
    // Verify admin cookie and get admin info
    let cookie = jar
        .get(ADMIN_COOKIE_NAME)
        .ok_or_else(|| AppError(eyre::eyre!("No admin session")))?;

    let claims = verify_admin_cookie(cookie.value(), &config.security.jwt_secret)
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