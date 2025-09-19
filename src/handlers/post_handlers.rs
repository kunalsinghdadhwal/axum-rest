use crate::db::repositories::post_repo::PostRepository;
use crate::helpers::response::{
    UnifiedResponse, error_response_generic, not_found_response_generic, sql_error_generic,
    sql_error_response, success_response,
};
use crate::model::model::{self, CreatePostRequest, PostResponse, UpdatePostRequest};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde_json::{Value, error};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub async fn create_post(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreatePostRequest>,
) -> UnifiedResponse<PostResponse> {
    info!("Handler: Creating new post for user_id: {}", user_id);

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        error!("Validation error: Title and content cannot be empty");
        return error_response_generic(
            "Bad Request".to_string(),
            "Title and content cannot be empty".to_string(),
        );
    }

    let repo = PostRepository::new((*pool).clone());

    match repo.create_post(payload, user_id).await {
        Ok(post) => match repo.find_by_id_with_author(post.id).await {
            Ok(Some(post_response)) => success_response(
                format!("Post '{}' created successfully", post.title),
                post_response,
            ),
            Ok(None) => {
                error!("Post created but not found: {}", post.id);
                error_response_generic(
                    "Internal Server Error".to_string(),
                    "Post created but failed to retrieve with author info".to_string(),
                )
            }
            Err(e) => {
                error!(
                    "Handler: Failed to retrieve created post with author info: {}",
                    e
                );
                sql_error_generic(e, "Failed to retrieve created post with author info")
            }
        },
        Err(e) => {
            error!("Handler: Failed to create post: {}", e);
            sql_error_generic(e, "Failed to create post")
        }
    }
}

pub async fn delete_post(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> UnifiedResponse<Value> {
    info!(
        "Handler: Deleting post with id: {} for user_id: {}",
        id, user_id
    );

    let repo = PostRepository::new((*pool).clone());

    match repo.delete_post(id, user_id).await {
        Ok(true) => success_response(
            format!("Post with id '{}' deleted successfully", id),
            Value::Null,
        ),
        Ok(false) => {
            error!("Post not found or unauthorized deletion attempt: {}", id);
            not_found_response_generic(
                "Post not found or you are not authorized to delete it".to_string(),
            )
        }
        Err(e) => {
            error!("Handler: Failed to delete post: {}", e);
            sql_error_generic(e, "Failed to delete post")
        }
    }
}

pub async fn update_post(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePostRequest>,
) -> UnifiedResponse<PostResponse> {
    info!(
        "Handler: Updating post with id: {} for user_id: {}",
        id, user_id
    );

    let repo = PostRepository::new((*pool).clone());

    match repo.update_post(id, user_id, payload).await {
        Ok(Some(post)) => match repo.find_by_id_with_author(post.id).await {
            Ok(Some(post_response)) => success_response(
                format!("Post {} updated successfully", post.title),
                post_response,
            ),
            Ok(None) => error_response_generic(
                "Internal Server Error".to_string(),
                "Post updated but failed to retrieve with author info".to_string(),
            ),
            Err(e) => {
                error!(
                    "Handler: Failed to retrieve updated post with author info: {}",
                    e
                );
                sql_error_generic(e, "Failed to retrieve updated post with author info")
            }
        },
        Ok(None) => {
            error!("Post not found or unauthorized update attempt: {}", id);
            not_found_response_generic(
                "Post not found or you are not authorized to update it".to_string(),
            )
        }
        Err(e) => {
            error!("Handler: Failed to update post: {}", e);
            sql_error_generic(e, "Failed to update post")
        }
    }
}

pub async fn get_all_posts(State(pool): State<Arc<PgPool>>) -> UnifiedResponse<Vec<PostResponse>> {
    info!("Handler: Retrieving all posts");

    let repo = PostRepository::new((*pool).clone());

    match repo.get_all_posts().await {
        Ok(posts) => success_response(format!("Retrieved {} posts", posts.len()), posts),
        Err(e) => {
            error!("Handler: Failed to retrieve posts: {}", e);
            sql_error_generic(e, "Failed to retrieve posts")
        }
    }
}

pub async fn get_user_posts(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
) -> UnifiedResponse<Vec<model::Post>> {
    info!("Handler: Retrieving posts for user_id: {}", user_id);

    let repo = PostRepository::new((*pool).clone());

    match repo.find_by_author(user_id).await {
        Ok(posts) => success_response(format!("Retrieved {} posts", posts.len()), posts),
        Err(e) => {
            error!("Handler: Failed to retrieve user posts: {}", e);
            sql_error_generic(e, "Failed to retrieve user posts")
        }
    }
}

pub async fn get_post(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> UnifiedResponse<PostResponse> {
    info!("Handler: Retrieving post with id: {}", id);

    let repo = PostRepository::new((*pool).clone());

    match repo.find_by_id_with_author(id).await {
        Ok(Some(post)) => success_response(
            format!("Post '{}' retrieved successfully", post.title),
            post,
        ),
        Ok(None) => {
            error!("Post not found: {}", id);
            not_found_response_generic("Post not found".to_string())
        }
        Err(e) => {
            error!("Handler: Failed to retrieve post: {}", e);
            sql_error_generic(e, "Failed to retrieve post")
        }
    }
}
