use std::{net::SocketAddr, sync::Arc};

use axum_rest::{db::db::get_pg_client, helpers::middleware::auth_middleware};
use dotenv::dotenv;
use tokio::signal;

use axum::{
    Router,
    http::{Method, StatusCode},
    middleware,
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer};
mod handlers;
use handlers::{
    auth_handlers::{get_profile, login_user, register_user, update_profile},
    post_handlers::{
        create_post, delete_post, get_all_posts, get_post, get_user_posts, update_post,
    },
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let sql_db = match get_pg_client().await {
        Ok(client) => {
            println!("Successfully connected to the database.");
            client
        }
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }
    };

    let pool = Arc::new(sql_db.get_pool().clone());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(handler))
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/posts", get(get_all_posts))
        .route("/posts/{id}", get(get_post))
        .route("/auth/profile", get(get_profile))
        .route("/auth/profile", put(update_profile))
        .route("/posts", post(create_post))
        .route("/posts/my", get(get_user_posts))
        .route("/posts/{id}", put(update_post))
        .route("/posts/{id}", delete(delete_post))
        .fallback(handler_404)
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            |req: axum::extract::Request, next: axum::middleware::Next| async move {
                // Auth middleware
                let path = req.uri().path();
                if path.starts_with("/auth/profile")
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

    tracing::debug!("listening on {}", sock_addr);
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
        _ = ctrl_c => {},
        _ = terminate => {},
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

async fn handler() -> Html<&'static str> {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Welcome</title>
        </head>
        <body>
            <h1>Welcome to the Home Page</h1>
            <p>This is a simple Axum web server.</p>
        </body>
        </html>
    "#;

    Html(html)
}
