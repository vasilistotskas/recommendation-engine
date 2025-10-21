#!/bin/bash
# Performance Validation Script for Linux/Mac

set -e

# Default values
URL="http://localhost:8080"
ENTITIES=100000
CONCURRENCY=1000
DURATION=60
SKIP_SETUP=""
QUICK=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --url)
            URL="$2"
            shift 2
            ;;
        --entities)
            ENTITIES="$2"
            shift 2
            ;;
        --concurrency)
            CONCURRENCY="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --skip-setup)
            SKIP_SETUP="--skip-setup"
            shift
            ;;
        --quick)
            QUICK=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Quick mode uses smaller dataset
if [ "$QUICK" = true ]; then
    ENTITIES=10000
    DURATION=30
fi

echo "================================================================================================"
echo "Recommendation Engine Performance Validation"
echo "================================================================================================"
echo ""

if [ "$QUICK" = true ]; then
    echo -e "\033[33mRunning in QUICK mode (10k entities, 30s duration)\033[0m"
    echo ""
fi

# Check if service is running
echo -e "\033[33mChecking if recommendation engine is running...\033[0m"
if curl -s -f "$URL/health" > /dev/null 2>&1; then
    echo -e "\033[32m✓ Service is running at $URL\033[0m"
else
    echo -e "\033[31m✗ Service is not running at $URL\033[0m"
    echo ""
    echo -e "\033[33mPlease start the service first:\033[0m"
    echo "  cd recommendation-engine"
    echo "  cargo run --release --bin recommendation-api"
    echo ""
    exit 1
fi

echo ""

# Build the performance validator
echo -e "\033[33mBuilding performance validator...\033[0m"
cargo build --release --bin performance-validator
echo -e "\033[32m✓ Build complete\033[0m"
echo ""

# Run performance tests
echo -e "\033[33mStarting performance validation...\033[0m"
echo ""

./target/release/performance-validator \
    --url "$URL" \
    --entities "$ENTITIES" \
    --concurrency "$CONCURRENCY" \
    --duration "$DURATION" \
    $SKIP_SETUP

echo ""
echo "================================================================================================"
echo "Performance validation complete!"
echo "================================================================================================"
