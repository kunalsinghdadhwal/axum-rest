# Tracing and OpenAPI Documentation Setup

## Tracing Configuration

Your Axum server now has comprehensive tracing configured with the following features:

### 1. Enhanced Tracing Subscriber
- Uses `tracing-subscriber` with env-filter support
- Default log level: `debug` for your application, `debug` for tower_http, `trace` for axum
- Can be overridden with the `RUST_LOG` environment variable

### 2. HTTP Request Tracing
- Added `TraceLayer::new_for_http()` to trace all HTTP requests/responses
- Logs request method, URI, status code, and response time
- Useful for debugging and monitoring API usage

### 3. Application-Level Tracing
- Database connection status
- Server startup information
- Graceful shutdown signals
- Handler execution (already present in your handlers)

## How to Use Tracing

### Running with Default Tracing
```bash
cargo run
```

### Running with Custom Log Levels
```bash
# Only errors and warnings
RUST_LOG=error cargo run

# Everything (very verbose)
RUST_LOG=trace cargo run

# Custom configuration
RUST_LOG=axum_rest=debug,tower_http=info,sqlx=debug cargo run

# Only your application logs
RUST_LOG=axum_rest=debug cargo run
```

### Common Log Levels
- `error`: Only errors
- `warn`: Warnings and errors
- `info`: General information, warnings, and errors
- `debug`: Debug information and above (default)
- `trace`: Everything (very verbose)

## OpenAPI Documentation

### Features Added
1. **Complete API Documentation**: All endpoints are documented with:
   - Request/response schemas
   - HTTP status codes
   - Authentication requirements
   - Parameter descriptions

2. **Interactive UI**: Scalar UI at `/docs` endpoint
   - Test endpoints directly from the browser
   - Authentication support
   - Request/response examples

3. **Authentication Documentation**: 
   - Bearer token authentication documented
   - Security schemes properly configured

### Accessing Documentation
1. Start your server: `cargo run`
2. Visit: `http://localhost:8080/docs`
3. Explore and test your API endpoints

### Home Page
- Visit: `http://localhost:8080/`
- Contains overview of all endpoints
- Direct link to documentation

## Example Tracing Output

When running with default settings, you'll see logs like:

```
2024-09-20T10:30:00.123456Z  INFO axum_rest: Starting Axum REST API server...
2024-09-20T10:30:00.234567Z  INFO axum_rest: Successfully connected to the database.
2024-09-20T10:30:00.345678Z  INFO axum_rest: Server starting on http://127.0.0.1:8080
2024-09-20T10:30:00.456789Z  INFO axum_rest: API Documentation available at http://127.0.0.1:8080/docs
2024-09-20T10:30:05.567890Z  INFO tower_http::trace::on_request: started processing request method=GET uri=/posts
2024-09-20T10:30:05.678901Z DEBUG axum_rest::handlers::post_handlers: Handler: Retrieving all posts
2024-09-20T10:30:05.789012Z  INFO tower_http::trace::on_response: finished processing request latency=221ms status=200
```

## Troubleshooting

### If you don't see logs:
1. Check your `RUST_LOG` environment variable
2. Ensure tracing statements use the correct module paths
3. Try running with `RUST_LOG=debug cargo run`

### If documentation doesn't load:
1. Ensure server is running on port 8080
2. Check that `/docs` endpoint is accessible
3. Verify no firewall blocking the connection

## What's Documented

### Authentication Endpoints
- `POST /auth/register` - User registration
- `POST /auth/login` - User authentication  
- `GET /auth/profile` - Get user profile (protected)
- `PUT /auth/profile` - Update user profile (protected)

### Post Management Endpoints
- `GET /posts` - Get all posts (public)
- `GET /posts/{id}` - Get specific post (public)
- `POST /posts` - Create new post (protected)
- `GET /posts/my` - Get user's posts (protected)
- `PUT /posts/{id}` - Update post (protected)
- `DELETE /posts/{id}` - Delete post (protected)

All protected endpoints require Bearer token authentication.
