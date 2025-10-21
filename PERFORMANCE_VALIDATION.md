# Performance Validation Report

This document describes the performance validation process for the Recommendation Engine and provides guidelines for running and interpreting performance tests.

## Requirements

Based on the system requirements (8.1 and 8.2), the recommendation engine must meet the following performance criteria:

### 1. Response Time (Requirement 8.1)
- **Metric**: p95 latency
- **Target**: < 200 milliseconds
- **Load**: 1000 requests per second
- **Description**: 95% of recommendation requests must complete within 200ms under normal load

### 2. Throughput (Requirement 8.1)
- **Metric**: Requests per second
- **Target**: ≥ 1000 req/s
- **Description**: The system must sustain at least 1000 requests per second with acceptable latency

### 3. Memory Usage (Requirement 8.2)
- **Metric**: Process memory consumption
- **Target**: < 2 GB
- **Dataset**: 100,000 entities and 1,000,000 interactions
- **Description**: The system must operate within 2GB of memory for the specified dataset size

## Validation Tool

The performance validation tool is located in `crates/performance-tests/` and provides automated testing of all three requirements.

### Quick Start

```bash
# Windows PowerShell
.\run_performance_tests.ps1

# Linux/Mac
chmod +x run_performance_tests.sh
./run_performance_tests.sh

# Or directly with cargo
cargo run --release --bin performance-validator
```

### Quick Validation (Development)

For faster validation during development:

```bash
# Windows
.\run_performance_tests.ps1 -Quick

# Linux/Mac
./run_performance_tests.sh --quick
```

This uses 10,000 entities and 30-second duration for faster feedback.

## Test Methodology

### Test 1: Response Time Validation

**Approach:**
1. Send 100 sequential recommendation requests
2. Measure latency for each request using high-precision timers
3. Calculate latency percentiles (p50, p95, p99, max)
4. Verify p95 < 200ms

**Request Pattern:**
- Algorithm: Hybrid (collaborative + content-based)
- Count: 10 recommendations per request
- Users: Rotates through 50 different users
- Endpoint: `GET /api/v1/recommendations/user/{id}?algorithm=hybrid&count=10`

**Pass Criteria:**
- p95 latency must be less than 200 milliseconds

### Test 2: Throughput Validation

**Approach:**
1. Launch 1000 concurrent workers (configurable)
2. Each worker continuously sends requests for 60 seconds (configurable)
3. Track successful and failed requests
4. Calculate requests per second
5. Monitor p95 latency under load

**Request Pattern:**
- Concurrent workers: 1000 (default)
- Duration: 60 seconds (default)
- Users: Rotates through 1000 different users
- Algorithm: Hybrid
- Count: 10 recommendations per request

**Pass Criteria:**
- Sustained throughput ≥ 1000 requests per second
- System remains stable throughout test duration

### Test 3: Memory Usage Validation

**Approach:**
1. Query system process information
2. Identify recommendation engine process
3. Measure resident memory usage
4. Calculate memory per entity
5. Verify total memory < 2GB for 100k entities

**Measurement:**
- Uses system APIs to get process memory (RSS)
- Accounts for entity count to scale requirements
- Provides memory per entity metric

**Pass Criteria:**
- Memory usage < 2GB for 100,000 entities
- Scales proportionally for smaller datasets

## Test Data Setup

The validation tool automatically creates test data if not present:

### Entities
- Type: Products
- Count: Configurable (default 100,000)
- Attributes:
  - name: "Product {id}"
  - category: 10 categories (category_0 to category_9)
  - price: Varies from $9.99 to $1009.99
  - brand: 50 brands (brand_0 to brand_49)
  - tags: 2 tags per product from 20 possible tags

### Interactions
- Users: 10% of entity count (minimum 100)
- Interactions per user: 5-10 random interactions
- Type: "view" interactions
- Distribution: Each user interacts with different products

### Model Updates
- After data creation, waits 15 seconds for model updates
- Ensures similarity matrices and caches are populated

## Interpreting Results

### Successful Validation

```
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

All three tests passed - the system meets performance requirements.

### Failed Validation Examples

#### High Latency
```
  ✗ FAIL Response Time: p95: 245.67ms (requirement: <200ms)
```

**Possible Causes:**
- Database query optimization needed
- Insufficient database connection pool
- Cache not warming properly
- Network latency issues
- System under other load

**Solutions:**
- Review slow query logs
- Increase database connection pool size
- Verify Redis cache is working
- Check pgvector index configuration
- Ensure system resources are available

#### Low Throughput
```
  ✗ FAIL Throughput: 847.23 req/s (requirement: ≥1000 req/s)
```

**Possible Causes:**
- CPU bottleneck
- Database connection exhaustion
- Inefficient algorithm implementation
- Lock contention
- Network bandwidth limitation

**Solutions:**
- Profile CPU usage
- Increase worker threads
- Optimize hot code paths
- Review database connection settings
- Consider horizontal scaling

#### High Memory Usage
```
  ✗ FAIL Memory Usage: 2.34 GB (requirement: <2GB for 100k entities)
```

**Possible Causes:**
- Memory leak
- Excessive caching
- Large connection pools
- Inefficient data structures

**Solutions:**
- Run memory profiler
- Review cache TTL settings
- Adjust connection pool sizes
- Optimize data structures
- Check for memory leaks

## Performance Optimization Tips

### Database Optimization
1. Ensure pgvector HNSW indices are created
2. Configure appropriate connection pool size (20-50 connections)
3. Use connection pooling with idle timeout
4. Enable query result caching

### Cache Optimization
1. Verify Redis is running and accessible
2. Configure appropriate TTL values (5-60 minutes)
3. Monitor cache hit rates (target >80%)
4. Use multi-layer caching strategy

### Application Optimization
1. Run in release mode (`--release`)
2. Enable LTO (Link Time Optimization)
3. Use appropriate number of worker threads
4. Minimize allocations in hot paths
5. Use async I/O throughout

### System Configuration
1. Ensure sufficient CPU cores (4+ recommended)
2. Allocate adequate memory (4GB+ recommended)
3. Use SSD for database storage
4. Configure OS network stack for high throughput

## Continuous Performance Monitoring

### CI/CD Integration

Add performance validation to your CI pipeline:

```yaml
# .github/workflows/performance.yml
name: Performance Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: pgvector/pgvector:pg17
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
      
      - name: Build
        run: cargo build --release
      
      - name: Start Service
        run: |
          cargo run --release --bin recommendation-api &
          sleep 10
      
      - name: Run Performance Tests
        run: |
          cargo run --release --bin performance-validator -- \
            --entities 10000 \
            --duration 30
```

### Production Monitoring

Monitor these metrics in production:

1. **Request Latency**
   - p50, p95, p99 latencies
   - Alert if p95 > 200ms

2. **Throughput**
   - Requests per second
   - Alert if < 1000 req/s

3. **Memory Usage**
   - Process RSS memory
   - Alert if > 2GB

4. **Cache Performance**
   - Cache hit rate
   - Alert if < 80%

5. **Database Performance**
   - Query duration
   - Connection pool utilization
   - Alert if pool > 90% utilized

## Benchmarking Best Practices

1. **Consistent Environment**
   - Use dedicated hardware for benchmarking
   - Minimize background processes
   - Use consistent network conditions

2. **Warm-up Period**
   - Allow caches to warm up
   - Run warm-up requests before measurement
   - Wait for model updates to complete

3. **Multiple Runs**
   - Run tests multiple times
   - Calculate average and standard deviation
   - Identify outliers

4. **Realistic Data**
   - Use production-like data distributions
   - Include variety in entity attributes
   - Simulate realistic user behavior

5. **Load Patterns**
   - Test with various concurrency levels
   - Include burst traffic scenarios
   - Test sustained load over time

## Troubleshooting Guide

### Service Won't Start

```
Error: Failed to connect to service
```

**Check:**
1. Is PostgreSQL running? `docker-compose ps`
2. Is Redis running? `redis-cli ping`
3. Are migrations applied? `sqlx migrate run`
4. Check logs for errors

### Build Failures

```
Error: could not compile `performance-validator`
```

**Solutions:**
1. Update Rust: `rustup update`
2. Clean build: `cargo clean`
3. Check dependencies: `cargo check`

### Inconsistent Results

**Causes:**
- System under other load
- Network variability
- Database not optimized
- Insufficient warm-up

**Solutions:**
- Run on dedicated system
- Increase warm-up period
- Run multiple times and average
- Check system resources

## Additional Resources

- [Cargo.toml](./crates/performance-tests/Cargo.toml) - Dependencies
- [README](./crates/performance-tests/README.md) - Detailed usage
- [Design Document](../.kiro/specs/recommendation-engine/design.md) - Architecture
- [Requirements](../.kiro/specs/recommendation-engine/requirements.md) - Full requirements

## Support

For issues or questions:
1. Check logs: `cargo run --release --bin recommendation-api`
2. Review metrics: `curl http://localhost:8080/metrics`
3. Check health: `curl http://localhost:8080/health`
4. Open an issue with performance test output
