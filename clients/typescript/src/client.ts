import type {
  RecommendationClientConfig,
  Entity,
  CreateEntityRequest,
  UpdateEntityRequest,
  Interaction,
  CreateInteractionRequest,
  RecommendationResponse,
  TrendingEntitiesResponse,
  BulkImportEntitiesRequest,
  BulkImportInteractionsRequest,
  BulkImportResponse,
  UserRecommendationsQuery,
  EntityRecommendationsQuery,
  TrendingEntitiesQuery,
  ErrorResponse,
} from './types.js';

/**
 * Client for the Recommendation Engine API
 */
export class RecommendationClient {
  private baseUrl: string;
  private headers: Record<string, string>;
  private timeout: number;

  /**
   * Creates a new RecommendationClient instance
   * @param config - Client configuration
   */
  constructor(config: RecommendationClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = config.timeout || 30000;

    this.headers = {
      'Content-Type': 'application/json',
      ...config.headers,
    };

    if (config.apiKey) {
      this.headers['Authorization'] = `Bearer ${config.apiKey}`;
    }
  }

  /**
   * Make a fetch request with timeout and error handling
   */
  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          ...this.headers,
          ...options.headers,
        },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({
          error: {
            code: response.status,
            message: response.statusText,
          },
        })) as ErrorResponse;

        throw new RecommendationError(
          errorData.error.message,
          errorData.error.code,
          errorData.error.details
        );
      }

      // Handle 204 No Content
      if (response.status === 204) {
        return undefined as T;
      }

      return (await response.json()) as T;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof RecommendationError) {
        throw error;
      }

      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          throw new RecommendationError(
            `Request timeout after ${this.timeout}ms`,
            408
          );
        }
        throw new RecommendationError(error.message, 0);
      }

      throw error;
    }
  }

  // ==================== Entity Operations ====================

  /**
   * Create a new entity
   * @param request - Entity creation request
   * @returns The created entity
   */
  async createEntity(request: CreateEntityRequest): Promise<Entity> {
    return this.request<Entity>('/api/v1/entities', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  /**
   * Get an entity by ID
   * @param entityId - The entity ID
   * @param tenantId - Optional tenant ID
   * @returns The entity
   */
  async getEntity(entityId: string, tenantId?: string): Promise<Entity> {
    const params = new URLSearchParams();
    if (tenantId) params.set('tenant_id', tenantId);

    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<Entity>(`/api/v1/entities/${entityId}${query}`);
  }

  /**
   * Update an entity
   * @param entityId - The entity ID
   * @param request - Entity update request
   * @returns The updated entity
   */
  async updateEntity(
    entityId: string,
    request: UpdateEntityRequest
  ): Promise<Entity> {
    return this.request<Entity>(`/api/v1/entities/${entityId}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  /**
   * Delete an entity
   * @param entityId - The entity ID
   * @param tenantId - Optional tenant ID
   */
  async deleteEntity(entityId: string, tenantId?: string): Promise<void> {
    const params = new URLSearchParams();
    if (tenantId) params.set('tenant_id', tenantId);

    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<void>(`/api/v1/entities/${entityId}${query}`, {
      method: 'DELETE',
    });
  }

  /**
   * Bulk import entities
   * @param request - Bulk import request
   * @returns Import status
   */
  async bulkImportEntities(
    request: BulkImportEntitiesRequest
  ): Promise<BulkImportResponse> {
    return this.request<BulkImportResponse>('/api/v1/entities/bulk', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  // ==================== Interaction Operations ====================

  /**
   * Create a new interaction
   * @param request - Interaction creation request
   * @returns The created interaction
   */
  async createInteraction(
    request: CreateInteractionRequest
  ): Promise<Interaction> {
    return this.request<Interaction>('/api/v1/interactions', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  /**
   * Get user interactions
   * @param userId - The user ID
   * @param options - Query options (limit, offset, tenant_id)
   * @returns Array of interactions
   */
  async getUserInteractions(
    userId: string,
    options?: { limit?: number; offset?: number; tenant_id?: string }
  ): Promise<Interaction[]> {
    const params = new URLSearchParams();
    if (options?.limit !== undefined) params.set('limit', options.limit.toString());
    if (options?.offset !== undefined) params.set('offset', options.offset.toString());
    if (options?.tenant_id) params.set('tenant_id', options.tenant_id);

    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<Interaction[]>(
      `/api/v1/interactions/user/${userId}${query}`
    );
  }

  /**
   * Bulk import interactions
   * @param request - Bulk import request
   * @returns Import status
   */
  async bulkImportInteractions(
    request: BulkImportInteractionsRequest
  ): Promise<BulkImportResponse> {
    return this.request<BulkImportResponse>('/api/v1/interactions/bulk', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  // ==================== Recommendation Operations ====================

  /**
   * Get recommendations for a user
   * @param userId - The user ID
   * @param query - Query parameters
   * @returns Recommendation response
   */
  async getUserRecommendations(
    userId: string,
    query?: UserRecommendationsQuery
  ): Promise<RecommendationResponse> {
    const params = new URLSearchParams();
    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        if (value !== undefined) {
          params.set(key, String(value));
        }
      });
    }

    const queryString = params.toString() ? `?${params.toString()}` : '';
    return this.request<RecommendationResponse>(
      `/api/v1/recommendations/user/${userId}${queryString}`
    );
  }

  /**
   * Get similar entities (content-based recommendations)
   * @param entityId - The entity ID
   * @param query - Query parameters
   * @returns Recommendation response
   */
  async getSimilarEntities(
    entityId: string,
    query?: EntityRecommendationsQuery
  ): Promise<RecommendationResponse> {
    const params = new URLSearchParams();
    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        if (value !== undefined) {
          params.set(key, String(value));
        }
      });
    }

    const queryString = params.toString() ? `?${params.toString()}` : '';
    return this.request<RecommendationResponse>(
      `/api/v1/recommendations/entity/${entityId}${queryString}`
    );
  }

  /**
   * Get trending entities
   * @param query - Query parameters
   * @returns Trending entities response
   */
  async getTrendingEntities(
    query?: TrendingEntitiesQuery
  ): Promise<TrendingEntitiesResponse> {
    const params = new URLSearchParams();
    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        if (value !== undefined) {
          params.set(key, String(value));
        }
      });
    }

    const queryString = params.toString() ? `?${params.toString()}` : '';
    return this.request<TrendingEntitiesResponse>(
      `/api/v1/recommendations/trending${queryString}`
    );
  }

  // ==================== Health & Status ====================

  /**
   * Check if the API is healthy
   * @returns true if healthy
   */
  async isHealthy(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/health`, {
        signal: AbortSignal.timeout(5000),
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Check if the API is ready
   * @returns true if ready
   */
  async isReady(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/ready`, {
        signal: AbortSignal.timeout(5000),
      });
      return response.ok;
    } catch {
      return false;
    }
  }
}

/**
 * Custom error class for Recommendation Engine API errors
 */
export class RecommendationError extends Error {
  constructor(
    message: string,
    public code: number,
    public details?: unknown
  ) {
    super(message);
    this.name = 'RecommendationError';
    Object.setPrototypeOf(this, RecommendationError.prototype);
  }
}
