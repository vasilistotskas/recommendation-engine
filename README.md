# Recommendation Engine

A high-performance, domain-agnostic recommendation microservice written in Rust.

## Features

- **Collaborative Filtering**: User-based recommendations using interaction patterns
- **Content-Based Filtering**: Entity similarity using feature vectors
- **Hybrid Recommendations**: Combines multiple algorithms with configurable weights
- **Real-Time Updates**: Continuous model updates based on new interactions
- **Multi-Tenancy**: Support for multiple isolated tenants
- **Scalable**: Horizontal scaling with stateless design
- **Production Ready**: Comprehensive observability, health checks, and graceful shutdown

## Quick Start

### Prerequisites

- Rust 1.90 or later
- PostgreSQL 17+ with pgvector extension
- Redis 8+

### Installation

1. Clone the repository:
```bash
git clone https://github.com/grooveshop/recommendation-engine.git
cd recommendation-engine
```

2. Copy the environment file:
```bash
cp .env.example .env
```

3. Update the `.env` file with your database and Redis URLs.

4. Build the project:
```bash
cargo build --release
```

5. Run the service:
```bash
cargo run --release --bin recommendation-api
```

The service will start on `http://localhost:8080`.

## Project Structure

```
recommendation-engine/
├── crates/
│   ├── api/           # HTTP API layer (axum)
│   ├── service/       # Business logic layer
│   ├── engine/        # Recommendation algorithms
│   ├── storage/       # Database and cache layer
│   └── models/        # Data models and types
├── Cargo.toml         # Workspace configuration
├── .env.example       # Environment variables template
└── README.md          # This file
```

## Configuration

All configuration is done via environment variables. See `.env.example` for available options.

Key configuration options:
- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `PORT`: HTTP server port (default: 8080)
- `LOG_LEVEL`: Logging level (default: info)

## API Endpoints

### Health Checks
- `GET /health` - Liveness probe
- `GET /ready` - Readiness probe

### Entities
- `POST /api/v1/entities` - Register entity
- `PUT /api/v1/entities/{id}` - Update entity
- `DELETE /api/v1/entities/{id}` - Delete entity
- `GET /api/v1/entities/{id}` - Get entity

### Interactions
- `POST /api/v1/interactions` - Record interaction
- `GET /api/v1/interactions/user/{id}` - Get user interactions

### Recommendations
- `GET /api/v1/recommendations/user/{id}` - Get user recommendations
- `GET /api/v1/recommendations/entity/{id}` - Get similar entities
- `GET /api/v1/recommendations/trending` - Get trending entities

## Development

### Running Tests
```bash
cargo test
```

### Running with Hot Reload
```bash
cargo watch -x run
```

### Linting
```bash
cargo clippy -- -D warnings
```

### Formatting
```bash
cargo fmt
```

## Architecture

The system follows a layered architecture:

1. **API Layer**: HTTP endpoints, request validation, authentication
2. **Service Layer**: Business logic, algorithm orchestration
3. **Engine Layer**: Recommendation algorithms (collaborative, content-based, hybrid)
4. **Storage Layer**: PostgreSQL (persistent), Redis (cache)

## Technology Stack

- **Web Framework**: axum 0.8
- **Async Runtime**: tokio 1.41
- **Database**: sqlx 0.8 with PostgreSQL + pgvector
- **Cache**: redis 0.32
- **Observability**: tracing, metrics, prometheus
- **Serialization**: serde, serde_json

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
