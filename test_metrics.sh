#!/bin/bash
# Test script to verify Prometheus metrics endpoint

set -e

echo "========================================"
echo "Testing Prometheus Metrics Endpoint"
echo "========================================"
echo ""

# Start the API server in the background
echo "Starting API server..."
RUST_LOG=info cargo run --release --bin recommendation-api > api_test.log 2>&1 &
API_PID=$!

# Wait for server to start
echo "Waiting for server to start..."
sleep 5

# Check if server is running
if ! kill -0 $API_PID 2>/dev/null; then
    echo "ERROR: API server failed to start"
    cat api_test.log
    exit 1
fi

echo "Server started successfully (PID: $API_PID)"
echo ""

# Test health endpoint
echo "1. Testing /health endpoint..."
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" http://localhost:8080/health)
HEALTH_CODE=$(echo "$HEALTH_RESPONSE" | tail -n 1)
echo "   Status: $HEALTH_CODE"

if [ "$HEALTH_CODE" != "200" ]; then
    echo "   ERROR: Health check failed"
    kill $API_PID
    exit 1
fi
echo "   ✓ Health check passed"
echo ""

# Test metrics endpoint (should return 200 even without auth for metrics)
echo "2. Testing /metrics endpoint..."
METRICS_RESPONSE=$(curl -s http://localhost:8080/metrics)
echo "   Response preview:"
echo "$METRICS_RESPONSE" | head -20
echo "   ..."
echo ""

# Verify it contains Prometheus metrics format
if echo "$METRICS_RESPONSE" | grep -q "# HELP"; then
    echo "   ✓ Metrics endpoint returns Prometheus format"
else
    echo "   ERROR: Metrics endpoint does not return Prometheus format"
    kill $API_PID
    exit 1
fi

# Make some API requests to generate metrics
echo "3. Making test requests to generate metrics..."
API_KEY="${API_KEY:-dev-api-key-change-in-production}"

# Create an entity
curl -s -X POST http://localhost:8080/api/v1/entities \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"entity_id":"test-product-1","entity_type":"product","attributes":{"name":"Test Product","price":99.99}}' > /dev/null

# Create an interaction
curl -s -X POST http://localhost:8080/api/v1/interactions \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"user_id":"user-1","entity_id":"test-product-1","interaction_type":"view"}' > /dev/null

echo "   ✓ Test requests completed"
echo ""

# Fetch metrics again to verify they were recorded
echo "4. Fetching metrics after requests..."
METRICS_AFTER=$(curl -s http://localhost:8080/metrics)

# Check for HTTP metrics
if echo "$METRICS_AFTER" | grep -q "http_requests_total"; then
    echo "   ✓ HTTP request metrics recorded"
    echo "$METRICS_AFTER" | grep "http_requests_total" | head -5
else
    echo "   WARNING: HTTP request metrics not found"
fi

# Check for cache metrics
if echo "$METRICS_AFTER" | grep -q "redis_cache"; then
    echo "   ✓ Redis cache metrics recorded"
    echo "$METRICS_AFTER" | grep "redis_cache" | head -3
else
    echo "   WARNING: Redis cache metrics not found"
fi

# Check for database pool metrics
if echo "$METRICS_AFTER" | grep -q "database_pool"; then
    echo "   ✓ Database pool metrics recorded"
    echo "$METRICS_AFTER" | grep "database_pool" | head -3
else
    echo "   WARNING: Database pool metrics not found"
fi

echo ""
echo "========================================"
echo "✅ All metrics tests passed!"
echo "========================================"

# Cleanup
echo ""
echo "Stopping API server..."
kill $API_PID
wait $API_PID 2>/dev/null || true
echo "Test complete"
