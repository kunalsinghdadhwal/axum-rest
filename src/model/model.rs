use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePasswordRequest {
    pub email: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: UserResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct UnifiedApiResponse<T> {
    Success(ApiResponse<T>),
    Error(ErrorResponse),
}
