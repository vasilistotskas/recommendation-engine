# Recommendation Engine TypeScript/JavaScript Client

Official TypeScript/JavaScript client library for the GrooveShop Recommendation Engine API.

## Features

- ✅ Full TypeScript support with comprehensive type definitions
- ✅ Modern ESM-only package (Node.js 22+)
- ✅ Uses native `fetch` API (no external HTTP dependencies)
- ✅ Works in both Node.js and browser environments
- ✅ Promise-based async API
- ✅ Automatic error handling with timeout support
- ✅ Support for all API endpoints
- ✅ Authentication support (API key)
- ✅ Multi-tenancy support
- ✅ Batch operations support
- ✅ Built with Vite for optimal bundling
- ✅ Tested with Vitest

## Installation

```bash
npm install @grooveshop/recommendation-engine-client
```

Or with Yarn:

```bash
yarn add @grooveshop/recommendation-engine-client
```

## Quick Start

```typescript
import { RecommendationClient } from '@grooveshop/recommendation-engine-client';

// Initialize the client
const client = new RecommendationClient({
  baseUrl: 'http://localhost:8080',
  apiKey: 'your-api-key', // Optional
  timeout: 30000, // Optional, default: 30000ms
});

// Get recommendations for a user
const recommendations = await client.getUserRecommendations('user_123', {
  algorithm: 'hybrid',
  count: 10,
});

console.log(recommendations.recommendations);
```

## Usage

### Client Initialization

```typescript
import { RecommendationClient } from '@grooveshop/recommendation-engine-client';

const client = new RecommendationClient({
  baseUrl: 'https://api.example.com',
  apiKey: 'your-secret-api-key',
  timeout: 30000,
  headers: {
    // Optional custom headers
    'X-Custom-Header': 'value',
  },
});
```

### Entity Operations

#### Create an Entity

```typescript
const entity = await client.createEntity({
  entity_id: 'product_1',
  entity_type: 'product',
  attributes: {
    name: 'Wireless Headphones',
    category: 'Electronics',
    price: 99.99,
    brand: 'TechPro',
    in_stock: true,
    tags: ['wireless', 'audio', 'bluetooth'],
  },
  tenant_id: 'tenant_a', // Optional
});
```

#### Get an Entity

```typescript
const entity = await client.getEntity('product_1', 'tenant_a');
console.log(entity.attributes);
```

#### Update an Entity

```typescript
const updatedEntity = await client.updateEntity('product_1', {
  attributes: {
    price: 89.99,
    in_stock: false,
  },
  tenant_id: 'tenant_a',
});
```

#### Delete an Entity

```typescript
await client.deleteEntity('product_1', 'tenant_a');
```

#### Bulk Import Entities

```typescript
const result = await client.bulkImportEntities({
  entities: [
    {
      entity_id: 'product_1',
      entity_type: 'product',
      attributes: {
        name: 'Product 1',
        price: 29.99,
      },
    },
    {
      entity_id: 'product_2',
      entity_type: 'product',
      attributes: {
        name: 'Product 2',
        price: 39.99,
      },
    },
  ],
  tenant_id: 'tenant_a',
});

console.log(`Imported ${result.successful}/${result.total_records} entities`);
```

### Interaction Operations

#### Create an Interaction

```typescript
const interaction = await client.createInteraction({
  user_id: 'user_123',
  entity_id: 'product_1',
  entity_type: 'product',
  interaction_type: 'purchase',
  metadata: {
    source: 'web',
    device: 'desktop',
  },
  tenant_id: 'tenant_a',
});
```

#### Get User Interactions

```typescript
const interactions = await client.getUserInteractions('user_123', {
  limit: 50,
  offset: 0,
  tenant_id: 'tenant_a',
});

interactions.forEach((interaction) => {
  console.log(`${interaction.user_id} -> ${interaction.entity_id}: ${interaction.interaction_type}`);
});
```

#### Bulk Import Interactions

```typescript
const result = await client.bulkImportInteractions({
  interactions: [
    {
      user_id: 'user_1',
      entity_id: 'product_1',
      entity_type: 'product',
      interaction_type: 'view',
    },
    {
      user_id: 'user_1',
      entity_id: 'product_2',
      entity_type: 'product',
      interaction_type: 'purchase',
    },
  ],
  tenant_id: 'tenant_a',
});
```

### Recommendation Operations

#### Get User Recommendations

```typescript
// Hybrid recommendations (default)
const recommendations = await client.getUserRecommendations('user_123', {
  algorithm: 'hybrid',
  count: 10,
  tenant_id: 'tenant_a',
});

// Collaborative filtering
const collabRecs = await client.getUserRecommendations('user_123', {
  algorithm: 'collaborative',
  count: 20,
});

// Content-based filtering
const contentRecs = await client.getUserRecommendations('user_123', {
  algorithm: 'content_based',
  count: 15,
});

recommendations.recommendations.forEach((rec) => {
  console.log(`${rec.entity_id}: score=${rec.score}, reason=${rec.reason}`);
});

if (recommendations.cold_start) {
  console.log('User has few interactions, showing trending items');
}
```

#### Get Similar Entities

```typescript
const similar = await client.getSimilarEntities('product_1', {
  algorithm: 'content_based',
  count: 10,
  entity_type: 'product',
  tenant_id: 'tenant_a',
});

similar.recommendations.forEach((item) => {
  console.log(`Similar to product_1: ${item.entity_id} (score: ${item.score})`);
});
```

#### Get Trending Entities

```typescript
const trending = await client.getTrendingEntities({
  entity_type: 'product',
  count: 20,
  tenant_id: 'tenant_a',
});

trending.trending.forEach((item, index) => {
  console.log(`#${index + 1}: ${item.entity_id} (score: ${item.score})`);
});
```

### Health Checks

```typescript
// Check if API is healthy
const isHealthy = await client.isHealthy();
console.log('API healthy:', isHealthy);

// Check if API is ready (db + redis connected)
const isReady = await client.isReady();
console.log('API ready:', isReady);
```

## Error Handling

The client automatically wraps API errors in a `RecommendationError` class:

```typescript
import { RecommendationClient, RecommendationError } from '@grooveshop/recommendation-engine-client';

try {
  const entity = await client.getEntity('non_existent_id');
} catch (error) {
  if (error instanceof RecommendationError) {
    console.error(`API Error ${error.code}: ${error.message}`);
    console.error('Details:', error.details);
  } else {
    console.error('Network or other error:', error);
  }
}
```

## TypeScript Support

This library is written in TypeScript and provides full type definitions. All API responses and requests are fully typed:

```typescript
import type {
  Entity,
  Interaction,
  RecommendationResponse,
  ScoredEntity,
  InteractionType,
} from '@grooveshop/recommendation-engine-client';

const processRecommendations = (response: RecommendationResponse): void => {
  response.recommendations.forEach((item: ScoredEntity) => {
    // item is fully typed
    console.log(item.entity_id, item.score);
  });
};
```

## Multi-Tenancy

The client supports multi-tenancy. You can specify a `tenant_id` in most operations:

```typescript
// Create entity for tenant A
await client.createEntity({
  entity_id: 'product_1',
  entity_type: 'product',
  attributes: { name: 'Product' },
  tenant_id: 'tenant_a',
});

// Get recommendations for tenant A
const recs = await client.getUserRecommendations('user_123', {
  tenant_id: 'tenant_a',
});
```

## Batch Operations

For importing large amounts of data, use the bulk import methods:

```typescript
// Prepare data
const entities = Array.from({ length: 1000 }, (_, i) => ({
  entity_id: `product_${i}`,
  entity_type: 'product',
  attributes: {
    name: `Product ${i}`,
    price: Math.random() * 100,
  },
}));

// Import in batches
const batchSize = 100;
for (let i = 0; i < entities.length; i += batchSize) {
  const batch = entities.slice(i, i + batchSize);
  const result = await client.bulkImportEntities({
    entities: batch,
    tenant_id: 'tenant_a',
  });
  console.log(`Batch ${i / batchSize + 1}: ${result.successful}/${result.total_records} successful`);
}
```

## Browser Usage

The client works in browser environments as well:

```html
<script type="module">
  import { RecommendationClient } from '@grooveshop/recommendation-engine-client';

  const client = new RecommendationClient({
    baseUrl: 'https://api.example.com',
    apiKey: 'your-api-key',
  });

  const recommendations = await client.getUserRecommendations('current-user', {
    count: 5,
  });

  // Display recommendations in UI
  recommendations.recommendations.forEach((rec) => {
    console.log(rec.entity_id, rec.score);
  });
</script>
```

## API Reference

### RecommendationClient Methods

#### Entity Operations
- `createEntity(request: CreateEntityRequest): Promise<Entity>`
- `getEntity(entityId: string, tenantId?: string): Promise<Entity>`
- `updateEntity(entityId: string, request: UpdateEntityRequest): Promise<Entity>`
- `deleteEntity(entityId: string, tenantId?: string): Promise<void>`
- `bulkImportEntities(request: BulkImportEntitiesRequest): Promise<BulkImportResponse>`

#### Interaction Operations
- `createInteraction(request: CreateInteractionRequest): Promise<Interaction>`
- `getUserInteractions(userId: string, options?: QueryOptions): Promise<Interaction[]>`
- `bulkImportInteractions(request: BulkImportInteractionsRequest): Promise<BulkImportResponse>`

#### Recommendation Operations
- `getUserRecommendations(userId: string, query?: UserRecommendationsQuery): Promise<RecommendationResponse>`
- `getSimilarEntities(entityId: string, query?: EntityRecommendationsQuery): Promise<RecommendationResponse>`
- `getTrendingEntities(query?: TrendingEntitiesQuery): Promise<TrendingEntitiesResponse>`

#### Health & Status
- `isHealthy(): Promise<boolean>`
- `isReady(): Promise<boolean>`

## Examples

See the `examples/` directory for complete examples:

- `examples/basic-usage.ts` - Basic client usage
- `examples/e-commerce.ts` - E-commerce recommendation flow
- `examples/content-platform.ts` - Content recommendation flow
- `examples/bulk-import.ts` - Bulk data import

## Requirements

- Node.js >= 22.0.0 (for native fetch and modern ES features)
- TypeScript >= 5.4.0 (if using TypeScript)

## Development

```bash
# Install dependencies
npm install

# Build for production
npm run build

# Development mode (watch)
npm run dev

# Run tests
npm test

# Run tests with UI
npm run test:ui

# Run tests with coverage
npm run test:coverage

# Type checking
npm run typecheck

# Lint
npm run lint

# Format
npm run format
```

## Modern Stack

This client uses a modern JavaScript/TypeScript stack:

- **Vite**: Fast build tool with optimal bundling
- **Vitest**: Lightning-fast unit testing
- **Native Fetch**: No external HTTP client dependencies
- **ESM**: Pure ES modules for better tree-shaking
- **TypeScript 5.4+**: Latest TypeScript features

## License

MIT

## Support

For issues and questions:
- GitHub Issues: https://github.com/grooveshop/recommendation-engine/issues
- Documentation: https://docs.grooveshop.com/recommendation-engine

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.
