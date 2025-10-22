import { describe, it, expect } from 'vitest';
import type {
  AttributeValue,
  Attributes,
  Entity,
  Interaction,
  InteractionType,
  RecommendationResponse,
  ScoredEntity,
  TrendingEntitiesResponse,
  CreateEntityRequest,
  UpdateEntityRequest,
  CreateInteractionRequest,
  BulkImportEntitiesRequest,
  BulkImportInteractionsRequest,
  BulkImportResponse,
  UserRecommendationsQuery,
  EntityRecommendationsQuery,
  TrendingEntitiesQuery,
  ErrorResponse,
  RecommendationClientConfig,
} from './types.js';

describe('Type Definitions', () => {
  it('should allow valid AttributeValue types', () => {
    const stringValue: AttributeValue = 'test';
    const numberValue: AttributeValue = 42;
    const booleanValue: AttributeValue = true;
    const stringArrayValue: AttributeValue = ['tag1', 'tag2'];

    expect(typeof stringValue).toBe('string');
    expect(typeof numberValue).toBe('number');
    expect(typeof booleanValue).toBe('boolean');
    expect(Array.isArray(stringArrayValue)).toBe(true);
  });

  it('should create valid Attributes object', () => {
    const attrs: Attributes = {
      name: 'Product',
      price: 99.99,
      in_stock: true,
      tags: ['electronics', 'wireless'],
    };

    expect(attrs.name).toBe('Product');
    expect(attrs.price).toBe(99.99);
    expect(attrs.in_stock).toBe(true);
    expect(attrs.tags).toEqual(['electronics', 'wireless']);
  });

  it('should create valid Entity', () => {
    const entity: Entity = {
      entity_id: 'product_1',
      entity_type: 'product',
      attributes: {
        name: 'Test Product',
        price: 99.99,
      },
      tenant_id: 'tenant_a',
      created_at: '2024-01-01T00:00:00Z',
      updated_at: '2024-01-01T00:00:00Z',
    };

    expect(entity.entity_id).toBe('product_1');
    expect(entity.entity_type).toBe('product');
    expect(entity.tenant_id).toBe('tenant_a');
  });

  it('should create valid Interaction', () => {
    const interaction: Interaction = {
      user_id: 'user_123',
      entity_id: 'product_1',
      entity_type: 'product',
      interaction_type: 'view',
      tenant_id: 'tenant_a',
      timestamp: '2024-01-01T00:00:00Z',
    };

    expect(interaction.user_id).toBe('user_123');
    expect(interaction.interaction_type).toBe('view');
  });

  it('should validate InteractionType', () => {
    const types: InteractionType[] = [
      'view',
      'click',
      'like',
      'dislike',
      'purchase',
      'add_to_cart',
      'remove_from_cart',
      'rate',
      'review',
      'share',
      'bookmark',
    ];

    expect(types.length).toBeGreaterThan(0);
    expect(types).toContain('view');
    expect(types).toContain('purchase');
  });

  it('should create valid ScoredEntity', () => {
    const scoredEntity: ScoredEntity = {
      entity_id: 'product_1',
      score: 0.95,
      reason: 'collaborative',
    };

    expect(scoredEntity.score).toBeGreaterThanOrEqual(0);
    expect(scoredEntity.score).toBeLessThanOrEqual(1);
  });

  it('should create valid RecommendationResponse', () => {
    const response: RecommendationResponse = {
      recommendations: [
        { entity_id: 'product_1', score: 0.95, reason: 'collaborative' },
        { entity_id: 'product_2', score: 0.85, reason: 'content_based' },
      ],
      algorithm: 'hybrid',
      count: 2,
      cold_start: false,
    };

    expect(response.recommendations.length).toBe(2);
    expect(response.algorithm).toBe('hybrid');
  });

  it('should create valid TrendingEntitiesResponse', () => {
    const response: TrendingEntitiesResponse = {
      trending: [
        { entity_id: 'product_1', score: 0.95, reason: 'trending' },
        { entity_id: 'product_2', score: 0.85, reason: 'trending' },
      ],
      count: 2,
    };

    expect(response.trending.length).toBe(2);
    expect(response.count).toBe(2);
  });

  it('should create valid CreateEntityRequest', () => {
    const request: CreateEntityRequest = {
      entity_id: 'product_1',
      entity_type: 'product',
      attributes: {
        name: 'Test Product',
        price: 99.99,
      },
      tenant_id: 'tenant_a',
    };

    expect(request.entity_id).toBe('product_1');
    expect(request.tenant_id).toBe('tenant_a');
  });

  it('should create valid UpdateEntityRequest', () => {
    const request: UpdateEntityRequest = {
      attributes: {
        price: 79.99,
      },
      tenant_id: 'tenant_a',
    };

    expect(request.attributes.price).toBe(79.99);
  });

  it('should create valid CreateInteractionRequest', () => {
    const request: CreateInteractionRequest = {
      user_id: 'user_123',
      entity_id: 'product_1',
      entity_type: 'product',
      interaction_type: 'view',
      metadata: {
        source: 'web',
        device: 'desktop',
      },
      tenant_id: 'tenant_a',
    };

    expect(request.user_id).toBe('user_123');
    expect(request.metadata).toBeDefined();
  });

  it('should create valid BulkImportEntitiesRequest', () => {
    const request: BulkImportEntitiesRequest = {
      entities: [
        {
          entity_id: 'product_1',
          entity_type: 'product',
          attributes: { name: 'Product 1' },
        },
        {
          entity_id: 'product_2',
          entity_type: 'product',
          attributes: { name: 'Product 2' },
        },
      ],
      tenant_id: 'tenant_a',
    };

    expect(request.entities.length).toBe(2);
  });

  it('should create valid BulkImportInteractionsRequest', () => {
    const request: BulkImportInteractionsRequest = {
      interactions: [
        {
          user_id: 'user_1',
          entity_id: 'product_1',
          entity_type: 'product',
          interaction_type: 'view',
        },
      ],
      tenant_id: 'tenant_a',
    };

    expect(request.interactions.length).toBe(1);
  });

  it('should create valid BulkImportResponse', () => {
    const response: BulkImportResponse = {
      total_records: 10,
      successful: 9,
      failed: 1,
      errors: [
        {
          record_index: 5,
          error: {
            code: 400,
            message: 'Invalid entity',
          },
        },
      ],
    };

    expect(response.total_records).toBe(10);
    expect(response.successful).toBe(9);
    expect(response.failed).toBe(1);
    expect(response.errors?.length).toBe(1);
  });

  it('should create valid UserRecommendationsQuery', () => {
    const query: UserRecommendationsQuery = {
      algorithm: 'hybrid',
      count: 10,
      tenant_id: 'tenant_a',
    };

    expect(query.algorithm).toBe('hybrid');
    expect(query.count).toBe(10);
  });

  it('should create valid EntityRecommendationsQuery', () => {
    const query: EntityRecommendationsQuery = {
      algorithm: 'content_based',
      count: 5,
      entity_type: 'product',
      tenant_id: 'tenant_a',
    };

    expect(query.algorithm).toBe('content_based');
    expect(query.entity_type).toBe('product');
  });

  it('should create valid TrendingEntitiesQuery', () => {
    const query: TrendingEntitiesQuery = {
      entity_type: 'product',
      count: 20,
      tenant_id: 'tenant_a',
    };

    expect(query.count).toBe(20);
  });

  it('should create valid ErrorResponse', () => {
    const error: ErrorResponse = {
      error: {
        code: 404,
        message: 'Not found',
        details: { entity_id: 'product_1' },
      },
    };

    expect(error.error.code).toBe(404);
    expect(error.error.message).toBe('Not found');
  });

  it('should create valid RecommendationClientConfig', () => {
    const config: RecommendationClientConfig = {
      baseUrl: 'http://localhost:8080',
      apiKey: 'test-key',
      timeout: 30000,
      headers: {
        'X-Custom-Header': 'value',
      },
    };

    expect(config.baseUrl).toBe('http://localhost:8080');
    expect(config.apiKey).toBe('test-key');
    expect(config.timeout).toBe(30000);
  });

  it('should create minimal RecommendationClientConfig', () => {
    const config: RecommendationClientConfig = {
      baseUrl: 'http://localhost:8080',
    };

    expect(config.baseUrl).toBeDefined();
    expect(config.apiKey).toBeUndefined();
  });
});
