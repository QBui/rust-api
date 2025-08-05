# Scalable Rust API - Makefile for development and deployment

.PHONY: help build test run docker-build docker-run clean setup migrate dev prod

# Default target
help:
	@echo "Available commands:"
	@echo "  setup     - Initial project setup"
	@echo "  dev       - Run development environment"
	@echo "  build     - Build the Rust application"
	@echo "  test      - Run all tests"
	@echo "  migrate   - Run database migrations"
	@echo "  docker    - Build and run with Docker"
	@echo "  prod      - Deploy production environment"
	@echo "  clean     - Clean build artifacts"

# Initial setup
setup:
	@echo "Setting up development environment..."
	cp .env.example .env
	docker-compose up -d postgres redis
	sleep 5
	$(MAKE) migrate

# Development environment
dev:
	@echo "Starting development environment..."
	docker-compose up -d postgres redis
	RUST_LOG=debug cargo run --bin api

# Build the application
build:
	@echo "Building Rust application..."
	cargo build --release

# Run tests
test:
	@echo "Running tests..."
	cargo test --all
	cargo clippy --all-targets --all-features
	cargo fmt --check

# Database migrations
migrate:
	@echo "Running database migrations..."
	cargo install sqlx-cli --no-default-features --features postgres
	sqlx migrate run --database-url "postgres://postgres:password@localhost:5432/scalable_api_dev"

# Docker build and run
docker:
	@echo "Building and running with Docker..."
	docker-compose down
	docker-compose up --build

# Production deployment
prod:
	@echo "Deploying production environment..."
	docker build -t scalable-rust-api:latest .
	docker-compose -f docker-compose.prod.yml up -d

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	docker-compose down -v
	docker system prune -f
