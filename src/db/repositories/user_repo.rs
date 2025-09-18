use anyhow::Result;
use chrono::{DateTime, Utc};
use mailchecker::is_valid;
use sqlx::{PgPool, Row};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    helpers::validation::strong_password,
    model::model::{
        CreateUserRequest, UpdatePasswordRequest, UpdateUserRequest, User, UserResponse,
    },
};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        debug!("Creating UserRepository");
        Self { pool }
    }

    pub async fn create_user(
        &self,
        user_data: CreateUserRequest,
        hashed_password: String,
    ) -> Result<User> {
        let id = Uuid::new_v4();
        let now: DateTime<Utc> = Utc::now();

        info!("Creating new user with email: {}", user_data.email);

        if !is_valid(&user_data.email) {
            anyhow::bail!("Invalid email");
        } else if !strong_password(&user_data.password) {
            anyhow::bail!("Strong password required");
        } else {
            let user = User {
                id,
                name: user_data.name,
                email: user_data.email,
                password: hashed_password,
                created_at: now,
                updated_at: now,
            };

            sqlx::query(
                r#"
                INSERT INTO users (id, name, email, password, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(id.to_string())
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(user.created_at.to_rfc3339())
            .bind(user.updated_at.to_rfc3339())
            .execute(&self.pool)
            .await?;

            debug!("User created with ID: {}", id);
            Ok(user)
        }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        debug!("Finding user by ID: {}", id);
        let row = sqlx::query(
            r#"
            SELECT id, name, email, password, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    name: row.get("name"),
                    email: row.get("email"),
                    password: row.get("password"),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                        .with_timezone(&Utc),
                };

                debug!("User found with ID: {}", id);
                Ok(Some(user))
            }
            None => {
                debug!("No user found with ID: {}", id);
                Ok(None)
            }
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        debug!("Finding user by email: {}", email);
        let row = sqlx::query(
            r#"
            SELECT id, name, email, password, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    name: row.get("name"),
                    email: row.get("email"),
                    password: row.get("password"),
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                        .with_timezone(&Utc),
                };

                debug!("User found with email: {}", email);
                Ok(Some(user))
            }
            None => {
                debug!("No user found with email: {}", email);
                Ok(None)
            }
        }
    }

    pub async fn update_user(
        &self,
        id: Uuid,
        update_data: UpdateUserRequest,
    ) -> Result<Option<User>> {
        info!("Updating user with ID: {}", id);

        let existing_user = self.find_by_id(id).await?;
        if existing_user.is_none() {
            return Ok(None);
        }

        let mut user = existing_user.unwrap();
        let mut updated = false;

        if let Some(name) = update_date.name {
            user.name = name;
            updated = true;
        }

        if let Some(email) = update_date.email {
            if !is_valid(&email) {
                anyhow::bail!("Invalid email");
            }
            user.email = email;
            updated = true;
        }

        if updated {
            user.updated_at = Utc::now();

            sqlx::query(
                r#"
                UPDATE users
                SET name = $1, email = $2, updated_at = $3
                WHERE id = $4
                "#,
            )
            .bind(&user.name)
            .bind(&user.email)
            .bind(user.updated_at.to_rfc3339())
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await?;

            debug!("User updated with ID: {}", user.id);
        }

        Ok(Some(user))
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        update_data: UpdatePasswordRequest,
    ) -> Result<Option<User>> {
        info!("Updating password for user ID: {}", id);

        let existing_user = self.find_by_id(id).await?;
        if existing_user.is_none() {
            return Ok(None);
        }

        let mut user = existing_user.unwrap();
        let mut updated = false;
        if !strong_password(&update_data.new_password) {
            anyhow::bail!("Strong password required");
        }

        if update_data.old_password != user.password {
            anyhow::bail!("Old password does not match");
        } else {
            user.password = update_data.new_password;
            updated = true;
        }

        if updated {
            user.updated_at = Utc::now();

            sqlx::query(
                r#"
                UPDATE users
                SET password = $1, updated_at = $2
                WHERE id = $3
                "#,
            )
            .bind(&user.password)
            .bind(user.updated_at.to_rfc3339())
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await?;

            debug!("Password updated for user ID: {}", user.id);
        }

        Ok(Some(user))
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<bool> {
        info!("Deleting user with ID: {}", id);
        let result = sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            debug!("No user found to delete with ID: {}", id);
            Ok(false)
        } else {
            debug!("User deleted with ID: {}", id);
            Ok(true)
        }
    }

    pub async fn get_all_users(&self) -> Result<Vec<UserResponse>> {
        debug!("Fetching all users");
        let rows = sqlx::query(
            r#"
            SELECT id, name, email, created_at, updated_at
            FROM users
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let users: Vec<UserResponse> = rows
            .into_iter()
            .map(|row| UserResponse {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                name: row.get("name"),
                email: row.get("email"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect();

        debug!("Fetched {} users", users.len());
        Ok(users)
    }
}
