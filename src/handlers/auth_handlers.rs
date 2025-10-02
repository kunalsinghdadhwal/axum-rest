use crate::{
    helpers::resend::{ResendClient, verify_email_template},
    model::{
        VerifyEmailQuery,
        model::{
            CreateUserRequest, LoginRequest, LoginResponse, Role, UpdatePasswordRequest,
            UpdateUserRequest, UserResponse,
        },
    },
};
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use axum_extra::extract::cookie::Cookie;
use mailchecker::is_valid;
use resend_rs::types::CreateEmailBaseOptions;
use sqlx::PgPool;
use std::{
    env,
    sync::{Arc, LazyLock},
};
use time::Duration;
use utoipa;
use uuid::Uuid;

use crate::db::repositories::user_repo::UserRepository;
use crate::helpers::auth::AuthHelper;
use crate::helpers::middleware::check_admin_role;
use crate::helpers::response::{
    CookieResponse, UnifiedResponse, error_response_generic, error_response_with_cookies,
    not_found_response_generic, sql_error_generic, sql_error_response_with_cookies,
    success_response, success_response_with_cookies,
};
use crate::helpers::validation::{strong_password, validate_user_registration};
use tracing::{error, info};

static RESEND_CLIENT: LazyLock<ResendClient> = LazyLock::new(|| ResendClient::new());

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
            let user_email = user.email.clone(); // Clone email before moving user
            let user_name = user.name.clone();

            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                role: user.role,
                email_verified: user.email_verified,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            let verification_token = AuthHelper::generate_email_verification_token(user.id);
            let base_url = env::var("BASE_URL").unwrap_or_else(|_| "localhost:3000".to_string());
            // Send verification email
            let verification_link = format!(
                "http://{}/auth/verify-email?token={}",
                base_url, verification_token
            );

            // Send verification email using Resend
            let from = "AXUM-REST <onboarding@resend.dev>";
            let to = [user_email];
            let subject = "Verify your email address";

            let email = CreateEmailBaseOptions::new(from, to, subject)
                .with_html(&verify_email_template(&user_name, &verification_link));

            match RESEND_CLIENT.resend.emails.send(email).await {
                Ok(response) => {
                    info!("Verification email sent: {:?}", response);
                }
                Err(e) => {
                    error!("Failed to send verification email: {:?}", e);
                }
            }
            success_response(
                "Registration Complete, Check Email for Verification Link".to_string(),
                user_response,
            )
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
                role: user.role,
                email_verified: user.email_verified,
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

    // Validate name
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

    // Validate email if provided
    if let Some(email) = &payload.email {
        if !is_valid(email) {
            return error_response_generic(
                "Update Failed".to_string(),
                "Please provide a valid email address".to_string(),
            );
        }
    }

    match repo.update_user(user_id, payload.clone()).await {
        Ok((Some(user), email_updated)) => {
            let user_response = UserResponse {
                id: user.id,
                name: user.name.clone(),
                email: user.email.clone(),
                role: user.role,
                email_verified: user.email_verified,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            // Send verification email only if email changed
            if email_updated {
                let verification_token = AuthHelper::generate_email_verification_token(user.id);
                let base_url =
                    std::env::var("BASE_URL").unwrap_or_else(|_| "localhost:3000".to_string());
                let verification_link = format!(
                    "http://{}/auth/verify-email?token={}",
                    base_url, verification_token
                );

                let from = "AXUM-REST <onboarding@resend.dev>";
                let to = [user_response.email.clone()];
                let subject = "Verify your email address";

                let email = CreateEmailBaseOptions::new(from, &to, subject).with_html(
                    &verify_email_template(&user_response.name, &verification_link),
                );

                match RESEND_CLIENT.resend.emails.send(email).await {
                    Ok(response) => info!("Verification email sent: {:?}", response),
                    Err(e) => error!("Failed to send verification email: {:?}", e),
                }
            }

            success_response("Profile Updated".to_string(), user_response)
        }
        Ok((None, _)) => not_found_response_generic("User not found".to_string()),
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

    let user_id = user.id;

    match AuthHelper::verify_password(&payload.password, &user.password) {
        Ok(true) => {
            match repo.is_verified(user_id).await {
                Ok(true) => {}
                Ok(false) => {
                    return error_response_with_cookies(
                        "Login Failed".to_string(),
                        "Email verification required. Please verify your email before logging in."
                            .to_string(),
                    );
                }
                Err(e) => {
                    error!("Email verification check error: {:?}", e);
                    return error_response_with_cookies(
                        "Login Failed".to_string(),
                        "Unable to verify email status".to_string(),
                    );
                }
            }

            let tokens = match AuthHelper::generate_token(user.id, user.role.clone()) {
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
                role: user.role,
                email_verified: user.email_verified,
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

/// Get all users (Admin only)
#[utoipa::path(
    get,
    path = "/admin/users",
    responses(
        (status = 200, description = "Users retrieved successfully", body = inline(crate::helpers::response::ApiSuccessResponse<Vec<UserResponse>>)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 403, description = "Forbidden - Admin access required", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Administration"
)]
pub async fn get_all_users_admin(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Extension(user_role): Extension<Role>,
) -> UnifiedResponse<Vec<UserResponse>> {
    info!(
        "Handler: Admin getting all users, requested by user_id: {:?}",
        user_id
    );

    // Check if user has admin role
    if let Err((_, json_response)) = check_admin_role(&user_role) {
        let error_resp = json_response.0;
        return error_response_generic(error_resp.error, error_resp.message);
    }

    let repo = UserRepository::new((*pool).clone());

    match repo.get_all_users().await {
        Ok(users) => {
            info!("Retrieved {} users for admin", users.len());
            success_response("Users Retrieved".to_string(), users)
        }
        Err(e) => {
            error!("Handler: Database error: {:?}", e);
            sql_error_generic(e, "Error fetching users")
        }
    }
}

/// Delete user account (Self or Admin)
#[utoipa::path(
    delete,
    path = "/auth/profile",
    responses(
        (status = 200, description = "User account deleted successfully", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
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
pub async fn delete_user_account(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
) -> CookieResponse<String> {
    info!(
        "Handler: User deleting their own account, user_id: {:?}",
        user_id
    );

    let repo = UserRepository::new((*pool).clone());

    match repo.delete_user(user_id).await {
        Ok(true) => {
            info!("User account deleted successfully: {}", user_id);

            // Create expired cookies to clear them after account deletion
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
                "Account Deleted".to_string(),
                "Your account has been permanently deleted".to_string(),
                vec![auth_cookie, refresh_cookie],
            )
        }
        Ok(false) => error_response_with_cookies(
            "Deletion Failed".to_string(),
            "User account not found".to_string(),
        ),
        Err(e) => {
            error!("Database error during user deletion: {:?}", e);
            sql_error_response_with_cookies(e, "Unable to delete user account")
        }
    }
}

/// Delete user account by ID (Admin only)
#[utoipa::path(
    delete,
    path = "/admin/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID to delete")
    ),
    responses(
        (status = 200, description = "User account deleted successfully", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 403, description = "Forbidden - Admin access required", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "User not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Administration"
)]
pub async fn delete_user_admin(
    State(pool): State<Arc<PgPool>>,
    Extension(admin_user_id): Extension<Uuid>,
    Extension(user_role): Extension<Role>,
    Path(target_user_id): Path<Uuid>,
) -> UnifiedResponse<String> {
    info!(
        "Handler: Admin deleting user account, admin_id: {:?}, target_user_id: {:?}",
        admin_user_id, target_user_id
    );

    // Check if user has admin role
    if let Err((_, json_response)) = check_admin_role(&user_role) {
        let error_resp = json_response.0;
        return error_response_generic(error_resp.error, error_resp.message);
    }

    // Prevent admin from deleting their own account through this endpoint
    if admin_user_id == target_user_id {
        return error_response_generic(
            "Invalid Operation".to_string(),
            "Admins cannot delete their own account through this endpoint. Use the profile deletion endpoint instead.".to_string(),
        );
    }

    let repo = UserRepository::new((*pool).clone());

    match repo.delete_user(target_user_id).await {
        Ok(true) => {
            info!(
                "Admin {} successfully deleted user account: {}",
                admin_user_id, target_user_id
            );
            success_response(
                "User Deleted".to_string(),
                format!(
                    "User account {} has been permanently deleted",
                    target_user_id
                ),
            )
        }
        Ok(false) => not_found_response_generic("User not found".to_string()),
        Err(e) => {
            error!("Database error during admin user deletion: {:?}", e);
            sql_error_generic(e, "Unable to delete user account")
        }
    }
}

/// Verify user email address
#[utoipa::path(
    get,
    path = "/auth/verify-email",
    params(
        ("token" = String, Query, description = "Email verification token")
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
        (status = 400, description = "Invalid or expired token", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "User not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    tag = "Authentication"
)]
pub async fn verify_email(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<VerifyEmailQuery>,
) -> UnifiedResponse<String> {
    let user_id = match AuthHelper::extract_user_id_from_token(&query.token) {
        Ok(id) => id,
        Err(_) => {
            return error_response_generic(
                "Invalid Token".to_string(),
                "The email verification token is invalid or has expired".to_string(),
            );
        }
    };

    let repo = UserRepository::new((*pool).clone());

    match repo.verify_email(user_id).await {
        Ok(Some(_)) => success_response(
            "Email Verified".to_string(),
            "Your email has been successfully verified".to_string(),
        ),
        Ok(None) => error_response_generic(
            "Verification Failed".to_string(),
            "User not found or already verified".to_string(),
        ),
        Err(e) => {
            error!("Database error: {:?}", e);
            sql_error_generic(e, "Unable to verify email")
        }
    }
}
