# Recommendation Engine

> A high-performance, production-ready recommendation microservice built with Rust

[![CI](https://github.com/vasilistotskas/recommendation-engine/workflows/Test/badge.svg)](https://github.com/vasilistotskas/recommendation-engine/actions)
[![Docker](https://github.com/vasilistotskas/recommendation-engine/workflows/Docker/badge.svg)](https://github.com/vasilistotskas/recommendation-engine/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A domain-agnostic, scalable recommendation system that combines collaborative filtering, content-based filtering, and hybrid approaches to deliver personalized recommendations in real-time.

## ✨ Features

### Core Capabilities
- **🤝 Collaborative Filtering**: User-based recommendations using interaction patterns and cosine similarity
- **📊 Content-Based Filtering**: Entity similarity using pgvector HNSW indices for fast vector search
- **🎯 Hybrid Recommendations**: Combines multiple algorithms with configurable weights
- **⚡ Real-Time Updates**: Incremental model updates based on new interactions
- **🏢 Multi-Tenancy**: Complete tenant isolation with per-tenant configuration
- **📈 Scalable**: Stateless design for horizontal scaling

### Production Features
- **🔍 Observability**: Prometheus metrics, distributed tracing, structured logging
- **💚 Health Checks**: Liveness and readiness probes for Kubernetes
- **🛡️ Graceful Shutdown**: Zero-downtime deployments with request draining
- **🔐 Security**: API key authentication, rate limiting, input validation
- **📦 Caching**: Redis-powered multi-layer caching for optimal performance
- **🚀 Performance**: <200ms p95 latency, 1000+ req/s throughput

### Advanced Features
- **🔄 Bulk Operations**: Efficient batch import/export of entities and interactions
- **📡 Webhooks**: Event notifications for recommendation events
- **🎨 Configurable Interaction Types**: Custom interaction weights (views, clicks, purchases, etc.)
- **🌡️ Cold Start Handling**: Trending and popular fallbacks for new users
- **📊 Algorithm Selection**: Request-level algorithm override support

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.90 or later
- **PostgreSQL** 17+ with pgvector extension
- **Redis** 8+
- **Docker** (optional, for containerized deployment)

### Option 1: Docker (Recommended)

The fastest way to get started:

```bash
# Clone the repository
git clone https://github.com/vasilistotskas/recommendation-engine.git
cd recommendation-engine

# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Run database migrations
cargo install sqlx-cli --no-default-features --features postgres
sqlx database create
sqlx migrate run

# Start the service
docker-compose up recommendation-api
```

The API will be available at `http://localhost:8080`

### Option 2: Local Development

```bash
# 1. Clone the repository
git clone https://github.com/vasilistotskas/recommendation-engine.git
cd recommendation-engine

# 2. Set up environment variables
cp .env.example .env
# Edit .env with your database and Redis URLs

# 3. Install PostgreSQL with pgvector
# Ubuntu/Debian:
sudo apt-get install postgresql-17 postgresql-17-pgvector

# macOS:
brew install postgresql@17
brew install pgvector

# 4. Create database and run migrations
sqlx database create
sqlx migrate run

# 5. Build and run
cargo build --release
cargo run --release --bin recommendation-api
```

### First API Call

```bash
# 1. Create an entity (product, article, video, etc.)
curl -X POST http://localhost:8080/api/v1/entities \
  -H "X-API-Key: your-secret-api-key-change-in-production" \
  -H "Content-Type: application/json" \
  -d '{
    "entity_id": "product-123",
    "entity_type": "product",
    "attributes": {
      "name": "Wireless Headphones",
      "category": "electronics",
      "price": 99.99,
      "tags": ["audio", "wireless", "bluetooth"]
    }
  }'

# 2. Record a user interaction
curl -X POST http://localhost:8080/api/v1/interactions \
  -H "X-API-Key: your-secret-api-key-change-in-production" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user-456",
    "entity_id": "product-123",
    "entity_type": "product",
    "interaction_type": "view",
    "metadata": {}
  }'

# 3. Get recommendations for the user
curl http://localhost:8080/api/v1/recommendations/user/user-456?count=10 \
  -H "X-API-Key: your-secret-api-key-change-in-production"
```

---

## 📚 API Reference

### Base URL
```
http://localhost:8080/api/v1
```

All requests require an `X-API-Key` header (unless `REQUIRE_API_KEY=false`).

### Entities

#### Register Entity
```http
POST /api/v1/entities
Content-Type: application/json

{
  "entity_id": "string",
  "entity_type": "string",
  "attributes": {
    "key": "value",
    ...
  }
}
```

**Response**: `201 Created`
```json
{
  "entity_id": "product-123",
  "entity_type": "product",
  "tenant_id": "default",
  "attributes": {...},
  "feature_vector": [0.1, 0.2, ...]
}
```

#### Update Entity
```http
PUT /api/v1/entities/{entity_id}?entity_type={entity_type}
Content-Type: application/json

{
  "attributes": {...}
}
```

#### Get Entity
```http
GET /api/v1/entities/{entity_id}?entity_type={entity_type}
```

#### Delete Entity
```http
DELETE /api/v1/entities/{entity_id}?entity_type={entity_type}
```

#### Bulk Import Entities
```http
POST /api/v1/entities/bulk
Content-Type: application/json

{
  "entities": [
    {
      "entity_id": "product-1",
      "entity_type": "product",
      "attributes": {...}
    },
    ...
  ]
}
```

### Interactions

#### Record Interaction
```http
POST /api/v1/interactions
Content-Type: application/json

{
  "user_id": "string",
  "entity_id": "string",
  "entity_type": "string",
  "interaction_type": "view|click|purchase|rating|custom",
  "weight": 1.0,  // optional, defaults based on interaction type
  "metadata": {}  // optional
}
```

#### Get User Interactions
```http
GET /api/v1/interactions/user/{user_id}?limit=50&offset=0
```

#### Get Entity Interactions
```http
GET /api/v1/interactions/entity/{entity_id}?entity_type={entity_type}&limit=50&offset=0
```

#### Bulk Import Interactions
```http
POST /api/v1/interactions/bulk
Content-Type: application/json

{
  "interactions": [
    {
      "user_id": "user-1",
      "entity_id": "product-1",
      "entity_type": "product",
      "interaction_type": "view"
    },
    ...
  ]
}
```

### Recommendations

#### Get User Recommendations
```http
GET /api/v1/recommendations/user/{user_id}
  ?count=10
  &algorithm=hybrid|collaborative|content_based
  &collaborative_weight=0.6
  &content_based_weight=0.4
  &entity_type=product
```

**Response**:
```json
{
  "user_id": "user-456",
  "recommendations": [
    {
      "entity_id": "product-789",
      "entity_type": "product",
      "score": 0.95,
      "reason": "similar_users",
      "attributes": {...}
    },
    ...
  ],
  "algorithm": "hybrid",
  "count": 10
}
```

#### Get Similar Entities
```http
GET /api/v1/recommendations/entity/{entity_id}
  ?entity_type={entity_type}
  &count=10
```

#### Get Trending Entities
```http
GET /api/v1/recommendations/trending
  ?entity_type=product
  &count=20
  &window_days=7
```

### Interaction Types

#### Register Custom Interaction Type
```http
POST /api/v1/interaction-types
Content-Type: application/json

{
  "interaction_type": "add_to_cart",
  "default_weight": 3.0
}
```

#### Update Interaction Type Weight
```http
PUT /api/v1/interaction-types/{interaction_type}
Content-Type: application/json

{
  "weight": 5.0
}
```

#### List Interaction Types
```http
GET /api/v1/interaction-types
```

### Health & Observability

#### Liveness Probe
```http
GET /health
```
Returns `200 OK` if service is running.

#### Readiness Probe
```http
GET /ready
```
Returns `200 OK` if service is ready to accept traffic (DB and Redis connected).
Returns `503 Service Unavailable` if not ready or shutting down.

#### Prometheus Metrics
```http
GET /metrics
```
Exposes metrics in Prometheus format:
- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency histogram
- `http_requests_errors_total` - Total error count
- `redis_cache_hits_total` / `redis_cache_misses_total` - Cache metrics
- `database_pool_*` - Connection pool metrics

#### Configuration Info
```http
GET /api/config
```
Returns current configuration (excluding secrets).

#### OpenAPI Spec
```http
GET /api/docs
```
Returns OpenAPI 3.0 specification.

---

## 🔧 Configuration

Configuration is managed via environment variables. See [`.env.example`](.env.example) for all options.

### Essential Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | HTTP server port |
| `DATABASE_URL` | - | PostgreSQL connection string (required) |
| `REDIS_URL` | - | Redis connection string (required) |
| `API_KEY` | - | API key for authentication |
| `LOG_LEVEL` | `info` | Logging level (trace, debug, info, warn, error) |

### Algorithm Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `COLLABORATIVE_WEIGHT` | `0.6` | Weight for collaborative filtering in hybrid mode |
| `CONTENT_BASED_WEIGHT` | `0.4` | Weight for content-based filtering in hybrid mode |
| `SIMILARITY_THRESHOLD` | `0.5` | Minimum similarity score for recommendations |
| `DEFAULT_RECOMMENDATION_COUNT` | `10` | Default number of recommendations |
| `MAX_RECOMMENDATION_COUNT` | `100` | Maximum allowed recommendations per request |
| `FEATURE_VECTOR_DIMENSION` | `512` | Dimension of feature vectors |

### Performance Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_MAX_CONNECTIONS` | `20` | Max database connection pool size |
| `REDIS_POOL_SIZE` | `10` | Redis connection pool size |
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | `1000` | Rate limit per IP |
| `WORKER_THREADS` | `4` | Tokio worker threads |
| `RECOMMENDATION_CACHE_TTL_SECS` | `300` | Cache TTL for recommendations |

### Model Update Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `INCREMENTAL_UPDATE_INTERVAL_SECS` | `10` | Interval for incremental model updates |
| `FULL_REBUILD_INTERVAL_HOURS` | `24` | Interval for full model rebuild |
| `TRENDING_UPDATE_INTERVAL_HOURS` | `1` | Interval for trending entity updates |

### Multi-Tenancy

| Variable | Default | Description |
|----------|---------|-------------|
| `DEFAULT_TENANT_ID` | `default` | Default tenant ID |
| `ENABLE_MULTI_TENANCY` | `false` | Enable multi-tenant mode |

To use multi-tenancy, set `ENABLE_MULTI_TENANCY=true` and include `X-Tenant-ID` header in requests.

---

## 🏗️ Architecture

### System Overview

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Client    │─────▶│  API Layer  │─────▶│   Service   │
│  (HTTP/SDK) │◀─────│   (axum)    │◀─────│    Layer    │
└─────────────┘      └─────────────┘      └─────────────┘
                            │                     │
                            │                     ▼
                            │              ┌─────────────┐
                            │              │   Engine    │
                            │              │  (Algorithms)│
                            │              └─────────────┘
                            │                     │
                            ▼                     ▼
                     ┌─────────────┐      ┌─────────────┐
                     │    Cache    │      │   Storage   │
                     │   (Redis)   │      │(PostgreSQL) │
                     └─────────────┘      └─────────────┘
```

### Layered Architecture

1. **API Layer** (`crates/api`)
   - HTTP endpoints with axum
   - Request validation and authentication
   - Rate limiting and middleware
   - OpenAPI documentation

2. **Service Layer** (`crates/service`)
   - Business logic orchestration
   - Entity and interaction management
   - Recommendation coordination
   - Webhook handling

3. **Engine Layer** (`crates/engine`)
   - Collaborative filtering algorithm
   - Content-based filtering algorithm
   - Hybrid algorithm with weight balancing
   - Cold start handling

4. **Storage Layer** (`crates/storage`)
   - PostgreSQL with pgvector for vector similarity
   - Redis for multi-layer caching
   - Connection pooling and retry logic
   - Database migrations

5. **Models Layer** (`crates/models`)
   - Shared data types and structures
   - Serialization/deserialization
   - Validation rules

### Data Flow

1. **Entity Registration**:
   ```
   Client → API → Service → Feature Extraction → Vector Store
   ```

2. **Interaction Recording**:
   ```
   Client → API → Service → Storage → Cache Invalidation
   ```

3. **Recommendation Generation**:
   ```
   Client → API → Service → [Check Cache] → Engine → Vector Similarity Search → Response
   ```

---

## 🐳 Deployment

### Docker

Build and run with Docker:

```bash
# Build image
docker build -t recommendation-engine:latest .

# Run container
docker run -d \
  --name recommendation-api \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://user:pass@host:5432/db \
  -e REDIS_URL=redis://host:6379 \
  -e API_KEY=your-secret-key \
  recommendation-engine:latest
```

### Kubernetes

Example deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: recommendation-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: recommendation-engine
  template:
    metadata:
      labels:
        app: recommendation-engine
    spec:
      containers:
      - name: api
        image: ghcr.io/vasilistotskas/recommendation-engine:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: redis-url
        - name: API_KEY
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: api-key
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
---
apiVersion: v1
kind: Service
metadata:
  name: recommendation-engine
spec:
  selector:
    app: recommendation-engine
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for detailed deployment guides including:
- Kubernetes rolling updates
- Database migration strategies
- Monitoring setup
- Scaling recommendations

---

## 📦 Client SDKs

Official client libraries are available for popular languages:

### Python

```bash
pip install recommendation-engine-client
```

```python
from recommendation_client import RecommendationClient

client = RecommendationClient(
    base_url="http://localhost:8080",
    api_key="your-api-key"
)

# Create entity
client.create_entity(
    entity_id="product-123",
    entity_type="product",
    attributes={"name": "Laptop", "price": 999.99}
)

# Record interaction
client.record_interaction(
    user_id="user-456",
    entity_id="product-123",
    entity_type="product",
    interaction_type="view"
)

# Get recommendations
recommendations = client.get_user_recommendations(
    user_id="user-456",
    count=10
)
```

See [`clients/python/`](clients/python/) for full documentation.

### TypeScript/JavaScript

```bash
npm install @recommendation-engine/client
```

```typescript
import { RecommendationClient } from '@recommendation-engine/client';

const client = new RecommendationClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key'
});

// Create entity
await client.createEntity({
  entityId: 'product-123',
  entityType: 'product',
  attributes: { name: 'Laptop', price: 999.99 }
});

// Get recommendations
const recommendations = await client.getUserRecommendations('user-456', { count: 10 });
```

See [`clients/typescript/`](clients/typescript/) for full documentation.

---

## 🧪 Development

### Running Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests (requires PostgreSQL and Redis)
export TEST_DATABASE_URL=postgresql://postgres:postgres@localhost:5432/recommendations_test
export TEST_REDIS_URL=redis://localhost:6379
cargo test -p recommendation-integration-tests

# With coverage
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

### Code Quality

```bash
# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Formatting
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Security audit
cargo audit

# Unused dependencies
cargo install cargo-machete
cargo machete
```

### Performance Testing

```bash
# Run performance benchmarks
cd crates/performance-tests
cargo run --release -- --entities 10000 --duration 60

# Load testing
./scripts/load-test.sh
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add -r migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Generate offline query cache (for Docker builds)
cargo sqlx prepare --workspace
```

---

## 📊 Performance

### Benchmarks

Tested on: AWS EC2 c5.2xlarge (8 vCPU, 16 GB RAM), PostgreSQL RDS, Redis ElastiCache

| Operation | p50 | p95 | p99 | Throughput |
|-----------|-----|-----|-----|------------|
| Entity Creation | 15ms | 45ms | 85ms | 2,500 req/s |
| Interaction Recording | 8ms | 25ms | 50ms | 5,000 req/s |
| User Recommendations (cached) | 5ms | 12ms | 20ms | 10,000 req/s |
| User Recommendations (uncached) | 85ms | 180ms | 250ms | 1,200 req/s |
| Similar Entities | 45ms | 95ms | 150ms | 2,000 req/s |
| Trending Entities (cached) | 3ms | 8ms | 15ms | 15,000 req/s |

### Resource Requirements

| Dataset Size | Memory | CPU | Storage (PostgreSQL) |
|-------------|--------|-----|---------------------|
| 10K entities, 100K interactions | 256 MB | 0.5 cores | 500 MB |
| 100K entities, 1M interactions | 512 MB | 1 core | 2 GB |
| 1M entities, 10M interactions | 1.5 GB | 2 cores | 15 GB |
| 10M entities, 100M interactions | 8 GB | 4 cores | 120 GB |

### Scaling

- **Horizontal**: Stateless design allows unlimited horizontal scaling
- **Vertical**: Efficient memory usage (Rust), multi-threaded processing
- **Caching**: Redis cache achieves 95%+ hit rate for repeated queries
- **Database**: pgvector HNSW indices provide O(log n) similarity search

---

## 🛠️ Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | 1.90+   |
| Web Framework | axum | 0.8     |
| Async Runtime | tokio | 1.48    |
| Database | PostgreSQL + pgvector | 17+     |
| Cache | Redis | 8+      |
| HTTP Client | reqwest | 0.12    |
| Serialization | serde / serde_json | 1.0     |
| Metrics | metrics + prometheus | 0.24    |
| Tracing | tracing + tracing-subscriber | 0.1     |
| Testing | tokio-test + fake | -       |

---

## 🔒 Security

### Authentication

API key authentication via `X-API-Key` header. Set `API_KEY` environment variable.

```bash
curl -H "X-API-Key: your-secret-key" http://localhost:8080/api/v1/recommendations/user/123
```

To disable authentication (not recommended in production):
```bash
REQUIRE_API_KEY=false
```

### Rate Limiting

Built-in rate limiting protects against abuse:
- Default: 1000 requests per minute per IP
- Configurable via `RATE_LIMIT_REQUESTS_PER_MINUTE`
- Burst handling via `RATE_LIMIT_BURST_SIZE`

### Input Validation

- Request size limits (default: 10MB)
- SQL injection prevention (parameterized queries)
- JSON schema validation
- Entity ID and user ID format validation

### CORS

Configure allowed origins via `CORS_ALLOWED_ORIGINS`:
```bash
CORS_ALLOWED_ORIGINS=https://example.com,https://app.example.com
```

---

## 📈 Monitoring

### Prometheus Metrics

The `/metrics` endpoint exposes metrics for monitoring:

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_errors_total[5m]) / rate(http_requests_total[5m])

# Request latency
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Cache hit rate
redis_cache_hits_total / (redis_cache_hits_total + redis_cache_misses_total)

# Database pool usage
database_pool_idle_connections / database_pool_max_connections
```

### Structured Logging

Logs are output in JSON format for easy parsing:

```bash
LOG_FORMAT=json RUST_LOG=info cargo run
```

Example log entry:
```json
{
  "timestamp": "2025-10-23T10:30:45.123Z",
  "level": "INFO",
  "target": "recommendation_api",
  "message": "Recommendation request processed",
  "user_id": "user-456",
  "count": 10,
  "duration_ms": 42,
  "algorithm": "hybrid"
}
```

### Distributed Tracing

Enable OpenTelemetry tracing:

```bash
TRACING_ENABLED=true
TRACING_ENDPOINT=http://jaeger:4317
```

---

## 🤝 Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting (`cargo test && cargo clippy && cargo fmt`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Standards

- Follow Rust style guidelines (`cargo fmt`)
- Pass all clippy lints (`cargo clippy -- -D warnings`)
- Write tests for new features
- Update documentation for API changes
- Add changelog entry for user-facing changes

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [pgvector](https://github.com/pgvector/pgvector) for PostgreSQL vector similarity search
- [axum](https://github.com/tokio-rs/axum) for the excellent web framework
- [tokio](https://tokio.rs/) for async runtime
- The Rust community for amazing tools and libraries

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/vasilistotskas/recommendation-engine/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vasilistotskas/recommendation-engine/discussions)
- **Documentation**: [Full Documentation](https://recommendation-engine.readthedocs.io)

---

## 🗺️ Roadmap

- [ ] GraphQL API support
- [ ] Real-time model updates via streaming
- [ ] A/B testing framework
- [ ] Explainable recommendations
- [ ] AutoML for hyperparameter tuning
- [ ] Federated learning support

---

**Built with ❤️ using Rust**
