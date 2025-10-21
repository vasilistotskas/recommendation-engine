# Integration Tests

This directory contains end-to-end integration tests for the Recommendation Engine.

## Prerequisites

Before running the integration tests, ensure you have:

1. **PostgreSQL** with pgvector extension installed and running
2. **Redis** installed and running
3. A test database created

### Setup Test Database

```bash
# Create test database
createdb recommendations_test

# Or using psql
psql -U postgres -c "CREATE DATABASE recommendations_test;"
```

### Install pgvector Extension

```bash
# Connect to test database
psql -U postgres -d recommendations_test

# Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;
```

## Running Tests

### Run All Integration Tests

```bash
# From the recommendation-engine directory
cargo test --test integration_test

# With output
cargo test --test integration_test -- --nocapture

# Run specific test
cargo test --test integration_test test_complete_workflow_from_entity_creation_to_recommendations
```

### Environment Variables

The tests use environment variables from `.env.test` or fall back to defaults:

- `TEST_DATABASE_URL`: PostgreSQL connection string (default: `postgresql://localhost:5432/recommendations_test`)
- `TEST_REDIS_URL`: Redis connection string (default: `redis://localhost:6379`)

You can override these by setting environment variables:

```bash
TEST_DATABASE_URL=postgresql://user:pass@localhost:5432/test_db cargo test --test integration_test
```

## Test Coverage

The integration tests cover:

### 1. Complete Workflow Test
- Entity creation with attributes
- Recording user interactions
- Generating recommendations using all algorithms
- Verifying recommendation quality

### 2. Multi-Tenancy Isolation Test
- Creating entities for multiple tenants
- Recording interactions per tenant
- Verifying data isolation between tenants
- Ensuring recommendations don't leak across tenants

### 3. All Algorithms Test
- Collaborative filtering recommendations
- Content-based filtering recommendations
- Hybrid algorithm recommendations
- Score validation and quality checks

## Test Data Cleanup

Each test automatically cleans up its test data:
- Before the test runs (to ensure clean state)
- After the test completes (to avoid pollution)

Test data uses unique tenant IDs to avoid conflicts:
- `test_tenant_workflow`
- `tenant_a_isolation` / `tenant_b_isolation`
- `test_tenant_algorithms`

## Troubleshooting

### Database Connection Errors

If you see connection errors:
1. Verify PostgreSQL is running: `pg_isready`
2. Check the database exists: `psql -l | grep recommendations_test`
3. Verify pgvector extension: `psql -d recommendations_test -c "SELECT * FROM pg_extension WHERE extname = 'vector';"`

### Redis Connection Errors

If you see Redis errors:
1. Verify Redis is running: `redis-cli ping`
2. Check Redis is accessible: `redis-cli -h localhost -p 6379 ping`

### Migration Errors

If migrations fail:
1. Ensure pgvector extension is installed
2. Check migration files in `migrations/` directory
3. Manually run migrations: `sqlx migrate run --database-url postgresql://localhost:5432/recommendations_test`

### Test Failures

If tests fail:
1. Check test output with `--nocapture` flag
2. Verify test database is clean (no leftover data)
3. Ensure Redis cache is cleared
4. Check logs for detailed error messages

## CI/CD Integration

These tests are designed to run in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Setup PostgreSQL
  run: |
    sudo apt-get install postgresql-14 postgresql-14-pgvector
    sudo systemctl start postgresql
    sudo -u postgres createdb recommendations_test
    sudo -u postgres psql -d recommendations_test -c "CREATE EXTENSION vector;"

- name: Setup Redis
  run: |
    sudo apt-get install redis-server
    sudo systemctl start redis

- name: Run Integration Tests
  run: cargo test --test integration_test
  env:
    TEST_DATABASE_URL: postgresql://postgres@localhost/recommendations_test
    TEST_REDIS_URL: redis://localhost:6379
```

## Performance Considerations

Integration tests may take longer than unit tests because they:
- Connect to real databases
- Run actual migrations
- Perform vector similarity searches
- Execute complex recommendation algorithms

Typical test execution time: 5-15 seconds per test.
