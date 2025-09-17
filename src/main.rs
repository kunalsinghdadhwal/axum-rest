use std::net::SocketAddr;

use dotenv::dotenv;
use tokio::signal;

use axum::{
    Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let sock_addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = Router::new().route("/", get(handler));
    let app = app.fallback(handler_404);

    let listener = tokio::net::TcpListener::bind(sock_addr).await.unwrap();
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
