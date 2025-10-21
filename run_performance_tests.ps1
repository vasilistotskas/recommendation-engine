#!/usr/bin/env pwsh
# Performance Validation Script for Windows PowerShell

param(
    [string]$Url = "http://localhost:8080",
    [int]$Entities = 10000,
    [int]$Concurrency = 100,
    [int]$Duration = 60,
    [switch]$SkipSetup,
    [switch]$Quick
)

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Recommendation Engine Performance Validation" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

if ($Quick) {
    $Entities = 10000
    $Duration = 30
    Write-Host "Running in QUICK mode (10k entities, 30s duration)" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "Checking if recommendation engine is running..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "$Url/health" -Method Get -TimeoutSec 5 -ErrorAction Stop
    if ($response.StatusCode -eq 200) {
        Write-Host "OK Service is running at $Url" -ForegroundColor Green
    }
} catch {
    Write-Host "X Service is not running at $Url" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please start the service first" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Building performance validator..." -ForegroundColor Yellow
cargo build --release --bin performance-validator
if ($LASTEXITCODE -ne 0) {
    Write-Host "X Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "OK Build complete" -ForegroundColor Green
Write-Host ""

Write-Host "Starting performance validation..." -ForegroundColor Yellow
Write-Host ""

$testArgs = @(
    "--url", $Url,
    "--entities", $Entities,
    "--concurrency", $Concurrency,
    "--duration", $Duration
)

if ($SkipSetup) {
    $testArgs += "--skip-setup"
}

& ".\target\release\performance-validator.exe" $testArgs

Write-Host ""
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "Performance validation complete!" -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
