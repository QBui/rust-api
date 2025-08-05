# Scalable Rust Web API

A highly scalable and maintainable web API built with Rust that demonstrates modern backend architecture patterns and best practices.

## 🏗️ Architecture Overview

This API is built using a modular, layered architecture with the following components:

- **API Layer** (`crates/api`): HTTP routing, middleware, and request handling
- **Core Layer** (`crates/core`): Business models, configuration, and error handling
- **Database Layer** (`crates/database`): Repository pattern with PostgreSQL
- **Auth Layer** (`crates/auth`): JWT authentication and password hashing
- **Monitoring Layer** (`crates/monitoring`): Metrics, logging, and observability

## 🚀 Features

### Core Features
- **RESTful API Design** with proper HTTP status codes and error handling
- **JWT Authentication** with secure password hashing using Argon2
- **Role-based Access Control (RBAC)** for user permissions
- **Database Migrations** with SQLx for schema management
- **Input Validation** using the validator crate
- **Pagination** for efficient data retrieval

### Scalability & Performance
- **Connection Pooling** for database efficiency
- **Rate Limiting** using token bucket algorithm
- **Caching Strategy** with Redis integration
- **Async/Await** throughout for non-blocking operations
- **Structured Logging** with tracing for observability
- **Metrics Collection** with Prometheus integration

### Security
- **Password Hashing** with Argon2 (secure against timing attacks)
- **JWT Token Validation** with proper expiration handling
- **Input Sanitization** and validation on all endpoints
- **CORS Configuration** for cross-origin security
- **Rate Limiting** to prevent abuse and DDoS attacks

### DevOps & Monitoring
- **Docker Containerization** with multi-stage builds
- **Docker Compose** for local development environment
- **Health Checks** for service monitoring
- **Prometheus Metrics** for application monitoring
- **Grafana Dashboards** for visualization
- **Structured Logging** with JSON output for production

## 📦 Project Structure

```
rust-api/
├── Cargo.toml                 # Workspace configuration
├── docker-compose.yml         # Development environment
├── Dockerfile                 # Production container
├── config/                    # Environment configurations
│   ├── development.yaml
│   └── production.yaml
├── migrations/                # Database migrations
│   ├── 001_create_users.sql
│   └── 002_create_products.sql
└── crates/                    # Rust crates
    ├── api/                   # HTTP API layer
    ├── auth/                  # Authentication service
    ├── core/                  # Core business logic
    ├── database/              # Data access layer
    └── monitoring/            # Observability
```

## 🛠️ Technology Stack

### Core Technologies
- **Rust** - Systems programming language for performance and safety
- **Axum** - Modern async web framework built on Tokio
- **SQLx** - Async SQL toolkit with compile-time checked queries
- **PostgreSQL** - Primary database with ACID compliance
- **Redis** - Caching and session storage

### Authentication & Security
- **Argon2** - Password hashing algorithm
- **JWT** - Stateless authentication tokens
- **UUID** - Secure unique identifiers

### Monitoring & Observability
- **Tracing** - Structured logging and distributed tracing
- **Prometheus** - Metrics collection and monitoring
- **Grafana** - Dashboards and visualization

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ with Cargo
- Docker and Docker Compose
- PostgreSQL 15+ (for local development)

### Development Setup

1. **Clone and setup the project:**
```bash
cd rust-api
cp config/development.yaml.example config/development.yaml
```

2. **Start the development environment:**
```bash
docker-compose up -d postgres redis
```

3. **Run database migrations:**
```bash
cargo install sqlx-cli
sqlx migrate run
```

4. **Start the API server:**
```bash
cargo run --bin api
```

The API will be available at `http://localhost:8080`

### Using Docker

**Run the complete stack:**
```bash
docker-compose up --build
```

This starts:
- API server on port 8080
- PostgreSQL on port 5432
- Redis on port 6379
- Prometheus on port 9091
- Grafana on port 3000

## 📚 API Documentation

### Authentication Endpoints

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
```

### User Management

```http
# List users (paginated)
GET /api/v1/users?page=1&per_page=20
Authorization: Bearer <jwt_token>

# Create user
POST /api/v1/users
Content-Type: application/json

{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "secure_password"
}

# Get user by ID
GET /api/v1/users/{id}
Authorization: Bearer <jwt_token>

# Update user
PUT /api/v1/users/{id}
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "username": "newusername",
  "email": "newemail@example.com"
}
```

### Product Management

```http
# List products
GET /api/v1/products?page=1&per_page=20

# Create product (requires admin/merchant role)
POST /api/v1/products
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "name": "Awesome Product",
  "description": "Product description",
  "price": 2999,
  "category_id": "uuid-here"
}
```

### Health & Monitoring

```http
# Health check
GET /health

# Prometheus metrics
GET /metrics
```

## 🔧 Configuration

The application uses YAML configuration files with environment variable overrides:

```yaml
server:
  host: "0.0.0.0"
  port: 8080

database:
  url: "postgres://user:pass@localhost/db"
  max_connections: 10

auth:
  jwt_secret: "your-secret-key"
  jwt_expiration: 3600

redis:
  url: "redis://localhost:6379"
```

### Environment Variables

Key environment variables for production:

- `DATABASE_URL` - PostgreSQL connection string
- `JWT_SECRET` - Secret key for JWT signing
- `REDIS_URL` - Redis connection string
- `RUST_LOG` - Logging level configuration

## 📊 Monitoring & Observability

### Metrics

The API exposes Prometheus metrics at `/metrics`:

- `http_requests_total` - Total HTTP requests by method, path, and status
- `http_request_duration_seconds` - Request duration histogram
- `database_operations_total` - Database operation counters
- `auth_events_total` - Authentication event counters

### Logging

Structured JSON logging with configurable levels:

```bash
# Set log level
export RUST_LOG=info,sqlx=warn

# Development logging
export RUST_LOG=debug
```

### Health Checks

- **Application Health**: `GET /health`
- **Database Health**: Included in health endpoint
- **Docker Health**: Built into container configuration

## 🧪 Testing

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --out Html
```

## 🚀 Deployment

### Production Build

```bash
# Build optimized binary
cargo build --release

# Build Docker image
docker build -t scalable-rust-api .
```

### Environment Considerations

- Use `production.yaml` config for production
- Set strong `JWT_SECRET` and database passwords
- Configure proper logging levels
- Set up monitoring and alerting
- Use connection pooling for database
- Implement proper backup strategies

## 🔒 Security Considerations

- **Password Security**: Uses Argon2 with appropriate cost parameters
- **JWT Security**: Tokens have expiration and proper validation
- **Input Validation**: All inputs are validated and sanitized
- **SQL Injection**: Protection through parameterized queries
- **Rate Limiting**: Built-in protection against abuse
- **CORS**: Configurable cross-origin resource sharing

## 📈 Performance Features

- **Async/Await**: Non-blocking I/O throughout the application
- **Connection Pooling**: Efficient database connection management
- **Lazy Loading**: Efficient data loading strategies
- **Caching**: Redis integration for frequently accessed data
- **Compression**: HTTP response compression
- **Pagination**: Efficient handling of large datasets

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
