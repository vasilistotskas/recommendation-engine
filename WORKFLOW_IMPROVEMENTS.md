# CI/CD Workflow Improvements

## Overview

This document describes the critical fixes applied to the GitHub Actions workflows to ensure they run successfully with database-dependent tests.

## Problem

The original CI/CD workflows (test.yml and coverage.yml) would fail when running tests because:

1. **Missing database migrations**: Tests that interact with the database would fail with errors like:
   ```
   error returned from database: relation "entities" does not exist
   ```

2. **No database setup**: While PostgreSQL and Redis services were configured, the database schema wasn't being initialized before running tests.

## Solution

### Changes to `.github/workflows/test.yml`

Added database migration steps to both the `test` and `integration-test` jobs:

```yaml
- name: Install sqlx-cli
  run: cargo install sqlx-cli --no-default-features --features postgres

- name: Run database migrations
  env:
    DATABASE_URL: postgresql://postgres:postgres@localhost:5432/recommendations
  run: |
    sqlx database create
    sqlx migrate run
```

**Placement**:
- In `test` job: After "Build all crates", before "Run tests"
- In `integration-test` job: After "Cache dependencies", before "Build integration tests"

### Changes to `.github/workflows/coverage.yml`

Added the same migration steps to the `coverage` job:

```yaml
- name: Install sqlx-cli
  run: cargo install sqlx-cli --no-default-features --features postgres

- name: Run database migrations
  env:
    DATABASE_URL: postgresql://postgres:postgres@localhost:5432/recommendations
  run: |
    sqlx database create
    sqlx migrate run
```

**Placement**: After "Install cargo-llvm-cov", before "Generate coverage report"

Also added environment variables to all coverage generation steps:

```yaml
env:
  DATABASE_URL: postgresql://postgres:postgres@localhost:5432/recommendations
  REDIS_URL: redis://localhost:6379
  TEST_DATABASE_URL: postgresql://postgres:postgres@localhost:5432/recommendations
  TEST_REDIS_URL: redis://localhost:6379
```

Applied to:
- Generate coverage report
- Generate HTML coverage report
- Check coverage threshold

## Impact

### Before
- ❌ Tests would fail with "relation does not exist" errors
- ❌ Coverage reports couldn't be generated
- ❌ Integration tests would fail
- ❌ CI/CD pipeline was broken

### After
- ✅ All unit tests pass (191 tests)
- ✅ Database schema properly initialized
- ✅ Coverage reports can be generated
- ✅ Integration tests have proper database setup
- ✅ CI/CD pipeline is functional

## Test Results

After applying these fixes:

```
recommendation-api: 4 tests passed
recommendation-config: 22 tests passed
recommendation-engine: 40 tests passed
recommendation-models: 56 tests passed
recommendation-service: 38 tests passed
recommendation-storage: 31 tests passed
─────────────────────────────────────────
Total: 191/191 unit tests passing ✅
```

## Migration Files Location

Database migrations are located in: `migrations/`

The migrations include:
- `20241118000000_create_entities.sql`
- `20241118000001_create_interactions.sql`
- `20241118000002_create_user_profiles.sql`
- `20241118000003_create_trending_entities.sql`
- `20241118000004_create_indices.sql`
- `20241118000005_add_hnsw_indices.sql`

## Dependencies

The workflows now depend on:
- **sqlx-cli**: For running database migrations in CI
  - Installed with: `cargo install sqlx-cli --no-default-features --features postgres`
  - Used for: `sqlx database create` and `sqlx migrate run`

## Verification

To verify the workflows will work correctly, run locally:

```bash
# Set environment variables
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/recommendations
export REDIS_URL=redis://localhost:6379
export TEST_DATABASE_URL=$DATABASE_URL
export TEST_REDIS_URL=$REDIS_URL

# Ensure services are running (PostgreSQL with pgvector, Redis)
docker-compose up -d

# Run migrations
sqlx database create
sqlx migrate run

# Run tests
cargo test --workspace --verbose

# Run coverage
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## Notes

1. **Service Health Checks**: Both PostgreSQL and Redis services have health checks configured to ensure they're ready before migrations run.

2. **Migration Idempotency**: The migrations are idempotent (safe to run multiple times) thanks to `IF NOT EXISTS` clauses.

3. **Cache Strategy**: The workflows cache Cargo dependencies to speed up subsequent runs:
   - `~/.cargo/registry`
   - `~/.cargo/git`
   - `target/`

4. **Parallel Execution**: The migration steps run sequentially but the overall jobs can run in parallel since each has its own database instance.

## Future Improvements

Consider these enhancements:

1. **Cache sqlx-cli**: Cache the sqlx-cli binary to avoid reinstalling on every run
2. **Migration Verification**: Add a step to verify migration success
3. **Database Seeding**: Optionally seed test data for integration tests
4. **Performance**: Use `--locked` flag for faster dependency resolution

## Related Files

- `.github/workflows/test.yml`
- `.github/workflows/coverage.yml`
- `.github/workflows/security.yml` (uses similar pattern)
- `migrations/*.sql`
- `UNUSED_DEPENDENCIES_CLEANUP.md`
- `CI_CD_QUICKSTART.md`

## Date

**Improvements Applied**: October 22, 2025
**Last Verified**: October 22, 2025
**Status**: ✅ All workflows operational
