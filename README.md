# Axum REST API

A modern, high-performance REST API built with Rust using the Axum framework. This application provides user authentication and blog post management functionality with support for both Bearer token and cookie-based authentication.

## Features

### Authentication
- User registration and login
- JWT-based authentication with Bearer tokens
- HTTP-only cookie authentication support
- Dual authentication system (Bearer token or cookies)
- Password change functionality
- User profile management
- Secure logout with cookie clearing

### Post Management
- Create, read, update, and delete blog posts
- User-specific post management
- Public post viewing
- Author-based access control
- Comprehensive post filtering and retrieval

### Technical Features
- Built with Axum 0.8.4 for high-performance async handling
- PostgreSQL database integration with SQLx
- OpenAPI 3.0 documentation with Scalar UI
- CORS support for cross-origin requests
- Structured logging with tracing
- Professional error handling and validation
- Docker support for development environment

## Technology Stack

- **Framework**: Axum 0.8.4
- **Database**: PostgreSQL with SQLx 0.8.6
- **Authentication**: JWT with jsonwebtoken, bcrypt for password hashing
- **Documentation**: OpenAPI 3.0 with utoipa and Scalar UI
- **Serialization**: Serde with JSON support
- **Async Runtime**: Tokio
- **Logging**: Tracing with structured logging
- **Environment**: dotenv for configuration management

## Prerequisites

- Rust 1.70+ (Edition 2024)
- PostgreSQL 12+
- Docker and Docker Compose (for development setup)

## Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd axum-rest
```

### 2. Environment Setup

Create a `.env` file in the project root:

```env
DATABASE_URL=postgresql://username:password@localhost:5432/axum_rest_db
POSTGRES_USER=username
POSTGRES_PASSWORD=password
POSTGRES_DB=axum_rest_db
JWT_SECRET=your-super-secret-jwt-key-here
```

### 3. Database Setup

Start PostgreSQL using Docker Compose:

```bash
docker-compose up -d
```

### 4. Run Database Migrations

Create the necessary database tables by running the application once (it will create tables automatically based on the schema).

### 5. Build and Run

```bash
cargo build --release
cargo run
```

The API will be available at `http://localhost:8080`

## API Documentation

### Interactive Documentation

Access the interactive API documentation at:
- **Scalar UI**: `http://localhost:8080/docs`

### Authentication Methods

The API supports two authentication methods:

1. **Bearer Token**: Include in Authorization header
   ```
   Authorization: Bearer <your-jwt-token>
   ```

2. **HTTP-Only Cookies**: Automatically set after login
   - Cookie name: `auth-token`
   - Secure, HTTP-only cookie for enhanced security

### Core Endpoints

#### Authentication Endpoints

| Method | Endpoint | Description | Authentication |
|--------|----------|-------------|----------------|
| POST | `/auth/register` | Register new user account | None |
| POST | `/auth/login` | User login (sets cookies and returns token) | None |
| POST | `/auth/logout` | User logout (clears cookies) | Required |
| GET | `/auth/profile` | Get current user profile | Required |
| PUT | `/auth/profile` | Update user profile | Required |
| PUT | `/auth/change-password` | Change user password | Required |

#### Post Management Endpoints

| Method | Endpoint | Description | Authentication |
|--------|----------|-------------|----------------|
| GET | `/posts` | Get all posts (public) | None |
| GET | `/posts/{id}` | Get specific post by ID | None |
| POST | `/posts` | Create new post | Required |
| GET | `/posts/my` | Get current user's posts | Required |
| PUT | `/posts/{id}` | Update post (owner only) | Required |
| DELETE | `/posts/{id}` | Delete post (owner only) | Required |

## Project Structure

```
src/
├── main.rs                 # Application entry point and routing
├── lib.rs                  # Library root
├── db/
│   ├── mod.rs              # Database module exports
│   ├── db.rs               # Database connection management
│   └── repositories/
│       ├── mod.rs          # Repository module exports
│       ├── user_repo.rs    # User database operations
│       └── post_repo.rs    # Post database operations
├── handlers/
│   ├── mod.rs              # Handler module exports
│   ├── auth_handlers.rs    # Authentication endpoint handlers
│   └── post_handlers.rs    # Post management endpoint handlers
├── helpers/
│   ├── mod.rs              # Helper module exports
│   ├── auth.rs             # Authentication utilities
│   ├── middleware.rs       # Authentication middleware
│   ├── response.rs         # Response type definitions
│   └── validation.rs       # Input validation utilities
└── model/
    ├── mod.rs              # Model module exports
    └── model.rs            # Data structures and schemas
```

## Development

### Running in Development Mode

```bash
cargo run
```

The server will start with hot reloading capabilities and detailed logging.

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Database Operations

The application uses SQLx for type-safe database operations with PostgreSQL. All database operations are async and use connection pooling for optimal performance.

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `JWT_SECRET` | Secret key for JWT token signing | Required |
| `POSTGRES_USER` | Database username | Required |
| `POSTGRES_PASSWORD` | Database password | Required |
| `POSTGRES_DB` | Database name | Required |

### Server Configuration

- **Host**: `127.0.0.1`
- **Port**: `8080`
- **CORS**: Enabled for all origins in development

## Security Features

- **Password Hashing**: bcrypt with secure salt rounds
- **JWT Tokens**: Signed with secret key, expiration included
- **HTTP-Only Cookies**: Secure cookie storage for authentication
- **Input Validation**: Comprehensive request validation
- **SQL Injection Protection**: Parameterized queries with SQLx
- **CORS Configuration**: Configurable cross-origin resource sharing

## Error Handling

The API provides consistent error responses with appropriate HTTP status codes:

- **400**: Bad Request (validation errors)
- **401**: Unauthorized (authentication required)
- **403**: Forbidden (insufficient permissions)
- **404**: Not Found (resource not found)
- **409**: Conflict (duplicate resources)
- **500**: Internal Server Error (server errors)

All error responses follow a consistent JSON structure with descriptive messages.

## Performance

- **Async/Await**: Full async support with Tokio runtime
- **Connection Pooling**: PostgreSQL connection pooling with SQLx
- **Zero-Copy Parsing**: Efficient request/response handling with Axum
- **Structured Logging**: Performance monitoring with tracing

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions, issues, or contributions, please:
- Open an issue on the repository
- Contact the development team
- Review the API documentation at `/docs`

## Changelog

### Version 1.0.0
- Initial release with full authentication and post management
- Dual authentication support (Bearer tokens and cookies)
- Complete OpenAPI documentation
- PostgreSQL integration with connection pooling
- Professional error handling and validation
