use crate::db::repositories::post_repo::PostRepository;
use crate::helpers::response::{
    UnifiedResponse, error_response_generic, not_found_response_generic, sql_error_generic,
    success_response,
};
use crate::model::model::{self, CreatePostRequest, PostResponse, UpdatePostRequest};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use utoipa;
use uuid::Uuid;

/// Create a new post
#[utoipa::path(
    post,
    path = "/posts",
    request_body = CreatePostRequest,
    responses(
        (status = 200, description = "Post created successfully", body = inline(crate::helpers::response::ApiSuccessResponse<PostResponse>)),
        (status = 400, description = "Validation error", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Posts"
)]
pub async fn create_post(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreatePostRequest>,
) -> UnifiedResponse<PostResponse> {
    info!("Handler: Creating new post for user_id: {}", user_id);

    if payload.title.trim().is_empty() || payload.content.trim().is_empty() {
        error!("Validation error: Title and content cannot be empty");
        return error_response_generic(
            "Creation Failed".to_string(),
            "Title and content are required".to_string(),
        );
    }

    let repo = PostRepository::new((*pool).clone());

    match repo.create_post(payload, user_id).await {
        Ok(post) => match repo.find_by_id_with_author(post.id).await {
            Ok(Some(post_response)) => success_response("Post Created".to_string(), post_response),
            Ok(None) => {
                error!("Post created but not found: {}", post.id);
                error_response_generic(
                    "Creation Failed".to_string(),
                    "Post was created but could not be retrieved".to_string(),
                )
            }
            Err(e) => {
                error!(
                    "Handler: Failed to retrieve created post with author info: {}",
                    e
                );
                sql_error_generic(e, "Unable to retrieve post details")
            }
        },
        Err(e) => {
            error!("Handler: Failed to create post: {}", e);
            sql_error_generic(e, "Unable to create post")
        }
    }
}

/// Delete a post by ID
#[utoipa::path(
    delete,
    path = "/posts/{id}",
    params(
        ("id" = Uuid, Path, description = "Post ID to delete")
    ),
    responses(
        (status = 200, description = "Post deleted successfully", body = inline(crate::helpers::response::ApiSuccessResponse<String>)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 403, description = "Forbidden - Not the post author", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "Post not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Posts"
)]
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
        Ok(true) => success_response("Post Deleted".to_string(), Value::Null),
        Ok(false) => {
            error!("Post not found or unauthorized deletion attempt: {}", id);
            not_found_response_generic("Post not found or unauthorized access".to_string())
        }
        Err(e) => {
            error!("Handler: Failed to delete post: {}", e);
            sql_error_generic(e, "Unable to delete post")
        }
    }
}

/// Update a post by ID
#[utoipa::path(
    put,
    path = "/posts/{id}",
    params(
        ("id" = Uuid, Path, description = "Post ID to update")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Post updated successfully", body = inline(crate::helpers::response::ApiSuccessResponse<PostResponse>)),
        (status = 400, description = "Validation error", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 403, description = "Forbidden - Not the post author", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 404, description = "Post not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Posts"
)]
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
            Ok(Some(post_response)) => success_response("Post Updated".to_string(), post_response),
            Ok(None) => error_response_generic(
                "Update Failed".to_string(),
                "Post was updated but could not be retrieved".to_string(),
            ),
            Err(e) => {
                error!(
                    "Handler: Failed to retrieve updated post with author info: {}",
                    e
                );
                sql_error_generic(e, "Unable to retrieve updated post details")
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
            sql_error_generic(e, "Unable to update post")
        }
    }
}

/// Get all posts
#[utoipa::path(
    get,
    path = "/posts",
    responses(
        (status = 200, description = "All posts retrieved successfully", body = inline(crate::helpers::response::ApiSuccessResponse<Vec<PostResponse>>)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    tag = "Posts"
)]
pub async fn get_all_posts(State(pool): State<Arc<PgPool>>) -> UnifiedResponse<Vec<PostResponse>> {
    info!("Handler: Retrieving all posts");

    let repo = PostRepository::new((*pool).clone());

    match repo.get_all_posts().await {
        Ok(posts) => success_response("Posts Retrieved".to_string(), posts),
        Err(e) => {
            error!("Handler: Failed to retrieve posts: {}", e);
            sql_error_generic(e, "Unable to retrieve posts")
        }
    }
}

/// Get current user's posts
#[utoipa::path(
    get,
    path = "/posts/my",
    responses(
        (status = 200, description = "User posts retrieved successfully", body = inline(crate::helpers::response::ApiSuccessResponse<Vec<crate::model::model::Post>>)),
        (status = 401, description = "Unauthorized - Invalid or missing authentication", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    security(
        ("bearer_auth" = []),
        ("cookie_auth" = [])
    ),
    tag = "Posts"
)]
pub async fn get_user_posts(
    State(pool): State<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
) -> UnifiedResponse<Vec<model::Post>> {
    info!("Handler: Retrieving posts for user_id: {}", user_id);

    let repo = PostRepository::new((*pool).clone());

    match repo.find_by_author(user_id).await {
        Ok(posts) => success_response("Your Posts Retrieved".to_string(), posts),
        Err(e) => {
            error!("Handler: Failed to retrieve user posts: {}", e);
            sql_error_generic(e, "Unable to retrieve your posts")
        }
    }
}

/// Get a specific post by ID
#[utoipa::path(
    get,
    path = "/posts/{id}",
    params(
        ("id" = Uuid, Path, description = "Post ID to retrieve")
    ),
    responses(
        (status = 200, description = "Post retrieved successfully", body = inline(crate::helpers::response::ApiSuccessResponse<PostResponse>)),
        (status = 404, description = "Post not found", body = inline(crate::helpers::response::ApiErrorResponse)),
        (status = 500, description = "Internal server error", body = inline(crate::helpers::response::ApiErrorResponse))
    ),
    tag = "Posts"
)]
pub async fn get_post(
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> UnifiedResponse<PostResponse> {
    info!("Handler: Retrieving post with id: {}", id);

    let repo = PostRepository::new((*pool).clone());

    match repo.find_by_id_with_author(id).await {
        Ok(Some(post)) => success_response("Post Retrieved".to_string(), post),
        Ok(None) => {
            error!("Post not found: {}", id);
            not_found_response_generic("Post not found".to_string())
        }
        Err(e) => {
            error!("Handler: Failed to retrieve post: {}", e);
            sql_error_generic(e, "Unable to retrieve post")
        }
    }
}
