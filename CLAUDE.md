# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

**Build and Development:**
- `cargo build --release` - Build optimized release binary
- `cargo run --bin api` - Run the API server directly
- `make dev` - Start development environment with Docker services
- `make build` - Build the application using Makefile

**Testing:**
- `cargo test --all` - Run all unit and integration tests
- `make test` - Run tests with clippy and formatting checks
- `cargo clippy --all-targets --all-features` - Run linting
- `cargo fmt --check` - Check code formatting

**Database:**
- `make migrate` - Run database migrations
- `sqlx migrate run` - Run migrations directly with SQLx

**Docker:**
- `make docker` - Build and run complete stack with Docker Compose
- `docker-compose up -d postgres redis` - Start just database services

## Architecture

This is a modular Rust web API using a workspace structure with 5 crates:

**Core Crates:**
- `crates/api/` - HTTP layer with Axum, handlers, middleware, routes
- `crates/core/` - Business models, configuration, error handling, enterprise features
- `crates/database/` - Repository pattern with SQLx and PostgreSQL
- `crates/auth/` - JWT authentication and Argon2 password hashing
- `crates/monitoring/` - Metrics, tracing, audit logs, circuit breakers, feature flags

**Key Patterns:**
- Repository pattern for data access in `database/repositories/`
- Middleware stack in `api/middleware/` for auth, rate limiting, metrics
- Handlers in `api/handlers/` following REST conventions
- Shared `AppState` containing all services and dependencies
- Enterprise features: audit logging, circuit breakers, feature flags

**Configuration:**
- YAML configs in `config/` directory (development.yaml, production.yaml)
- Environment variable overrides supported
- Database migrations in `migrations/` and `crates/database/migrations/`

**Tech Stack:**
- Axum web framework with tower middleware
- SQLx for type-safe database queries
- Redis for caching and rate limiting
- Prometheus metrics integration
- Structured logging with tracing crate