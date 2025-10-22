import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { RecommendationClient, RecommendationError } from './client.js';
import type {
  Entity,
  Interaction,
  RecommendationResponse,
  TrendingEntitiesResponse,
  BulkImportResponse,
} from './types.js';

// Mock fetch globally
global.fetch = vi.fn();

describe('RecommendationClient', () => {
  let client: RecommendationClient;
  const mockBaseUrl = 'http://localhost:8080';
  const mockApiKey = 'test-api-key';

  beforeEach(() => {
    client = new RecommendationClient({
      baseUrl: mockBaseUrl,
      apiKey: mockApiKey,
      timeout: 5000,
    });
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe('Constructor', () => {
    it('should initialize with correct config', () => {
      expect(client).toBeDefined();
    });

    it('should strip trailing slash from baseUrl', () => {
      const clientWithSlash = new RecommendationClient({
        baseUrl: 'http://localhost:8080/',
      });
      expect(clientWithSlash).toBeDefined();
    });

    it('should set default timeout', () => {
      const clientNoTimeout = new RecommendationClient({
        baseUrl: mockBaseUrl,
      });
      expect(clientNoTimeout).toBeDefined();
    });

    it('should add Authorization header when apiKey is provided', () => {
      expect(client).toBeDefined();
    });
  });

  describe('Entity Operations', () => {
    const mockEntity: Entity = {
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

    it('should create an entity', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockEntity,
      });

      const result = await client.createEntity({
        entity_id: 'product_1',
        entity_type: 'product',
        attributes: { name: 'Test Product', price: 99.99 },
      });

      expect(result).toEqual(mockEntity);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/entities`,
        expect.objectContaining({
          method: 'POST',
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
            Authorization: `Bearer ${mockApiKey}`,
          }),
        })
      );
    });

    it('should get an entity', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockEntity,
      });

      const result = await client.getEntity('product_1', 'tenant_a');

      expect(result).toEqual(mockEntity);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/entities/product_1?tenant_id=tenant_a`,
        expect.any(Object)
      );
    });

    it('should update an entity', async () => {
      const updatedEntity = { ...mockEntity, attributes: { price: 79.99 } };
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => updatedEntity,
      });

      const result = await client.updateEntity('product_1', {
        attributes: { price: 79.99 },
      });

      expect(result).toEqual(updatedEntity);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/entities/product_1`,
        expect.objectContaining({
          method: 'PUT',
        })
      );
    });

    it('should delete an entity', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        status: 204,
      });

      await client.deleteEntity('product_1', 'tenant_a');

      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/entities/product_1?tenant_id=tenant_a`,
        expect.objectContaining({
          method: 'DELETE',
        })
      );
    });

    it('should bulk import entities', async () => {
      const mockResponse: BulkImportResponse = {
        total_records: 2,
        successful: 2,
        failed: 0,
        errors: [],
      };

      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.bulkImportEntities({
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
      });

      expect(result).toEqual(mockResponse);
    });
  });

  describe('Interaction Operations', () => {
    const mockInteraction: Interaction = {
      user_id: 'user_123',
      entity_id: 'product_1',
      entity_type: 'product',
      interaction_type: 'view',
      tenant_id: 'tenant_a',
      timestamp: '2024-01-01T00:00:00Z',
    };

    it('should create an interaction', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockInteraction,
      });

      const result = await client.createInteraction({
        user_id: 'user_123',
        entity_id: 'product_1',
        entity_type: 'product',
        interaction_type: 'view',
      });

      expect(result).toEqual(mockInteraction);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/interactions`,
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('should get user interactions', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => [mockInteraction],
      });

      const result = await client.getUserInteractions('user_123', {
        limit: 10,
        offset: 0,
        tenant_id: 'tenant_a',
      });

      expect(result).toEqual([mockInteraction]);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/interactions/user/user_123?limit=10&offset=0&tenant_id=tenant_a`,
        expect.any(Object)
      );
    });

    it('should bulk import interactions', async () => {
      const mockResponse: BulkImportResponse = {
        total_records: 2,
        successful: 2,
        failed: 0,
        errors: [],
      };

      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

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
      });

      expect(result).toEqual(mockResponse);
    });
  });

  describe('Recommendation Operations', () => {
    const mockRecommendationResponse: RecommendationResponse = {
      recommendations: [
        {
          entity_id: 'product_1',
          score: 0.95,
          reason: 'collaborative',
        },
        {
          entity_id: 'product_2',
          score: 0.85,
          reason: 'content_based',
        },
      ],
      algorithm: 'hybrid',
      count: 2,
      cold_start: false,
    };

    it('should get user recommendations', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockRecommendationResponse,
      });

      const result = await client.getUserRecommendations('user_123', {
        algorithm: 'hybrid',
        count: 10,
        tenant_id: 'tenant_a',
      });

      expect(result).toEqual(mockRecommendationResponse);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/recommendations/user/user_123?algorithm=hybrid&count=10&tenant_id=tenant_a`,
        expect.any(Object)
      );
    });

    it('should get similar entities', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockRecommendationResponse,
      });

      const result = await client.getSimilarEntities('product_1', {
        algorithm: 'content_based',
        count: 5,
      });

      expect(result).toEqual(mockRecommendationResponse);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/recommendations/entity/product_1?algorithm=content_based&count=5`,
        expect.any(Object)
      );
    });

    it('should get trending entities', async () => {
      const mockTrendingResponse: TrendingEntitiesResponse = {
        trending: [
          { entity_id: 'product_1', score: 0.95, reason: 'trending' },
          { entity_id: 'product_2', score: 0.85, reason: 'trending' },
        ],
        count: 2,
      };

      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        json: async () => mockTrendingResponse,
      });

      const result = await client.getTrendingEntities({
        entity_type: 'product',
        count: 10,
      });

      expect(result).toEqual(mockTrendingResponse);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/api/v1/recommendations/trending?entity_type=product&count=10`,
        expect.any(Object)
      );
    });
  });

  describe('Health & Status', () => {
    it('should check if API is healthy', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
      });

      const result = await client.isHealthy();

      expect(result).toBe(true);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/health`,
        expect.objectContaining({
          signal: expect.any(AbortSignal),
        })
      );
    });

    it('should return false when health check fails', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
        new Error('Network error')
      );

      const result = await client.isHealthy();

      expect(result).toBe(false);
    });

    it('should check if API is ready', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
      });

      const result = await client.isReady();

      expect(result).toBe(true);
      expect(fetch).toHaveBeenCalledWith(
        `${mockBaseUrl}/ready`,
        expect.any(Object)
      );
    });
  });

  describe('Error Handling', () => {
    it('should throw RecommendationError on API error', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        json: async () => ({
          error: {
            code: 404,
            message: 'Entity not found',
            details: { entity_id: 'product_1' },
          },
        }),
      });

      await expect(client.getEntity('product_1')).rejects.toThrow(
        RecommendationError
      );
    });

    it('should handle timeout errors', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockImplementationOnce(() => {
        const error = new Error('The operation was aborted');
        error.name = 'AbortError';
        return Promise.reject(error);
      });

      await expect(client.getEntity('product_1')).rejects.toThrow(
        RecommendationError
      );
    });

    it('should handle invalid JSON responses', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: async () => {
          throw new Error('Invalid JSON');
        },
      });

      await expect(client.getEntity('product_1')).rejects.toThrow(
        RecommendationError
      );
    });

    it('should handle network errors', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
        new Error('Network error')
      );

      await expect(client.getEntity('product_1')).rejects.toThrow(
        RecommendationError
      );
    });

    it('should handle 204 No Content responses', async () => {
      (fetch as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        ok: true,
        status: 204,
      });

      const result = await client.deleteEntity('product_1');

      expect(result).toBeUndefined();
    });
  });

  describe('RecommendationError', () => {
    it('should create error with message and code', () => {
      const error = new RecommendationError('Test error', 404);

      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(RecommendationError);
      expect(error.message).toBe('Test error');
      expect(error.code).toBe(404);
      expect(error.name).toBe('RecommendationError');
    });

    it('should create error with details', () => {
      const details = { entity_id: 'product_1' };
      const error = new RecommendationError('Test error', 404, details);

      expect(error.details).toEqual(details);
    });
  });
});
