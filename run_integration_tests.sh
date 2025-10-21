#!/bin/bash
# Integration Tests Runner Script
# This script sets up the environment and runs integration tests

set -e

# Default values
DATABASE_URL="${TEST_DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/recommendations_test}"
REDIS_URL="${TEST_REDIS_URL:-redis://localhost:6379/1}"
SKIP_MIGRATIONS=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --database-url)
            DATABASE_URL="$2"
            shift 2
            ;;
        --redis-url)
            REDIS_URL="$2"
            shift 2
            ;;
        --skip-migrations)
            SKIP_MIGRATIONS=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --database-url URL    PostgreSQL connection string"
            echo "  --redis-url URL       Redis connection string"
            echo "  --skip-migrations     Skip running database migrations"
            echo "  --verbose             Show detailed test output"
            echo "  --help                Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment variables
export TEST_DATABASE_URL="$DATABASE_URL"
export TEST_REDIS_URL="$REDIS_URL"
export DATABASE_URL="$DATABASE_URL"

if [ "$VERBOSE" = true ]; then
    export RUST_LOG="debug"
fi

echo "=== Recommendation Engine Integration Tests ==="
echo ""
echo "Configuration:"
echo "  Database: $DATABASE_URL"
echo "  Redis: $REDIS_URL"
echo ""

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "WARNING: sqlx-cli not found. Install with: cargo install sqlx-cli"
    echo ""
fi

# Run migrations unless skipped
if [ "$SKIP_MIGRATIONS" = false ] && command -v sqlx &> /dev/null; then
    echo "Running database migrations..."
    if sqlx migrate run; then
        echo "✓ Migrations applied successfully"
    else
        echo "✗ Migration failed"
        echo ""
        echo "Common issues:"
        echo "  1. PostgreSQL is not running"
        echo "  2. Database 'recommendations_test' does not exist"
        echo "  3. pgvector extension is not installed"
        echo ""
        echo "To create the database:"
        echo "  createdb recommendations_test"
        echo ""
        echo "To install pgvector:"
        echo "  See: https://github.com/pgvector/pgvector#installation"
        echo ""
        exit 1
    fi
    echo ""
fi

# Run tests
echo "Running integration tests..."
echo ""

TEST_ARGS="test -p recommendation-integration-tests -- --test-threads=1"

if [ "$VERBOSE" = true ]; then
    TEST_ARGS="$TEST_ARGS --nocapture"
fi

if cargo $TEST_ARGS; then
    echo ""
    echo "=== All tests passed! ==="
    exit 0
else
    echo ""
    echo "=== Tests failed ==="
    echo ""
    echo "Troubleshooting:"
    echo "  1. Ensure PostgreSQL is running with pgvector extension"
    echo "  2. Ensure Redis is running"
    echo "  3. Check database credentials in connection string"
    echo "  4. Run with --verbose flag for detailed output"
    echo ""
    echo "For more information, see:"
    echo "  - INTEGRATION_TESTS.md"
    echo "  - crates/integration-tests/tests/README.md"
    exit 1
fi
