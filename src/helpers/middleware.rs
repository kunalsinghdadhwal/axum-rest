use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json
};

use crate::helpers::auth::AuthHelper;
use crate::model::model::ErrorResponse;

use tracing::{error, info};

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| {
            if auth_str.starts_with("Bearer ") {
                Some(auth_str[7..].to_string())
            } else {
                None
            }
        });

    let token = match auth_header {
        Some(token) => token,
        None => {
            error!("Authorization header missing or malformed");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(), 
                    message: "Authorization header missing or malformed".to_string(),
                }),
            ));
        }
    }

    let user_id = match AuthHelper::extract_user_id_from_token(&token) {
        Ok(user_id) => user_id,
        Err(err) => {
            error!("Token validation failed: {}", err);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    message: "Invalid or expired token".to_string(),
                }),
            ));
        }
    };

    info!("Authenticated user_id: {}", user_id);
    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}