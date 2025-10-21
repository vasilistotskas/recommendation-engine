# Quick Start: Performance Validation

## Prerequisites

1. PostgreSQL with pgvector extension running
2. Redis running
3. Recommendation engine built

## Run Performance Tests

### Option 1: Using Convenience Scripts (Recommended)

**Windows:**
```powershell
# Full validation (100k entities, 60s duration)
.\run_performance_tests.ps1

# Quick validation (10k entities, 30s duration)
.\run_performance_tests.ps1 -Quick
```

**Linux/Mac:**
```bash
# Make script executable (first time only)
chmod +x run_performance_tests.sh

# Full validation
./run_performance_tests.sh

# Quick validation
./run_performance_tests.sh --quick
```

### Option 2: Using Cargo Directly

```bash
# Build the validator
cargo build --release --bin performance-validator

# Run with defaults
cargo run --release --bin performance-validator

# Run with custom settings
cargo run --release --bin performance-validator -- \
  --entities 50000 \
  --concurrency 500 \
  --duration 30
```

## What Gets Tested

1. **Response Time**: p95 latency must be < 200ms
2. **Throughput**: Must sustain ≥ 1000 requests/second
3. **Memory Usage**: Must use < 2GB for 100k entities

## Expected Output

```
✓ PASS Response Time: p95: 156.78ms (requirement: <200ms)
✓ PASS Throughput: 1253.90 req/s (requirement: ≥1000 req/s)
✓ PASS Memory Usage: 1.80 GB (requirement: <2GB for 100k entities)

✓ All performance requirements met!
```

## Troubleshooting

**Service not running:**
```bash
cd recommendation-engine
cargo run --release --bin recommendation-api
```

**Database not ready:**
```bash
docker-compose up -d postgres redis
```

**Need more details:**
See [PERFORMANCE_VALIDATION.md](./PERFORMANCE_VALIDATION.md) for comprehensive documentation.

## Command Line Options

```
--url <URL>              Service URL (default: http://localhost:8080)
--entities <COUNT>       Number of entities (default: 100000)
--concurrency <COUNT>    Concurrent requests (default: 1000)
--duration <SECONDS>     Test duration (default: 60)
--skip-setup            Skip test data creation
```

## Quick Reference

| Mode | Entities | Duration | Use Case |
|------|----------|----------|----------|
| Quick | 10,000 | 30s | Development, fast feedback |
| Default | 100,000 | 60s | Full validation, CI/CD |
| Custom | Variable | Variable | Specific scenarios |

## Next Steps

After validation passes:
1. Review detailed metrics in output
2. Check [PERFORMANCE_VALIDATION.md](./PERFORMANCE_VALIDATION.md) for optimization tips
3. Set up continuous monitoring in production
4. Integrate into CI/CD pipeline
