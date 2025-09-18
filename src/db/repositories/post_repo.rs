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

        if post_data.title.trim().is_empty() {
            anyhow::bail!("Title cannot be empty");
        }

        if post_data.content.trim().is_empty() {
            anyhow::bail!("Content cannot be empty");
        }


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
            "#
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
            "#
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let post = Post {
                    id: Uuid::parse_str(row.get::<String, _>("id").as_str())?,
                    title: row.get("title"),
                    content: row.get("content"),
                    author_id: Uuid::parse_str(row.get::<String, _>("author_id").as_str())?,
                    created_at: DateTime::parse_from_rfc3339(row.get::<String, _>("created_at").as_str())?.with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(row.get::<String, _>("updated_at").as_str())?.with_timezone(&Utc),
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


    pub async fn find_by_id_with_author()
}