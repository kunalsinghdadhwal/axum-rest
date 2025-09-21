use crate::model::model::{
    CreateUserRequest, LoginRequest, LoginResponse, UpdatePasswordRequest, UpdateUserRequest,
    UserResponse,
};
use axum::{
    Json,
    extract::{Extension, State},
};
use axum_extra::extract::cookie::Cookie;
use mailchecker::is_valid;
use sqlx::PgPool;
use std::sync::Arc;
use time::Duration;
use utoipa;
use uuid::Uuid;

use crate::db::repositories::user_repo::UserRepository;
use crate::helpers::auth::AuthHelper;
use crate::helpers::response::{
    CookieResponse, UnifiedResponse, error_response_generic, error_response_with_cookies,
    not_found_response_generic, sql_error_generic, sql_error_response_with_cookies,
    success_response, success_response_with_cookies,
};
use crate::helpers::validation::{strong_password, validate_user_registration};
use tracing::{error, info};

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User registered successfully", body = inline(crate::helpers::response::ApiSuccessResponse<UserResponse>)),
        (status = 400, description = "Validation error", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 409, description = "User already exists", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    tag = "Authentication"
)]
pub async fn register_user(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<CreateUserRequest>,
) -> UnifiedResponse<UserResponse> {
    info!("Handler: Registering user: {:?}", payload.email);

    if let Err(validation_errors) = validate_user_registration(&payload) {
        return error_response_generic("Registration Failed".to_string(), validation_errors);
    }

    if !is_valid(&payload.email) {
        return error_response_generic(
            "Invalid Email".to_string(),
            "Please provide a valid email address".to_string(),
        );
    }

    let repo = UserRepository::new((*pool).clone());

    match repo.find_by_email(&payload.email).await {
        Ok(Some(_)) => {
            return error_response_generic(
                "Account Exists".to_string(),
                "An account with this email already exists".to_string(),
            );
        }
        Ok(None) => {}
        Err(e) => {
            error!("Database error: {:?}", e);
            return sql_error_generic(e, "Error checking existing user");
        }
    }

    let hashed_password = match AuthHelper::hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing error: {:?}", e);
            return error_response_generic(
                "Registration Failed".to_string(),
                "Unable to process password securely".to_string(),
            );
        }
    };

    match repo.create_user(payload.clone(), hashed_password).await {
        Ok(user) => {
            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            success_response("Registration Complete".to_string(), user_response)
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            sql_error_generic(e, "Error creating user")
        }
    }
}

/// Get user profile
#[utoipa::path(
    get,
    path = "/auth/profile",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = inline(crate::helpers::response::ApiSuccessResponse<UserResponse>)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "User not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn get_profile(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
) -> UnifiedResponse<UserResponse> {
    info!("Handler: Fetching profile for user_id: {:?}", user_id);

    let repo = UserRepository::new((*pool).clone());

    match repo.find_by_id(user_id).await {
        Ok(Some(user)) => {
            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            success_response("Profile Retrieved".to_string(), user_response)
        }
        Ok(None) => not_found_response_generic("User not found".to_string()),
        Err(e) => {
            error!("Handler: Database error: {:?}", e);
            sql_error_generic(e, "Error fetching user profile")
        }
    }
}

/// Update user profile
#[utoipa::path(
    put,
    path = "/auth/profile",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User profile updated successfully", body = inline(crate::helpers::response::ApiSuccessResponse<UserResponse>)),
        (status = 400, description = "Validation error", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "User not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn update_profile(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> UnifiedResponse<UserResponse> {
    info!("Handler: Updating profile for user_id: {:?}", user_id);

    let repo = UserRepository::new((*pool).clone());

    let mut update_data = payload.clone();

    if let Some(name) = &payload.name {
        if name.trim().is_empty() {
            return error_response_generic(
                "Update Failed".to_string(),
                "Name cannot be empty".to_string(),
            );
        }
    } else {
        return error_response_generic("Update Failed".to_string(), "Name is required".to_string());
    }

    if let Some(email) = &payload.email {
        if !is_valid(email) {
            return error_response_generic(
                "Update Failed".to_string(),
                "Please provide a valid email address".to_string(),
            );
        }
    }

    match repo.update_user(user_id, update_data).await {
        Ok(Some(user)) => {
            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            success_response("Profile Updated".to_string(), user_response)
        }
        Ok(None) => not_found_response_generic("User not found".to_string()),
        Err(e) => {
            error!("Handler: Database error: {:?}", e);
            sql_error_generic(e, "Error updating user profile")
        }
    }
}

/// User login
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful - returns JWT token and sets HTTP-only auth cookies (auth_token: 24h, refresh_token: 7d)", body = inline(crate::helpers::response::ApiSuccessResponse<LoginResponse>)),
        (status = 400, description = "Invalid credentials", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    tag = "Authentication"
)]
pub async fn login_user(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<LoginRequest>,
) -> CookieResponse<LoginResponse> {
    info!("Handler: Logging in user: {:?}", payload.email);

    let repo = UserRepository::new((*pool).clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return error_response_with_cookies(
                "Login Failed".to_string(),
                "Invalid email or password".to_string(),
            );
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return sql_error_response_with_cookies(e, "Unable to verify credentials");
        }
    };

    match AuthHelper::verify_password(&payload.password, &user.password) {
        Ok(true) => {
            let tokens = match AuthHelper::generate_token(user.id) {
                Ok(t) => t,
                Err(e) => {
                    error!("Token generation error: {:?}", e);
                    return error_response_with_cookies(
                        "Login Failed".to_string(),
                        "Unable to create authentication session".to_string(),
                    );
                }
            };

            let (auth_token, refresh_token) = tokens;

            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            let login_response = LoginResponse {
                user: user_response,
                auth_token: auth_token.clone(),
                refresh_token: refresh_token.clone(),
            };

            // Create cookies for auth tokens
            let auth_cookie = Cookie::build(("auth_token", auth_token))
                .path("/")
                .max_age(Duration::hours(24)) // 24 hours
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .build();

            let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
                .path("/")
                .max_age(Duration::days(7)) // 7 days
                .http_only(true)
                .secure(false) // Set to true in production with HTTPS
                .same_site(axum_extra::extract::cookie::SameSite::Lax)
                .build();

            success_response_with_cookies(
                "Login Successful".to_string(),
                login_response,
                vec![auth_cookie, refresh_cookie],
            )
        }
        Ok(false) => error_response_with_cookies(
            "Login Failed".to_string(),
            "Invalid email or password".to_string(),
        ),
        Err(e) => {
            error!("Password verification error: {:?}", e);
            error_response_with_cookies(
                "Login Failed".to_string(),
                "Unable to verify credentials".to_string(),
            )
        }
    }
}

/// User logout
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful - clears HTTP-only authentication cookies", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn logout_user() -> CookieResponse<String> {
    info!("Handler: Logging out user");

    // Create expired cookies to clear them
    let auth_cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .max_age(Duration::seconds(-1)) // Expired
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(Duration::seconds(-1)) // Expired
        .http_only(true)
        .secure(false) // Set to true in production with HTTPS
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .build();

    success_response_with_cookies(
        "Logout Successful".to_string(),
        "Authentication session ended".to_string(),
        vec![auth_cookie, refresh_cookie],
    )
}

/// Change user password
#[utoipa::path(
    put,
    path = "/auth/change-password",
    request_body = UpdatePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
        (status = 400, description = "Validation error", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "User not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Authentication"
)]
pub async fn change_password(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> UnifiedResponse<String> {
    info!("Handler: Changing password for user_id: {:?}", user_id);

    // Validate new password strength
    if !strong_password(&payload.new_password) {
        return error_response_generic(
            "Weak Password".to_string(),
            "Password must be at least 8 characters long with mixed case, numbers, and special characters".to_string(),
        );
    }

    // Check if new password is same as old password
    if payload.old_password == payload.new_password {
        return error_response_generic(
            "Invalid Password".to_string(),
            "New password must be different from current password".to_string(),
        );
    }

    let repo = UserRepository::new((*pool).clone());

    // Get current user to verify old password
    let user = match repo.find_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return error_response_generic(
                "User Not Found".to_string(),
                "User account not found".to_string(),
            );
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return sql_error_generic(e, "Unable to retrieve user account");
        }
    };

    // Verify old password
    match AuthHelper::verify_password(&payload.old_password, &user.password) {
        Ok(true) => {
            // Old password is correct, proceed to update
        }
        Ok(false) => {
            return error_response_generic(
                "Incorrect Password".to_string(),
                "Current password is incorrect".to_string(),
            );
        }
        Err(e) => {
            error!("Password verification error: {:?}", e);
            return error_response_generic(
                "Password Change Failed".to_string(),
                "Unable to verify current password".to_string(),
            );
        }
    }

    // Hash new password
    let hashed_new_password = match AuthHelper::hash_password(&payload.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing error: {:?}", e);
            return error_response_generic(
                "Password Change Failed".to_string(),
                "Unable to process new password securely".to_string(),
            );
        }
    };

    // Update password in database using the simpler change_password function
    match repo.change_password(user_id, hashed_new_password).await {
        Ok(Some(_)) => success_response(
            "Password Changed".to_string(),
            "Password has been updated successfully".to_string(),
        ),
        Ok(None) => error_response_generic(
            "Password Change Failed".to_string(),
            "User account not found".to_string(),
        ),
        Err(e) => {
            error!("Password update error: {:?}", e);
            sql_error_generic(e, "Unable to update password")
        }
    }
}

/// Home page with cookie authentication documentation
pub async fn home() -> axum::response::Html<String> {
    axum::response::Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Axum REST API Server</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background-color: #f5f5f5; }
        .container { background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .header { text-align: center; margin-bottom: 30px; }
        .section { margin-bottom: 25px; }
        .endpoint { background: #f8f9fa; padding: 15px; border-radius: 5px; margin: 10px 0; border-left: 4px solid #007bff; }
        .method { display: inline-block; padding: 4px 8px; border-radius: 3px; color: white; font-weight: bold; margin-right: 10px; }
        .post { background-color: #28a745; }
        .get { background-color: #007bff; }
        .put { background-color: #ffc107; color: black; }
        .delete { background-color: #dc3545; }
        .cookie-info { background: #e7f3ff; padding: 15px; border-radius: 5px; border-left: 4px solid #0066cc; }
        .warning { background: #fff3cd; padding: 10px; border-radius: 5px; border-left: 4px solid #ffc107; }
        h1 { color: #333; margin-bottom: 10px; }
        h2 { color: #007bff; border-bottom: 2px solid #007bff; padding-bottom: 10px; }
        code { background: #f1f1f1; padding: 2px 6px; border-radius: 3px; }
        ul { line-height: 1.6; }
        .docs-link { display: inline-block; background: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; margin: 10px 0; }
        .docs-link:hover { background: #0056b3; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Axum REST API Server</h1>
            <p>High-performance Rust web server with cookie-based authentication</p>
            <a href="/docs" class="docs-link">View OpenAPI Documentation</a>
        </div>

        <div class="section">
            <h2>Cookie Authentication</h2>
            <div class="cookie-info">
                <h3>Authentication Cookies:</h3>
                <ul>
                    <li><strong>auth_token</strong>: Main authentication cookie (24 hours expiry)</li>
                    <li><strong>refresh_token</strong>: Refresh authentication cookie (7 days expiry)</li>
                </ul>
                <p><strong>Security Features:</strong></p>
                <ul>
                    <li>HTTP-Only cookies (not accessible via JavaScript)</li>
                    <li>Secure flag for HTTPS environments</li>
                    <li>Automatic expiration handling</li>
                    <li>Cross-site request forgery protection</li>
                </ul>
            </div>
        </div>

        <div class="section">
            <h2>Authentication Endpoints</h2>
            
            <div class="endpoint">
                <span class="method post">POST</span><code>/auth/register</code>
                <p>Register a new user account</p>
            </div>
            
            <div class="endpoint">
                <span class="method post">POST</span><code>/auth/login</code>
                <p>Login user and set authentication cookies</p>
                <div class="warning">
                    <strong>Note:</strong> Login now sets both Bearer token (in response) and HTTP-only cookies for dual authentication support.
                </div>
            </div>
            
            <div class="endpoint">
                <span class="method post">POST</span><code>/auth/logout</code>
                <p>Logout user and clear authentication cookies</p>
            </div>
            
            <div class="endpoint">
                <span class="method get">GET</span><code>/auth/profile</code>
                <p>Get current user profile (requires authentication)</p>
            </div>
            
            <div class="endpoint">
                <span class="method put">PUT</span><code>/auth/profile</code>
                <p>Update user profile (requires authentication)</p>
            </div>
            
            <div class="endpoint">
                <span class="method put">PUT</span><code>/auth/change-password</code>
                <p>Change user password (requires authentication)</p>
            </div>
        </div>

        <div class="section">
            <h2>Post Management Endpoints</h2>
            
            <div class="endpoint">
                <span class="method get">GET</span><code>/posts</code>
                <p>Get all posts (public access)</p>
            </div>
            
            <div class="endpoint">
                <span class="method get">GET</span><code>/posts/{id}</code>
                <p>Get specific post by ID (public access)</p>
            </div>
            
            <div class="endpoint">
                <span class="method post">POST</span><code>/posts</code>
                <p>Create new post (requires authentication)</p>
            </div>
            
            <div class="endpoint">
                <span class="method get">GET</span><code>/posts/my</code>
                <p>Get current user's posts (requires authentication)</p>
            </div>
            
            <div class="endpoint">
                <span class="method put">PUT</span><code>/posts/{id}</code>
                <p>Update post (requires authentication & ownership)</p>
            </div>
            
            <div class="endpoint">
                <span class="method delete">DELETE</span><code>/posts/{id}</code>
                <p>Delete post (requires authentication & ownership)</p>
            </div>
        </div>

        <div class="section">
            <h2>Authentication Methods</h2>
            <div class="cookie-info">
                <h3>Two Authentication Options:</h3>
                <ol>
                    <li><strong>Bearer Token:</strong> Include <code>Authorization: Bearer {token}</code> header</li>
                    <li><strong>Cookies:</strong> Automatic authentication via HTTP-only cookies (recommended for web browsers)</li>
                </ol>
                <p>Both methods provide the same level of security and functionality.</p>
            </div>
        </div>

        <div class="section">
            <h2>Server Features</h2>
            <ul>
                <li>Graceful shutdown handling</li>
                <li>Comprehensive request/response tracing</li>
                <li>CORS support for cross-origin requests</li>
                <li>PostgreSQL database integration</li>
                <li>OpenAPI 3.0 documentation with Scalar UI</li>
                <li>JWT-based authentication with cookie support</li>
                <li>Input validation and error handling</li>
            </ul>
        </div>

        <div class="section">
            <div class="warning">
                <strong>Development Note:</strong> This server includes enhanced tracing for debugging. 
                Check the console logs for detailed request/response information.
            </div>
        </div>
    </div>
</body>
</html>
    "#.to_string())
}
