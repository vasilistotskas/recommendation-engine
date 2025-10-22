# Recommendation Engine Python Client

Official Python client library for the GrooveShop Recommendation Engine API.

## Features

-  Full type hints with comprehensive type definitions
-  Modern Python 3.14+ with latest features
-  Async/await support with httpx
-  Automatic error handling with custom exceptions
-  Support for all API endpoints
-  Authentication support (API key)
-  Multi-tenancy support
-  Batch operations support
-  Context manager support for resource cleanup
-  Fully tested with pytest (85%+ coverage)
-  Type-checked with mypy
-  Linted with ruff

## Installation

```bash
pip install recommendation-engine-client
```

Or with uv (recommended):

```bash
uv add recommendation-engine-client
```

## Quick Start

```python
from recommendation_engine_client import RecommendationClient

# Initialize the client
async with RecommendationClient(
    base_url="http://localhost:8080",
    api_key="your-api-key",  # Optional
    timeout=30.0,  # Optional, default: 30.0
) as client:
    # Get recommendations for a user
    recommendations = await client.get_user_recommendations(
        "user_123",
        {"algorithm": "hybrid", "count": 10}
    )

    print(recommendations["recommendations"])
```

## Usage

### Client Initialization

```python
from recommendation_engine_client import RecommendationClient

# Basic initialization
client = RecommendationClient(base_url="https://api.example.com")

# With API key authentication
client = RecommendationClient(
    base_url="https://api.example.com",
    api_key="your-secret-api-key",
)

# With custom configuration
client = RecommendationClient(
    base_url="https://api.example.com",
    api_key="your-secret-api-key",
    timeout=60.0,
    headers={
        "X-Custom-Header": "value",
    },
)

# Using as context manager (recommended)
async with RecommendationClient(base_url="https://api.example.com") as client:
    # Use client
    pass
```

### Entity Operations

#### Create an Entity

```python
entity = await client.create_entity({
    "entity_id": "product_1",
    "entity_type": "product",
    "attributes": {
        "name": "Wireless Headphones",
        "category": "Electronics",
        "price": 99.99,
        "brand": "TechPro",
        "in_stock": True,
        "tags": ["wireless", "audio", "bluetooth"],
    },
    "tenant_id": "tenant_a",  # Optional
})
```

#### Get an Entity

```python
entity = await client.get_entity("product_1", tenant_id="tenant_a")
print(entity["attributes"])
```

#### Update an Entity

```python
updated_entity = await client.update_entity(
    "product_1",
    {
        "attributes": {
            "price": 89.99,
            "in_stock": False,
        },
        "tenant_id": "tenant_a",
    },
)
```

#### Delete an Entity

```python
await client.delete_entity("product_1", tenant_id="tenant_a")
```

#### Bulk Import Entities

```python
result = await client.bulk_import_entities({
    "entities": [
        {
            "entity_id": "product_1",
            "entity_type": "product",
            "attributes": {
                "name": "Product 1",
                "price": 29.99,
            },
        },
        {
            "entity_id": "product_2",
            "entity_type": "product",
            "attributes": {
                "name": "Product 2",
                "price": 39.99,
            },
        },
    ],
    "tenant_id": "tenant_a",
})

print(f"Imported {result['successful']}/{result['total_records']} entities")
```

### Interaction Operations

#### Create an Interaction

```python
interaction = await client.create_interaction({
    "user_id": "user_123",
    "entity_id": "product_1",
    "entity_type": "product",
    "interaction_type": "purchase",
    "metadata": {
        "source": "web",
        "device": "desktop",
    },
    "tenant_id": "tenant_a",
})
```

#### Get User Interactions

```python
interactions = await client.get_user_interactions(
    "user_123",
    limit=50,
    offset=0,
    tenant_id="tenant_a",
)

for interaction in interactions:
    print(f"{interaction['user_id']} -> {interaction['entity_id']}: {interaction['interaction_type']}")
```

#### Bulk Import Interactions

```python
result = await client.bulk_import_interactions({
    "interactions": [
        {
            "user_id": "user_1",
            "entity_id": "product_1",
            "entity_type": "product",
            "interaction_type": "view",
        },
        {
            "user_id": "user_1",
            "entity_id": "product_2",
            "entity_type": "product",
            "interaction_type": "purchase",
        },
    ],
    "tenant_id": "tenant_a",
})
```

### Recommendation Operations

#### Get User Recommendations

```python
# Hybrid recommendations (default)
recommendations = await client.get_user_recommendations(
    "user_123",
    {
        "algorithm": "hybrid",
        "count": 10,
        "tenant_id": "tenant_a",
    },
)

# Collaborative filtering
collab_recs = await client.get_user_recommendations(
    "user_123",
    {"algorithm": "collaborative", "count": 20},
)

# Content-based filtering
content_recs = await client.get_user_recommendations(
    "user_123",
    {"algorithm": "content_based", "count": 15},
)

for rec in recommendations["recommendations"]:
    print(f"{rec['entity_id']}: score={rec['score']}, reason={rec.get('reason')}")

if recommendations["cold_start"]:
    print("User has few interactions, showing trending items")
```

#### Get Similar Entities

```python
similar = await client.get_similar_entities(
    "product_1",
    {
        "algorithm": "content_based",
        "count": 10,
        "entity_type": "product",
        "tenant_id": "tenant_a",
    },
)

for item in similar["recommendations"]:
    print(f"Similar to product_1: {item['entity_id']} (score: {item['score']})")
```

#### Get Trending Entities

```python
trending = await client.get_trending_entities({
    "entity_type": "product",
    "count": 20,
    "tenant_id": "tenant_a",
})

for index, item in enumerate(trending["trending"], 1):
    print(f"#{index}: {item['entity_id']} (score: {item['score']})")
```

### Health Checks

```python
# Check if API is healthy
is_healthy = await client.is_healthy()
print(f"API healthy: {is_healthy}")

# Check if API is ready (db + redis connected)
is_ready = await client.is_ready()
print(f"API ready: {is_ready}")
```

## Error Handling

The client provides custom exceptions for better error handling:

```python
from recommendation_engine_client import (
    RecommendationClient,
    RecommendationError,
    TimeoutError,
    NetworkError,
)

async with RecommendationClient(base_url="http://localhost:8080") as client:
    try:
        entity = await client.get_entity("non_existent_id")
    except RecommendationError as error:
        print(f"API Error [{error.code}]: {error.message}")
        if error.details:
            print(f"Details: {error.details}")
    except TimeoutError as error:
        print(f"Request timeout: {error}")
    except NetworkError as error:
        print(f"Network error: {error}")
```

## Type Hints

This library is fully typed and provides comprehensive type definitions:

```python
from recommendation_engine_client import (
    Entity,
    Interaction,
    RecommendationResponse,
    ScoredEntity,
    InteractionType,
)

def process_recommendations(response: RecommendationResponse) -> None:
    """Process recommendations with full type safety."""
    for item in response["recommendations"]:
        # item is fully typed as ScoredEntity
        print(item["entity_id"], item["score"])
```

## Multi-Tenancy

The client supports multi-tenancy. You can specify a `tenant_id` in most operations:

```python
# Create entity for tenant A
await client.create_entity({
    "entity_id": "product_1",
    "entity_type": "product",
    "attributes": {"name": "Product"},
    "tenant_id": "tenant_a",
})

# Get recommendations for tenant A
recs = await client.get_user_recommendations(
    "user_123",
    {"tenant_id": "tenant_a"},
)
```

## Batch Operations

For importing large amounts of data, use the bulk import methods:

```python
# Prepare data
entities = [
    {
        "entity_id": f"product_{i}",
        "entity_type": "product",
        "attributes": {
            "name": f"Product {i}",
            "price": i * 10.0,
        },
    }
    for i in range(1000)
]

# Import in batches
batch_size = 100
for i in range(0, len(entities), batch_size):
    batch = entities[i:i + batch_size]
    result = await client.bulk_import_entities({
        "entities": batch,
        "tenant_id": "tenant_a",
    })
    print(f"Batch {i // batch_size + 1}: {result['successful']}/{result['total_records']} successful")
```

## API Reference

### RecommendationClient Methods

#### Entity Operations
- `create_entity(request: CreateEntityRequest) -> Entity`
- `get_entity(entity_id: str, tenant_id: str | None = None) -> Entity`
- `update_entity(entity_id: str, request: UpdateEntityRequest) -> Entity`
- `delete_entity(entity_id: str, tenant_id: str | None = None) -> None`
- `bulk_import_entities(request: BulkImportEntitiesRequest) -> BulkImportResponse`

#### Interaction Operations
- `create_interaction(request: CreateInteractionRequest) -> Interaction`
- `get_user_interactions(user_id: str, *, limit: int | None = None, offset: int | None = None, tenant_id: str | None = None) -> list[Interaction]`
- `bulk_import_interactions(request: BulkImportInteractionsRequest) -> BulkImportResponse`

#### Recommendation Operations
- `get_user_recommendations(user_id: str, query: UserRecommendationsQuery | None = None) -> RecommendationResponse`
- `get_similar_entities(entity_id: str, query: EntityRecommendationsQuery | None = None) -> RecommendationResponse`
- `get_trending_entities(query: TrendingEntitiesQuery | None = None) -> TrendingEntitiesResponse`

#### Health & Status
- `is_healthy() -> bool`
- `is_ready() -> bool`

## Examples

See the `examples/` directory for complete examples:

- `examples/basic_usage.py` - Basic client usage
- `examples/e_commerce.py` - E-commerce recommendation flow
- `examples/content_platform.py` - Content recommendation flow
- `examples/bulk_import.py` - Bulk data import

## Requirements

- Python >= 3.14
- httpx >= 0.28.1
- typing-extensions >= 4.12.2

## Development

### Setup

```bash
# Clone the repository
git clone https://github.com/grooveshop/recommendation-engine
cd recommendation-engine/clients/python

# Install dependencies with uv
uv sync --dev
```

### Running Tests

```bash
# Run all tests
uv run pytest

# Run with coverage
uv run pytest --cov

# Run with verbose output
uv run pytest -v
```

### Linting and Formatting

```bash
# Check code with ruff
uv run ruff check src/

# Fix issues automatically
uv run ruff check --fix src/

# Format code
uv run ruff format src/
```

### Type Checking

```bash
# Type check with mypy
uv run mypy src/
```

## Modern Stack

This client uses a modern Python stack:

- **Python 3.14**: Latest Python with modern type hints
- **httpx**: Modern async HTTP client
- **uv**: Fast Python package installer and resolver
- **ruff**: Lightning-fast linter and formatter
- **pytest**: Modern testing framework
- **mypy**: Static type checker

## License

MIT

## Support

For issues and questions:
- GitHub Issues: https://github.com/grooveshop/recommendation-engine/issues
- Documentation: https://docs.grooveshop.com/recommendation-engine

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.
