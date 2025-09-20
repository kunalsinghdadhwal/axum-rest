use axum::{
    Json,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::helpers::auth::AuthHelper;
use crate::model::model::ErrorResponse;

use tracing::{error, info};

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // First try to get token from cookies
    let mut token_opt = None;

    // Extract cookies from request headers
    if let Some(cookie_header) = request.headers().get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            // Parse cookies manually to find auth_token
            for cookie_part in cookie_str.split(';') {
                let cookie_part = cookie_part.trim();
                if cookie_part.starts_with("auth_token=") {
                    token_opt = Some(cookie_part[11..].to_string());
                    info!("Found auth token in cookies");
                    break;
                }
            }
        }
    }

    // If no cookie token found, try Authorization header
    if token_opt.is_none() {
        token_opt = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|auth_header| auth_header.to_str().ok())
            .and_then(|auth_str| {
                if auth_str.starts_with("Bearer ") {
                    info!("Found Bearer token in Authorization header");
                    Some(auth_str[7..].to_string())
                } else {
                    None
                }
            });
    }

    let token = match token_opt {
        Some(token) => token,
        None => {
            error!("No authentication found - neither cookie nor Authorization header");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Unauthorized".to_string(),
                    message: "Authentication required - provide either auth_token cookie or Authorization header".to_string(),
                }),
            ));
        }
    };

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
