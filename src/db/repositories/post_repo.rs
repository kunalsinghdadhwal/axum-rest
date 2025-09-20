use sqlx::{PgPool, Row};

use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{debug, info};
use uuid::Uuid;

use crate::model::model::{CreatePostRequest, Post, PostResponse, UpdatePostRequest, UserResponse};

pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        debug!("Creating new PostRepository");
        Self { pool }
    }

    pub async fn create_post(&self, post_data: CreatePostRequest, authod_id: Uuid) -> Result<Post> {
        let id = Uuid::new_v4();
        let now: DateTime<Utc> = Utc::now();

        info!("Creating new post with title: {}", post_data.title);

        let post = Post {
            id,
            title: post_data.title,
            content: post_data.content,
            author_id: authod_id,
            created_at: now,
            updated_at: now,
        };

        sqlx::query(
            r#"
                INSERT INTO posts (id, title, content, author_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6
            "#,
        )
        .bind(post.id.to_string())
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id.to_string())
        .bind(post.created_at.to_rfc3339())
        .bind(post.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        debug!("Post created with ID: {}", post.id);
        Ok(post)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Post>> {
        debug!("Finding post by ID: {}", id);

        let row = sqlx::query(
            r#"
                SELECT id, title, content, author_id, created_at, updated_at
                FROM posts
                WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let post = Post {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                    author_id: row.get("author_id"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                };
                debug!("Post found with id {}", id);
                Ok(Some(post))
            }
            None => {
                debug!("No post found with id {}", id);
                Ok(None)
            }
        }
    }

    pub async fn find_by_id_with_author(&self, id: Uuid) -> Result<Option<PostResponse>> {
        debug!("Finding post with author by ID: {}", id);

        let row = sqlx::query(
            r#"
                SELECT 
                    p.id as post_id, p.title, p.content, p.author_id, p.created_at as post_created_at, p.updated_at as post_updated_at,
                    u.id as user_id, u.name as user_name, u.email as user_email, u.created_at as user_created_at, u.updated_at as user_updated_at
                FROM posts p
                JOIN users u ON p.author_id = u.id
                WHERE p.id = $1
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let author = UserResponse {
                    id: row.get("user_id"),
                    name: row.get("user_name"),
                    email: row.get("user_email"),
                    created_at: row.get("user_created_at"),
                    updated_at: row.get("user_updated_at"),
                };

                let post_response = PostResponse {
                    id: row.get("id"),
                    title: row.get("title"),
                    content: row.get("content"),
                    author,
                    created_at: row.get("post_created_at"),
                    updated_at: row.get("post_updated_at"),
                };

                debug!("Post with author found with id {}", id);
                Ok(Some(post_response))
            }
            None => {
                debug!("No post found with id {}", id);
                Ok(None)
            }
        }
    }

    pub async fn find_by_author(&self, authod_id: Uuid) -> Result<Vec<Post>> {
        debug!("Finding posts by author ID: {}", authod_id);

        let rows = sqlx::query(
            r#"
                SELECT id, title, content, author_id, created_at, updated_at
                FROM posts
                WHERE author_id = $1
                ORDER BY created_at DESC   
            "#,
        )
        .bind(authod_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let posts: Result<Vec<Post>> = rows
            .into_iter()
            .map(|row| {
                Ok(Post {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    title: row.get("title"),
                    content: row.get("content"),
                    author_id: Uuid::parse_str(&row.get::<String, _>("author_id"))?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                        .with_timezone(&Utc),
                })
            })
            .collect();

        posts
    }

    pub async fn update_post(
        &self,
        id: Uuid,
        authod_id: Uuid,
        update_data: UpdatePostRequest,
    ) -> Result<Option<Post>> {
        debug!("Updating post ID: {}", id);

        let existing_post = self.find_by_id(id).await?;

        if existing_post.is_none() {
            debug!("No post found with id {}", id);
            return Ok(None);
        }

        let existing_post = existing_post.unwrap();

        if existing_post.author_id != authod_id {
            anyhow::bail!("Unauthorized: You can only update your own posts");
        }

        let updated_title = update_data
            .title
            .unwrap_or(existing_post.title)
            .trim()
            .to_string();
        let updated_content = update_data
            .content
            .unwrap_or(existing_post.content)
            .trim()
            .to_string();
        let now: DateTime<Utc> = Utc::now();

        sqlx::query(
            r#"
                UPDATE posts
                SET title = $1, content = $2, updated_at = $3
                WHERE id = $4
            "#,
        )
        .bind(&updated_title)
        .bind(&updated_content)
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        let updated_post = Post {
            id,
            title: updated_title,
            content: updated_content,
            author_id: authod_id,
            created_at: existing_post.created_at,
            updated_at: now,
        };

        debug!("Post updated with ID: {}", id);
        Ok(Some(updated_post))
    }

    pub async fn delete_post(&self, id: Uuid, authod_id: Uuid) -> Result<bool> {
        debug!("Deleting post ID: {}", id);

        let existing_post = self.find_by_id(id).await?;

        if existing_post.is_none() {
            debug!("No post found with id {}", id);
            return Ok(false);
        }

        let existing_post = existing_post.unwrap();

        if existing_post.author_id != authod_id {
            anyhow::bail!("Unauthorized: You can only delete your own posts");
        }

        let result = sqlx::query(
            r#"
                DELETE FROM posts
                WHERE id = $1
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            debug!("No post deleted with id {}", id);
            Ok(false)
        } else {
            debug!("Post deleted with ID: {}", id);
            Ok(true)
        }
    }

    pub async fn get_all_posts(&self) -> Result<Vec<PostResponse>> {
        debug!("Retrieving all posts");

        let rows = sqlx::query(
            r#"
                SELECT 
                    p.id, p.title, p.content, p.author_id, p.created_at, p.updated_at,
                    u.name as author_name, u.email as author_email, u.created_at as author_created_at, u.updated_at as author_updated_at
                FROM posts p
                JOIN users u ON p.author_id = u.id
                ORDER BY p.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let posts = rows
            .into_iter()
            .map(|row| {
                let author = UserResponse {
                    id: Uuid::parse_str(&row.get::<String, _>("author_id"))?,
                    name: row.get("author_name"),
                    email: row.get("author_email"),
                    created_at: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("author_created_at"),
                    )?
                    .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("author_updated_at"),
                    )?
                    .with_timezone(&Utc),
                };

                Ok(PostResponse {
                    id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                    title: row.get("title"),
                    content: row.get("content"),
                    author,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?
                        .with_timezone(&Utc),
                })
            })
            .collect();

        posts
    }
}
