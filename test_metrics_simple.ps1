# Simple PowerShell script to test metrics endpoint
Write-Host "========================================"
Write-Host "Testing Prometheus Metrics Endpoint"
Write-Host "========================================"
Write-Host ""

Write-Host "Testing /metrics endpoint..."
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/metrics" -UseBasicParsing

    Write-Host "Status Code: $($response.StatusCode)"
    Write-Host ""
    Write-Host "Response preview (first 30 lines):"
    Write-Host "----------------------------------------"
    $lines = $response.Content -split "`n"
    $lines[0..29] | ForEach-Object { Write-Host $_ }
    Write-Host "----------------------------------------"
    Write-Host ""

    # Check for Prometheus format
    if ($response.Content -match "# HELP") {
        Write-Host "✓ Metrics endpoint returns Prometheus format" -ForegroundColor Green
    } else {
        Write-Host "✗ Metrics endpoint does not return Prometheus format" -ForegroundColor Red
        exit 1
    }

    # Check for specific metrics
    $metricsFound = @()

    if ($response.Content -match "http_requests_total") {
        $metricsFound += "HTTP request metrics"
    }

    if ($response.Content -match "redis_cache") {
        $metricsFound += "Redis cache metrics"
    }

    if ($response.Content -match "database_pool") {
        $metricsFound += "Database pool metrics"
    }

    if ($metricsFound.Count -gt 0) {
        Write-Host ""
        Write-Host "Metrics found:" -ForegroundColor Green
        $metricsFound | ForEach-Object { Write-Host "  ✓ $_" -ForegroundColor Green }
    }

    Write-Host ""
    Write-Host "========================================"
    Write-Host "✅ Metrics test passed!" -ForegroundColor Green
    Write-Host "========================================"

} catch {
    Write-Host "✗ Error testing metrics endpoint: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Make sure the API server is running:"
    Write-Host "  cargo run --release --bin recommendation-api"
    exit 1
}
