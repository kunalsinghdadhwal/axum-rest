use std::{net::SocketAddr, sync::Arc};

use dotenv::dotenv;
use tokio::signal;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use axum::{
    Router,
    http::{Method, StatusCode},
    middleware,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
pub mod model;
pub use model::model::User;
mod db;
use db::db::get_pg_client;

pub mod helpers;

use helpers::middleware::auth_middleware;

mod handlers;
use handlers::{
    auth_handlers::{get_profile, home, login_user, logout_user, register_user, update_profile},
    post_handlers::{
        create_post, delete_post, get_all_posts, get_post, get_user_posts, update_post,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth_handlers::register_user,
        handlers::auth_handlers::login_user,
        handlers::auth_handlers::logout_user,
        handlers::auth_handlers::get_profile,
        handlers::auth_handlers::update_profile,
        handlers::post_handlers::create_post,
        handlers::post_handlers::delete_post,
        handlers::post_handlers::update_post,
        handlers::post_handlers::get_all_posts,
        handlers::post_handlers::get_user_posts,
        handlers::post_handlers::get_post,
    ),
    components(schemas(
        model::model::User,
        model::model::CreateUserRequest,
        model::model::UpdatePasswordRequest,
        model::model::UpdateUserRequest,
        model::model::LoginRequest,
        model::model::LoginResponse,
        model::model::UserResponse,
        model::model::Post,
        model::model::CreatePostRequest,
        model::model::UpdatePostRequest,
        model::model::PostResponse,
        model::model::ApiResponse<model::model::UserResponse>,
        model::model::ApiResponse<model::model::LoginResponse>,
        model::model::ApiResponse<model::model::PostResponse>,
        model::model::ApiResponse<Vec<model::model::PostResponse>>,
        model::model::ApiResponse<Vec<model::model::Post>>,
        model::model::ErrorResponse,
        helpers::response::UnifiedResponse<model::model::UserResponse>,
        helpers::response::UnifiedResponse<model::model::LoginResponse>,
        helpers::response::UnifiedResponse<model::model::PostResponse>,
        helpers::response::UnifiedResponse<Vec<model::model::PostResponse>>,
        helpers::response::UnifiedResponse<Vec<model::model::Post>>,
    )),
    tags(
        (name = "Authentication", description = "User authentication and profile management"),
        (name = "Posts", description = "Blog post management operations")
    ),
    info(
        title = "Axum REST API",
        version = "1.0.0",
        description = "A REST API built with Axum framework for user authentication and blog post management. Supports both Bearer token and cookie-based authentication.",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server")
    )
)]
struct ApiDoc;

impl ApiDoc {
    fn with_security() -> utoipa::openapi::OpenApi {
        use utoipa::openapi::security::{
            ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme,
        };

        let mut openapi = Self::openapi();
        let components = openapi.components.as_mut().unwrap();

        // Bearer token authentication
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT token for API authentication. Include as: Authorization: Bearer <token>"))
                    .build(),
            ),
        );

        // Cookie authentication
        components.add_security_scheme(
            "cookie_auth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth_token"))),
        );

        openapi
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize tracing with proper configuration
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Axum REST API server...");

    let sql_db = match get_pg_client().await {
        Ok(client) => {
            tracing::info!("Successfully connected to the database.");
            client
        }
        Err(e) => {
            tracing::error!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }
    };

    let pool = Arc::new(sql_db.get_pool().clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::with_security()))
        // Home route
        .route("/", get(home))
        // Authentication routes
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
        .route("/auth/profile", get(get_profile))
        .route("/auth/profile", put(update_profile))
        // Public post routes
        .route("/posts", get(get_all_posts))
        .route("/posts/{id}", get(get_post))
        // Protected post routes
        .route("/posts", post(create_post))
        .route("/posts/my", get(get_user_posts))
        .route("/posts/{id}", put(update_post))
        .route("/posts/{id}", delete(delete_post))
        .fallback(handler_404)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            |req: axum::extract::Request, next: axum::middleware::Next| async move {
                // Auth middleware
                let path = req.uri().path();
                if path.starts_with("/auth/profile")
                    || path.starts_with("/auth/logout")
                    || path.starts_with("/posts") && req.method() == "POST"
                    || path.starts_with("/posts/my")
                    || (path.starts_with("/posts/")
                        && (req.method() == "PUT" || req.method() == "DELETE"))
                {
                    auth_middleware(req, next).await
                } else {
                    Ok(next.run(req).await)
                }
            },
        ))
        .with_state(pool);

    let sock_addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::info!("Server starting on http://{}", sock_addr);
    let listener = tokio::net::TcpListener::bind(sock_addr).await.unwrap();

    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, initiating graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal, initiating graceful shutdown...");
        },
    }
}

async fn handler_404() -> impl IntoResponse {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>404 - Not Found</title>
        </head>
        <body>
            <h1>404 - Page Not Found</h1>
            <h2>Sorry, the page you are looking for does not exist.</h2>
            <p><a href="/">Go back to Home</a></p>
        </body>
        </html>
    "#;

    (StatusCode::NOT_FOUND, Html(html))
}
