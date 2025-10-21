# Integration Tests Runner Script
# This script sets up the environment and runs integration tests

param(
    [string]$DatabaseUrl = "postgresql://postgres:postgres@localhost:5432/recommendations_test",
    [string]$RedisUrl = "redis://localhost:6379/1",
    [switch]$SkipMigrations = $false,
    [switch]$Verbose = $false
)

Write-Host "=== Recommendation Engine Integration Tests ===" -ForegroundColor Cyan
Write-Host ""

# Set environment variables
$env:TEST_DATABASE_URL = $DatabaseUrl
$env:TEST_REDIS_URL = $RedisUrl
$env:DATABASE_URL = $DatabaseUrl

if ($Verbose) {
    $env:RUST_LOG = "debug"
}

Write-Host "Configuration:" -ForegroundColor Yellow
Write-Host "  Database: $DatabaseUrl"
Write-Host "  Redis: $RedisUrl"
Write-Host ""

# Check if sqlx-cli is installed
$sqlxInstalled = Get-Command sqlx -ErrorAction SilentlyContinue
if (-not $sqlxInstalled) {
    Write-Host "WARNING: sqlx-cli not found. Install with: cargo install sqlx-cli" -ForegroundColor Yellow
    Write-Host ""
}

# Run migrations unless skipped
if (-not $SkipMigrations -and $sqlxInstalled) {
    Write-Host "Running database migrations..." -ForegroundColor Green
    try {
        sqlx migrate run
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✓ Migrations applied successfully" -ForegroundColor Green
        } else {
            Write-Host "✗ Migration failed with exit code $LASTEXITCODE" -ForegroundColor Red
            Write-Host ""
            Write-Host "Common issues:" -ForegroundColor Yellow
            Write-Host "  1. PostgreSQL is not running"
            Write-Host "  2. Database 'recommendations_test' does not exist"
            Write-Host "  3. pgvector extension is not installed"
            Write-Host ""
            Write-Host "To create the database:" -ForegroundColor Cyan
            Write-Host "  createdb recommendations_test"
            Write-Host ""
            Write-Host "To install pgvector:" -ForegroundColor Cyan
            Write-Host "  See: https://github.com/pgvector/pgvector#installation"
            Write-Host ""
            exit 1
        }
    } catch {
        Write-Host "✗ Error running migrations: $_" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
}

# Run tests
Write-Host "Running integration tests..." -ForegroundColor Green
Write-Host ""

$testArgs = @(
    "test",
    "-p", "recommendation-integration-tests",
    "--",
    "--test-threads=1"
)

if ($Verbose) {
    $testArgs += "--nocapture"
}

& cargo @testArgs

$exitCode = $LASTEXITCODE

Write-Host ""
if ($exitCode -eq 0) {
    Write-Host "=== All tests passed! ===" -ForegroundColor Green
} else {
    Write-Host "=== Tests failed ===" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Ensure PostgreSQL is running with pgvector extension"
    Write-Host "  2. Ensure Redis is running"
    Write-Host "  3. Check database credentials in connection string"
    Write-Host "  4. Run with -Verbose flag for detailed output"
    Write-Host ""
    Write-Host "For more information, see:" -ForegroundColor Cyan
    Write-Host "  - INTEGRATION_TESTS.md"
    Write-Host "  - crates/integration-tests/tests/README.md"
}

exit $exitCode
