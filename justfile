# Loco API - Task Runner (Justfile)
# Run `just` or `just --list` to view all available commands

default:
    @just --list

# ------------------------------------------------------------------------------
# Development & Service
# ------------------------------------------------------------------------------

# Start the application server
dev *args:
    cargo loco start {{args}}

# Start with file watching and auto-restart
watch *args:
    cargo loco watch {{args}}

# Print all registered HTTP routes
routes:
    cargo loco routes

# Diagnose system dependencies, database and cache connectivity
doctor *args:
    cargo loco doctor {{args}}

# ------------------------------------------------------------------------------
# Database Operations
# ------------------------------------------------------------------------------

# Apply pending database migrations
migrate:
    cargo loco db migrate

# Show database migration status
db-status:
    cargo loco db status

# Rollback migrations
rollback *args:
    cargo loco db rollback {{args}}

# Reset database (truncate/drop, recreate, and migrate)
db-reset:
    cargo loco db reset

# Seed database with initial fixtures
db-seed:
    cargo loco db seed

# ------------------------------------------------------------------------------
# Quality & CI Gates
# ------------------------------------------------------------------------------

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check code format without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter with strict warning checks
lint:
    cargo clippy --all-targets -- -D warnings

# Run all test suites
test *args:
    cargo test {{args}}

# Build optimized release binary (target/release/locos)
build:
    cargo build --release --bin locos

# Complete quality gate: format check, lint, and test
check: fmt-check lint test

# ------------------------------------------------------------------------------
# Container & Infrastructure
# ------------------------------------------------------------------------------

# Start Docker Compose services in background
up:
    docker compose up -d

# Stop Docker Compose services
down:
    docker compose down

# Follow logs from Docker Compose services
logs *args:
    docker compose logs -f {{args}}
