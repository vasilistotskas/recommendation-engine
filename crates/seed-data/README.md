# Recommendation Engine - Sample Data Seeder

A CLI tool to seed the recommendation engine with realistic sample data for testing and demonstration purposes.

## Features

- Generates realistic e-commerce product data with attributes like:
  - Names, descriptions, categories, brands
  - Prices, ratings, stock status
  - Tags and colors
- Generates content data (articles) with:
  - Titles, content, categories, authors
  - Read time, view counts, tags
- Generates user interactions including:
  - Views, add-to-cart, purchases, likes
  - Realistic timestamps (last 30 days)
  - Device and source metadata
- Supports multi-tenancy
- Batch processing for efficient bulk imports
- Progress tracking and error reporting

## Installation

Build the seed tool from the workspace root:

```bash
cargo build --release -p seed-data
```

The binary will be available at `target/release/seed-data` (or `seed-data.exe` on Windows).

## Usage

### Basic Usage

Seed with default values (assumes API is running at http://localhost:8080):

```bash
./target/release/seed-data
```

This will generate:
- 100 products
- 50 articles
- 500 interactions from 50 users

### Custom Configuration

```bash
./target/release/seed-data \
  --api-url http://localhost:8080 \
  --num-products 200 \
  --num-articles 100 \
  --num-users 100 \
  --num-interactions 1000 \
  --batch-size 100
```

### With Authentication

If your API requires authentication:

```bash
./target/release/seed-data --api-key your-api-key-here
```

Or use an environment variable:

```bash
export API_KEY=your-api-key-here
./target/release/seed-data
```

### Multi-Tenancy

Seed data for a specific tenant:

```bash
./target/release/seed-data --tenant-id tenant_xyz
```

## CLI Options

| Option | Default | Description |
|--------|---------|-------------|
| `--api-url` | `http://localhost:8080` | Base URL of the recommendation engine API |
| `--api-key` | None | API key for authentication (can use `API_KEY` env var) |
| `--tenant-id` | None | Tenant ID for multi-tenancy |
| `--num-products` | `100` | Number of products to generate |
| `--num-articles` | `50` | Number of articles to generate |
| `--num-users` | `50` | Number of users to simulate |
| `--num-interactions` | `500` | Number of interactions to generate |
| `--batch-size` | `100` | Batch size for bulk imports |

## Sample Data

### Products

Each product includes:
- Entity ID: `product_1`, `product_2`, etc.
- Category: Electronics, Clothing, Books, Home & Kitchen, Sports, Toys, Beauty, Automotive
- Brand: TechPro, StyleMax, HomeEssentials, SportFit, BeautyGlow, AutoMaster
- Price: Random between $10 - $1000
- Rating: Random between 3.0 - 5.0
- Stock status: 90% in stock
- Tags: 2-5 random descriptive tags
- Optional color attribute

### Articles

Each article includes:
- Entity ID: `article_1`, `article_2`, etc.
- Category: Technology, Business, Science, Health, Entertainment, Sports, Politics, Lifestyle
- Author: Random author name
- Read time: 3-15 minutes
- View count: 100-10,000 views
- Tags: 3-6 random descriptive tags

### Interactions

Each interaction includes:
- User ID: `user_1` through `user_N`
- Entity reference: 70% products, 30% articles
- Type: view, add_to_cart, purchase, like
- Timestamp: Random within last 30 days
- Metadata: source (web), device (desktop/mobile)

## Example Output

```
╔═══════════════════════════════════════════════════════════╗
║   Recommendation Engine - Sample Data Seeder             ║
╚═══════════════════════════════════════════════════════════╝

Configuration:
  API URL: http://localhost:8080
  Tenant ID: default
  Products: 100
  Articles: 50
  Users: 50
  Interactions: 500
  Batch Size: 100

Generating sample data...
✓ Generated 100 products
✓ Generated 50 articles
✓ Generated 500 interactions

Seeding 150 entities in batches of 100...
Processing batch 1/2 (100 entities)... ✓ Success: 100/100 entities imported
Processing batch 2/2 (50 entities)... ✓ Success: 50/50 entities imported

Entity seeding complete:
  Total: 150
  Successful: 150
  Failed: 0

Seeding 500 interactions in batches of 100...
Processing batch 1/5 (100 interactions)... ✓ Success: 100/100 interactions imported
Processing batch 2/5 (100 interactions)... ✓ Success: 100/100 interactions imported
Processing batch 3/5 (100 interactions)... ✓ Success: 100/100 interactions imported
Processing batch 4/5 (100 interactions)... ✓ Success: 100/100 interactions imported
Processing batch 5/5 (100 interactions)... ✓ Success: 100/100 interactions imported

Interaction seeding complete:
  Total: 500
  Successful: 500
  Failed: 0

╔═══════════════════════════════════════════════════════════╗
║   Seeding Complete! 🎉                                    ║
╚═══════════════════════════════════════════════════════════╝

You can now:
  1. Get recommendations: GET /api/v1/recommendations/user/user_1
  2. View trending: GET /api/v1/recommendations/trending
  3. Find similar items: GET /api/v1/recommendations/entity/product_1
```

## Testing After Seeding

Once seeding is complete, you can test the recommendation engine:

### Get User Recommendations

```bash
curl http://localhost:8080/api/v1/recommendations/user/user_1?count=10
```

### Get Trending Entities

```bash
curl http://localhost:8080/api/v1/recommendations/trending?entity_type=product&count=10
```

### Find Similar Products

```bash
curl http://localhost:8080/api/v1/recommendations/entity/product_1?count=10
```

### Get Similar Articles

```bash
curl http://localhost:8080/api/v1/recommendations/entity/article_1?count=10
```

## Error Handling

The tool will:
- Report detailed errors for failed API requests
- Show success/failure counts for each batch
- Continue processing even if some batches fail
- Display a summary at the end

## Development

To run in development mode:

```bash
cargo run -p seed-data -- --num-products 20 --num-interactions 100
```

## Dependencies

- `reqwest` - HTTP client for API calls
- `fake` - Fake data generation
- `rand` - Random number generation
- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `serde/serde_json` - JSON serialization
- `chrono` - Date/time handling
- `anyhow` - Error handling
