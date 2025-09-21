use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub enum Role {
    USER,
    ADMIN,
}

impl Default for Role {
    fn default() -> Self {
        Role::USER
    }
}

impl From<Role> for String {
    fn from(role: Role) -> Self {
        match role {
            Role::USER => "USER".to_string(),
            Role::ADMIN => "ADMIN".to_string(),
        }
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "ADMIN" => Role::ADMIN,
            _ => Role::USER,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdatePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginResponse {
    pub auth_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author: UserResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub role: Role,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub enum UnifiedApiResponse<T> {
    Success(ApiResponse<T>),
    Error(ErrorResponse),
}
