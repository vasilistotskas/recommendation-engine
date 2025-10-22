/**
 * @grooveshop/recommendation-engine-client
 *
 * TypeScript/JavaScript client library for the GrooveShop Recommendation Engine
 */

export { RecommendationClient, RecommendationError } from './client.js';
export type {
  AttributeValue,
  Attributes,
  Entity,
  CreateEntityRequest,
  UpdateEntityRequest,
  InteractionType,
  Interaction,
  CreateInteractionRequest,
  ScoredEntity,
  RecommendationResponse,
  TrendingEntitiesResponse,
  BulkEntityItem,
  BulkImportEntitiesRequest,
  BulkInteractionItem,
  BulkImportInteractionsRequest,
  BulkImportResponse,
  UserRecommendationsQuery,
  EntityRecommendationsQuery,
  TrendingEntitiesQuery,
  RecommendationClientConfig,
  ApiError,
  ErrorResponse,
} from './types.js';
