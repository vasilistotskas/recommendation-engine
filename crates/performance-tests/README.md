# Performance Validation Tests

This crate contains performance validation tools for the recommendation engine, validating that the system meets the specified SLA requirements.

## Requirements Validated

Based on Requirements 8.1 and 8.2:

1. **Response Time**: p95 latency < 200ms at 1000 req/s
2. **Throughput**: ≥1000 requests per second sustained
3. **Memory Usage**: <2GB for 100,000 entities and 1,000,000 interactions

## Usage

### Prerequisites

1. Start the recommendation engine:
```bash
cd recommendation-engine
cargo run --release --bin recommendation-api
```

2. Ensure PostgreSQL and Redis are running (via docker-compose or locally)

### Running Performance Validation

```bash
# Full validation with 100k entities (recommended)
cargo run --release --bin performance-validator -- --entities 100000 --duration 60

# Quick validation with fewer entities
cargo run --release --bin performance-validator -- --entities 10000 --duration 30

# Skip entity creation if data already exists
cargo run --release --bin performance-validator -- --skip-setup

# Custom service URL
cargo run --release --bin performance-validator -- --url http://localhost:8080

# Full options
cargo run --release --bin performance-validator -- \
  --url http://localhost:8080 \
  --entities 100000 \
  --concurrency 1000 \
  --duration 60
```

### Command Line Options

- `--url, -u`: Base URL of the recommendation engine (default: http://localhost:8080)
- `--entities, -e`: Number of entities to create for testing (default: 100000)
- `--concurrency, -c`: Number of concurrent requests for throughput test (default: 1000)
- `--duration, -d`: Duration of throughput test in seconds (default: 60)
- `--skip-setup`: Skip entity creation and use existing data

## Test Details

### Test 1: Response Time Validation

- Sends 100 sequential recommendation requests
- Measures latency for each request
- Calculates p50, p95, p99, and max latency
- **Pass Criteria**: p95 < 200ms

### Test 2: Throughput Validation

- Runs concurrent requests for specified duration (default 60s)
- Uses configurable concurrency level (default 1000)
- Measures successful requests per second
- Tracks p95 latency under load
- **Pass Criteria**: ≥1000 req/s sustained

### Test 3: Memory Usage Validation

- Checks process memory usage
- Calculates memory per entity
- **Pass Criteria**: <2GB for 100k entities (scales proportionally for smaller datasets)

## Output

The tool provides:
- Real-time progress bars for each test
- Detailed metrics (latency percentiles, throughput, memory)
- Color-coded pass/fail status
- Summary report with all results

Example output:
```
================================================================================
Recommendation Engine Performance Validation
================================================================================

Checking service health...
✓ Service is healthy

Setting up test data (100000 entities)...
✓ Test data created

Test 1: Response Time Validation
--------------------------------------------------------------------------------
Running latency test (100 requests)...
  p50 latency: 45.23ms
  p95 latency: 156.78ms
  p99 latency: 189.45ms
  max latency: 198.23ms
  Requirement: p95 < 200ms
  Status: PASS

Test 2: Throughput Validation
--------------------------------------------------------------------------------
Running throughput test (1000 concurrent requests for 60s)...
  Total requests: 75234
  Successful: 75234
  Failed: 0
  Throughput: 1253.90 req/s
  p95 latency: 167.34ms
  Requirement: ≥1000 req/s
  Status: PASS

Test 3: Memory Usage Validation
--------------------------------------------------------------------------------
Checking memory usage...
  Process: recommendation-api (PID: 12345)
  Memory usage: 1847.23 MB
  Entity count: 100000
  Memory per entity: 18.47 KB
  Requirement: <2GB for 100k entities
  Status: PASS

================================================================================
Performance Validation Summary
================================================================================

  ✓ PASS Response Time: p95: 156.78ms (requirement: <200ms)
  ✓ PASS Throughput: 1253.90 req/s (requirement: ≥1000 req/s)
  ✓ PASS Memory Usage: 1.80 GB (requirement: <2GB for 100k entities)

================================================================================
✓ All performance requirements met!
================================================================================
```

## Troubleshooting

### Service Not Running
```
Error: Failed to connect to service
```
**Solution**: Start the recommendation engine with `cargo run --release --bin recommendation-api`

### Low Throughput
If throughput is below 1000 req/s:
- Ensure the service is running in release mode (`--release`)
- Check database connection pool settings
- Verify Redis is running and accessible
- Check system resources (CPU, network)

### High Memory Usage
If memory exceeds 2GB for 100k entities:
- Check for memory leaks in the application
- Verify cache TTL settings are appropriate
- Review database connection pool size
- Consider enabling memory profiling

### Process Not Found
If memory test shows "process not found":
- The service may be running in a container
- Memory validation will be skipped
- Consider using container monitoring tools instead

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/performance.yml
- name: Run Performance Tests
  run: |
    # Start services
    docker-compose up -d
    
    # Wait for services to be ready
    sleep 10
    
    # Run performance validation
    cargo run --release --bin performance-validator -- \
      --entities 10000 \
      --duration 30
```

## Notes

- Always run in release mode for accurate performance measurements
- Ensure the system is not under other load during testing
- Results may vary based on hardware and system configuration
- For production validation, use 100k entities and 60s duration
- The tool automatically creates test data if not present
