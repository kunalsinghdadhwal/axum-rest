use anyhow::Result;
use chrono::{DateTime, Utc};
use mailchecker::is_valid;
use sqlx::{PgPool, Row};
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    helpers::validation::strong_password,
    model::model::{
        CreateUserRequest, Role, UpdatePasswordRequest, UpdateUserRequest, User, UserResponse,
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
                role: Role::default(), // Default to USER role
                email_verified: false, // Default to false, requires verification
                created_at: now,
                updated_at: now,
            };

            sqlx::query(
                r#"
                INSERT INTO users (id, name, email, password, role, email_verified, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(id)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(&String::from(user.role.clone()))
            .bind(user.email_verified)
            .bind(user.created_at)
            .bind(user.updated_at)
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
            SELECT id, name, email, password, role, email_verified, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    name: row.get("name"),
                    email: row.get("email"),
                    password: row.get("password"),
                    role: Role::from(row.get::<&str, _>("role")),
                    email_verified: row.get("email_verified"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
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
            SELECT id, name, email, password, role, email_verified, created_at, updated_at
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
                    id: row.get("id"),
                    name: row.get("name"),
                    email: row.get("email"),
                    password: row.get("password"),
                    role: Role::from(row.get::<&str, _>("role")),
                    email_verified: row.get("email_verified"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
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
    ) -> Result<(Option<User>, bool)> {
        info!("Updating user with ID: {}", id);

        let mut email_updated = false;
        let existing_user = self.find_by_id(id).await?;
        if existing_user.is_none() {
            return Ok((None, false));
        }

        let mut user = existing_user.unwrap();

        if let Some(name) = update_data.name {
            user.name = name;
            user.updated_at = Utc::now();
        }

        if let Some(email) = update_data.email {
            if !is_valid(&email) {
                anyhow::bail!("Invalid email");
            }
            user.email = email;
            user.email_verified = false;
            email_updated = true;
        }

        sqlx::query(
            r#"
            UPDATE users
            SET name = $1, email = $2, email_verified = $3, updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(user.email_verified)
        .bind(user.updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok((Some(user), email_updated))
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
        let updated;
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
            .bind(user.updated_at)
            .bind(user.id)
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
        .bind(id)
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
            SELECT id, name, email, role, email_verified, created_at, updated_at
            FROM users
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let users: Vec<UserResponse> = rows
            .into_iter()
            .map(|row| UserResponse {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                role: Role::from(row.get::<&str, _>("role")),
                email_verified: row.get("email_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        debug!("Fetched {} users", users.len());
        Ok(users)
    }

    pub async fn change_password(
        &self,
        id: Uuid,
        new_hashed_password: String,
    ) -> Result<Option<User>> {
        info!("Changing password for user ID: {}", id);

        let existing_user = self.find_by_id(id).await?;
        if existing_user.is_none() {
            return Ok(None);
        }

        let mut user = existing_user.unwrap();
        user.password = new_hashed_password;
        user.updated_at = Utc::now();

        sqlx::query(
            r#"
            UPDATE users
            SET password = $1, updated_at = $2
            WHERE id = $3
            "#,
        )
        .bind(&user.password)
        .bind(user.updated_at)
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        debug!("Password changed for user ID: {}", user.id);
        Ok(Some(user))
    }

    pub async fn verify_email(&self, id: Uuid) -> Result<Option<User>> {
        info!("Verifying Email for User: {}", id);

        let existing_user = self.find_by_id(id).await?;
        if existing_user.is_none() {
            return Ok(None);
        }

        let user = existing_user.unwrap();

        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = TRUE, updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        let updated_user = self.find_by_id(id).await?;

        if updated_user.is_none() {
            debug!("Email Verified but error while fetching user {}", id);
            Ok(None)
        } else {
            debug!("Email verified for user ID: {}", user.id);
            Ok(updated_user)
        }
    }

    pub async fn is_verified(&self, id: Uuid) -> Result<bool> {
        debug!("Checking if user ID: {} is verified", id);
        let row = sqlx::query(
            r#"
            SELECT email_verified
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let verified: bool = row.get("email_verified");
                debug!("User ID: {} verified status: {}", id, verified);
                Ok(verified)
            }
            None => {
                debug!("No user found with ID: {}", id);
                Ok(false)
            }
        }
    }
}
