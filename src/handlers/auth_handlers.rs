use crate::{
    helpers::validation::strong_password,
    model::model::{
        CreateUserRequest, LoginRequest, LoginResponse, UpdatePasswordRequest, UpdateUserRequest,
        UserResponse,
    },
};
use axum::{
    Json,
    extract::{Extension, State},
};
use mailchecker::is_valid;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::repositories::user_repo::UserRepository;
use crate::helpers::auth::AuthHelper;
use crate::helpers::response::{
    UnifiedResponse, error_response_generic, not_found_response_generic, sql_error_generic,
    success_response,
};
use crate::helpers::validation::validate_user_registration;
use tracing::{error, info};

pub async fn register_user(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<CreateUserRequest>,
) -> UnifiedResponse<UserResponse> {
    info!("Handler: Registering user: {:?}", payload.email);

    if let Err(validation_errors) = validate_user_registration(&payload) {
        return error_response_generic("Validation Error".to_string(), validation_errors);
    }

    if !is_valid(&payload.email) {
        return error_response_generic(
            "Invalid email format".to_string(),
            "Email format is invalid".to_string(),
        );
    }

    let repo = UserRepository::new((*pool).clone());

    match repo.find_by_email(&payload.email).await {
        Ok(Some(_)) => {
            return error_response_generic(
                "User already exists".to_string(),
                "Email already in use".to_string(),
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
                "Internal Server Error".to_string(),
                "Error hashing password".to_string(),
            );
        }
    };

    match repo.create_user(payload.clone(), hashed_password).await {
        Ok(user) => {
            let user_name = user.name.clone();
            let user_response = UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            success_response(
                format!("User {} registered successfully", user_name),
                user_response,
            )
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            sql_error_generic(e, "Error creating user")
        }
    }
}

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

            success_response(format!("User profile fetched successfully"), user_response)
        }
        Ok(None) => not_found_response_generic("User not found".to_string()),
        Err(e) => {
            error!("Handler: Database error: {:?}", e);
            sql_error_generic(e, "Error fetching user profile")
        }
    }
}

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
                "Invalid name".to_string(),
                "Name cannot be empty".to_string(),
            );
        }
    } else {
        return error_response_generic("Invalid name".to_string(), "Name is required".to_string());
    }

    if let Some(email) = &payload.email {
        if !is_valid(email) {
            return error_response_generic(
                "Invalid email format".to_string(),
                "Email format is invalid".to_string(),
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

            success_response(format!("User profile updated successfully"), user_response)
        }
        Ok(None) => not_found_response_generic("User not found".to_string()),
        Err(e) => {
            error!("Handler: Database error: {:?}", e);
            sql_error_generic(e, "Error updating user profile")
        }
    }
}

pub async fn login_user(
    State(pool): State<Arc<PgPool>>,
    Json(payload): Json<LoginRequest>,
) -> UnifiedResponse<LoginResponse> {
    info!("Handler: Logging in user: {:?}", payload.email);

    let repo = UserRepository::new((*pool).clone());

    let user = match repo.find_by_email(&payload.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return error_response_generic(
                "Invalid credentials".to_string(),
                "Email or password is incorrect".to_string(),
            );
        }
        Err(e) => {
            error!("Database error: {:?}", e);
            return sql_error_generic(e, "Error fetching user");
        }
    };

    match AuthHelper::verify_password(&payload.password, &user.password) {
        Ok(true) => {
            let tokens = match AuthHelper::generate_token(user.id) {
                Ok(t) => t,
                Err(e) => {
                    error!("Token generation error: {:?}", e);
                    return error_response_generic(
                        "Internal Server Error".to_string(),
                        "Error generating token".to_string(),
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
                auth_token,
                refresh_token,
            };

            success_response("Login Successfull".to_string(), login_response)
        }
        Ok(false) => error_response_generic(
            "Invalid credentials".to_string(),
            "Email or password is incorrect".to_string(),
        ),
        Err(e) => {
            error!("Password verification error: {:?}", e);
            error_response_generic(
                "Internal Server Error".to_string(),
                "Error verifying password".to_string(),
            )
        }
    }
}
