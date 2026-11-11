# Loco API - Task Runner (Justfile)
# Run `just` or `just --list` to view all available commands
set shell := ["bash", "-euo", "pipefail", "-c"]
set dotenv-load := true
app_port := env("APP_PORT", "5150")

default:
    @just --list

# ------------------------------------------------------------------------------
# Development & Service
# ------------------------------------------------------------------------------

# Start the application server
dev *args:
    cargo loco start --no-banner {{args}}

# Start with file watching and auto-restart
watch *args:
    cargo loco watch --no-banner {{args}}

# Build release binary and start the server (ideal for benchmarks and perf testing)
start-release *args: build
    ./target/release/locos start --no-banner {{args}}

# Build release binary and start server in production mode
prod *args: build
    LOCO_ENV=production ./target/release/locos start --no-banner {{args}}

# Print all registered HTTP routes
routes:
    cargo loco routes

# Diagnose system dependencies, database and cache connectivity
doctor *args:
    cargo loco doctor {{args}}

outdated :
    cargo outdated -d 1
upgrade-dry:
    cargo upgrade --dry-run --incompatible
upgrade *args:
    cargo upgrade {{args}}

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

# Run all test suites (auto-detects cargo-nextest with fallback to cargo test)
test *args:
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        cargo nextest run {{args}}; \
    else \
        cargo test {{args}}; \
    fi

# Build optimized release binary (target/release/locos)
build:
    cargo build --release --bin locos

# Complete quality gate: format check, lint, and test
check: fmt-check lint test

# 压测（wrk）
wrk path="/livez" *args="":
    wrk -t4 -c200 -d30s --latency {{args}} "http://localhost:{{app_port}}{{path}}"

oha path="/livez":
	oha -z 30s -c 200 --latency-correction "http://localhost:{{app_port}}{{path}}"


# ------------------------------------------------------------------------------
# Maintenance & Cleanup
# ------------------------------------------------------------------------------

# Smart sweep target directory (removes stale artifacts & incremental cache, keeps latest build)
sweep maxsize="2000":
    @if command -v cargo-sweep >/dev/null 2>&1; then \
        cargo sweep --maxsize {{maxsize}}; \
    else \
        echo "⚠️ cargo-sweep not installed. Run: cargo install cargo-sweep"; \
    fi
    @rm -rf target/debug/incremental
    @echo "✅ target directory cleaned (incremental cache wiped, stale artifacts purged)"
