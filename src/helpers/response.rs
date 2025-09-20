use axum::{Json, http::StatusCode, response::IntoResponse};
use axum_extra::extract::cookie::Cookie;
use serde_json::Value;
use utoipa::ToSchema;

use crate::model::model::{ApiResponse, ErrorResponse};

// Type aliases for OpenAPI documentation
pub type ApiSuccessResponse<T> = ApiResponse<T>;
pub type ApiErrorResponse = ErrorResponse;

#[derive(serde::Serialize, ToSchema)]
#[serde(untagged)]
pub enum UnifiedResponse<T> {
    Success(ApiResponse<T>),
    Error(ErrorResponse),
}

impl<T> IntoResponse for UnifiedResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            UnifiedResponse::Success(response) => {
                let (status, json) = (StatusCode::OK, Json(response));
                (status, json).into_response()
            }
            UnifiedResponse::Error(err) => {
                let (status, json) = (StatusCode::BAD_REQUEST, Json(err));
                (status, json).into_response()
            }
        }
    }
}

pub fn error_response_generic<T>(error: String, message: String) -> UnifiedResponse<T> {
    UnifiedResponse::Error(ErrorResponse { error, message })
}

pub fn not_found_response_generic<T>(message: String) -> UnifiedResponse<T> {
    UnifiedResponse::Success(ApiResponse {
        message,
        data: None,
    })
}

pub fn sql_error_generic<T>(_error: anyhow::Error, context: &str) -> UnifiedResponse<T> {
    UnifiedResponse::Error(ErrorResponse {
        error: "Database Error".to_string(),
        message: context.to_string(),
    })
}

pub fn create_response<T>(
    message: String,
    data: Option<T>,
    status_code: StatusCode,
) -> (StatusCode, Json<ApiResponse<T>>) {
    let response = ApiResponse { message, data };
    (status_code, Json(response))
}

pub fn create_error_response(
    error: String,
    message: String,
    status_code: StatusCode,
) -> (StatusCode, Json<ErrorResponse>) {
    let response = ErrorResponse { error, message };
    (status_code, Json(response))
}

pub fn handle_sql_error(
    error: anyhow::Error,
    error_ctx: &str,
) -> (StatusCode, Json<ErrorResponse>) {
    create_error_response(
        "DatabaseError".to_string(),
        format!("{}: {}", error_ctx, error),
        StatusCode::INTERNAL_SERVER_ERROR,
    )
}

pub fn success_response<T>(message: String, data: T) -> UnifiedResponse<T> {
    UnifiedResponse::Success(ApiResponse {
        message,
        data: Some(data),
    })
}

pub fn error_response(error: String, message: String) -> UnifiedResponse<Value> {
    UnifiedResponse::Error(ErrorResponse { error, message })
}

pub fn not_found_response(message: String) -> UnifiedResponse<Value> {
    UnifiedResponse::Success(ApiResponse {
        message,
        data: None,
    })
}

pub fn sql_error_response(error: anyhow::Error, context: &str) -> UnifiedResponse<Value> {
    UnifiedResponse::Error(ErrorResponse {
        error: "Database Error".to_string(),
        message: format!("{}: {}", context, error),
    })
}

// Cookie-enabled response for login functionality
pub struct CookieResponse<T> {
    pub response: UnifiedResponse<T>,
    pub cookies: Vec<Cookie<'static>>,
}

impl<T> CookieResponse<T> {
    pub fn new(response: UnifiedResponse<T>) -> Self {
        Self {
            response,
            cookies: Vec::new(),
        }
    }

    pub fn with_cookie(mut self, cookie: Cookie<'static>) -> Self {
        self.cookies.push(cookie);
        self
    }
}

impl<T> IntoResponse for CookieResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let mut response = self.response.into_response();

        // Add cookies to response headers
        for cookie in self.cookies {
            if let Ok(header_value) = cookie.to_string().parse() {
                response
                    .headers_mut()
                    .append(axum::http::header::SET_COOKIE, header_value);
            }
        }

        response
    }
}

pub fn success_response_with_cookies<T>(
    message: String,
    data: T,
    cookies: Vec<Cookie<'static>>,
) -> CookieResponse<T> {
    let mut response = CookieResponse::new(UnifiedResponse::Success(ApiResponse {
        message,
        data: Some(data),
    }));

    for cookie in cookies {
        response = response.with_cookie(cookie);
    }

    response
}

pub fn error_response_with_cookies<T>(error: String, message: String) -> CookieResponse<T> {
    CookieResponse::new(UnifiedResponse::Error(ErrorResponse { error, message }))
}

pub fn sql_error_response_with_cookies<T>(
    error: anyhow::Error,
    context: &str,
) -> CookieResponse<T> {
    CookieResponse::new(UnifiedResponse::Error(ErrorResponse {
        error: "DatabaseError".to_string(),
        message: format!("{}: {}", context, error),
    }))
}
