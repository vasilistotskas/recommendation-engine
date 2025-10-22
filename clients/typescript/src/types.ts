/**
 * Type definitions for the Recommendation Engine API
 */

export type AttributeValue = string | number | boolean | string[];

export interface Attributes {
  [key: string]: AttributeValue;
}

export interface Entity {
  entity_id: string;
  entity_type: string;
  attributes: Attributes;
  tenant_id?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateEntityRequest {
  entity_id: string;
  entity_type: string;
  attributes: Attributes;
  tenant_id?: string;
}

export interface UpdateEntityRequest {
  attributes: Attributes;
  tenant_id?: string;
}

export type InteractionType =
  | 'view'
  | 'add_to_cart'
  | 'purchase'
  | 'like'
  | { rating: number }
  | { custom: string };

export interface Interaction {
  id?: number;
  user_id: string;
  entity_id: string;
  interaction_type: InteractionType;
  weight: number;
  metadata?: Record<string, string>;
  tenant_id?: string;
  timestamp: string;
}

export interface CreateInteractionRequest {
  user_id: string;
  entity_id: string;
  entity_type: string;
  interaction_type: InteractionType;
  metadata?: Record<string, string>;
  tenant_id?: string;
  timestamp?: string;
}

export interface ScoredEntity {
  entity_id: string;
  entity_type: string;
  score: number;
  reason?: string;
}

export interface RecommendationResponse {
  recommendations: ScoredEntity[];
  algorithm: string;
  cold_start: boolean;
  generated_at: string;
}

export interface TrendingEntitiesResponse {
  trending: ScoredEntity[];
  count: number;
}

export interface BulkEntityItem {
  entity_id: string;
  entity_type: string;
  attributes: Attributes;
}

export interface BulkImportEntitiesRequest {
  entities: BulkEntityItem[];
  tenant_id?: string;
}

export interface BulkInteractionItem {
  user_id: string;
  entity_id: string;
  entity_type: string;
  interaction_type: InteractionType;
  metadata?: Record<string, string>;
  timestamp?: string;
}

export interface BulkImportInteractionsRequest {
  interactions: BulkInteractionItem[];
  tenant_id?: string;
}

export interface BulkImportResponse {
  job_id: string;
  status: string;
  total_records: number;
  processed: number;
  successful: number;
  failed: number;
  errors?: Array<{
    entity_id?: string;
    user_id?: string;
    error: string;
  }>;
}

export interface UserRecommendationsQuery {
  algorithm?: 'collaborative' | 'content_based' | 'hybrid';
  count?: number;
  tenant_id?: string;
  [key: string]: string | number | undefined;
}

export interface EntityRecommendationsQuery {
  algorithm?: 'collaborative' | 'content_based' | 'hybrid';
  count?: number;
  tenant_id?: string;
  entity_type?: string;
}

export interface TrendingEntitiesQuery {
  entity_type?: string;
  count?: number;
  tenant_id?: string;
}

export interface RecommendationClientConfig {
  baseUrl: string;
  apiKey?: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export interface ApiError {
  code: number;
  message: string;
  details?: unknown;
}

export interface ErrorResponse {
  error: ApiError;
}
