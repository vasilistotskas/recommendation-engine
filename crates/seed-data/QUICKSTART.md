# Quick Start Guide - Sample Data Seeder

This guide will help you quickly get started with seeding your recommendation engine with sample data.

## Prerequisites

1. The recommendation engine API must be running
2. You have built the seed-data tool (see main README)

## Basic Seeding (Recommended for Testing)

The simplest way to seed with default values:

```bash
# From the workspace root
cargo run -p seed-data
```

This will:
- Generate 100 products
- Generate 50 articles
- Simulate 50 users
- Create 500 realistic interactions
- Import all data to http://localhost:8080

## Larger Dataset (Recommended for Demo)

For a more substantial demo dataset:

```bash
cargo run -p seed-data -- \
  --num-products 500 \
  --num-articles 200 \
  --num-users 200 \
  --num-interactions 5000
```

## Production-Like Dataset

For testing at scale:

```bash
cargo run --release -p seed-data -- \
  --num-products 10000 \
  --num-articles 5000 \
  --num-users 5000 \
  --num-interactions 100000 \
  --batch-size 500
```

Note: This may take several minutes to complete.

## With Authentication

If your API requires an API key:

```bash
export API_KEY=your-secret-api-key
cargo run -p seed-data
```

Or pass it directly:

```bash
cargo run -p seed-data -- --api-key your-secret-api-key
```

## Multi-Tenant Setup

Seed data for a specific tenant:

```bash
cargo run -p seed-data -- --tenant-id tenant_a
```

Seed multiple tenants (run multiple times):

```bash
# Tenant A
cargo run -p seed-data -- --tenant-id tenant_a --num-products 200

# Tenant B
cargo run -p seed-data -- --tenant-id tenant_b --num-products 300

# Tenant C
cargo run -p seed-data -- --tenant-id tenant_c --num-products 150
```

## Different API URL

If your API is running on a different host/port:

```bash
cargo run -p seed-data -- --api-url http://api.example.com:3000
```

## Verify Seeded Data

After seeding completes, test the recommendations:

### Get User Recommendations

```bash
curl http://localhost:8080/api/v1/recommendations/user/user_1?count=10 | jq
```

### Get Trending Products

```bash
curl http://localhost:8080/api/v1/recommendations/trending?entity_type=product&count=20 | jq
```

### Find Similar Items

```bash
curl http://localhost:8080/api/v1/recommendations/entity/product_1?count=10 | jq
```

### Check a Specific Entity

```bash
curl http://localhost:8080/api/v1/entities/product_1 | jq
```

## Sample Data Details

### Products Generated
- Categories: Electronics, Clothing, Books, Home & Kitchen, Sports, Toys, Beauty, Automotive
- Brands: TechPro, StyleMax, HomeEssentials, SportFit, BeautyGlow, AutoMaster
- Price range: $10 - $1000
- Rating range: 3.0 - 5.0 stars
- 90% in stock
- Random tags and optional colors

### Articles Generated
- Categories: Technology, Business, Science, Health, Entertainment, Sports, Politics, Lifestyle
- Read time: 3-15 minutes
- View counts: 100-10,000
- Multiple authors
- Random descriptive tags

### Interactions Generated
- Types: view, add_to_cart, purchase, like
- Distribution: 70% products, 30% articles
- Time range: Last 30 days
- Metadata: device (desktop/mobile), source (web)
- Chronologically ordered

## Troubleshooting

### Connection Refused
- Ensure the API is running: `docker-compose ps` or check your API process
- Verify the URL is correct: `--api-url http://localhost:8080`

### Authentication Failed
- Check your API key is correct
- Verify the API key is set: `echo $API_KEY`
- Try passing it directly with `--api-key`

### Slow Performance
- Increase batch size: `--batch-size 500`
- Use release mode: `cargo run --release -p seed-data`
- Check your database performance

### Some Imports Failed
- Check the error output for details
- Verify entities don't already exist (unique constraint)
- Check database connectivity

## Next Steps

After seeding:

1. **Test Recommendations**: Try the API endpoints shown above
2. **Monitor Performance**: Check response times and accuracy
3. **Adjust Algorithms**: Tune collaborative vs content-based weights
4. **Add Real Data**: Start importing your actual entities and interactions
5. **Test Cold Start**: Create a new user and see fallback recommendations

## Tips

- Start small (defaults) to verify everything works
- Use release mode for large datasets (10x faster)
- The tool is idempotent - you can run it multiple times
- Each run creates NEW data (doesn't clear old data)
- Interactions reference the generated entity IDs (product_1, article_1, etc.)
- User IDs are user_1 through user_N

## Clean Up

To start fresh, clear the database:

```bash
# Using docker-compose
docker-compose down -v
docker-compose up -d

# Run migrations again
cargo run -p recommendation-api -- --migrate

# Then re-seed
cargo run -p seed-data
```
