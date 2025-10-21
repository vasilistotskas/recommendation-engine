#!/usr/bin/env pwsh
# Start the recommendation API with performance testing configuration

Write-Host "Starting Recommendation Engine API for Performance Testing..." -ForegroundColor Cyan
Write-Host ""

# Load environment variables from .env.performance
if (Test-Path ".env.performance") {
    Get-Content ".env.performance" | ForEach-Object {
        if ($_ -match '^\s*([^#][^=]+)=(.*)$') {
            $name = $matches[1].Trim()
            $value = $matches[2].Trim()
            [Environment]::SetEnvironmentVariable($name, $value, "Process")
            Write-Host "  Set $name" -ForegroundColor Green
        }
    }
} else {
    Write-Host "Warning: .env.performance not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Configuration:" -ForegroundColor Cyan
Write-Host "  RATE_LIMIT_MAX_REQUESTS: $env:RATE_LIMIT_MAX_REQUESTS" -ForegroundColor White
Write-Host "  RATE_LIMIT_WINDOW_SECS: $env:RATE_LIMIT_WINDOW_SECS" -ForegroundColor White
Write-Host ""

# Start the API server
cargo run --release --bin recommendation-api
