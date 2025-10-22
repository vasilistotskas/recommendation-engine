"""Python client library for the GrooveShop Recommendation Engine.

This module provides a Python client for interacting with the Recommendation Engine API.
It supports all API operations including entity management, interaction tracking, and
generating recommendations using collaborative filtering, content-based filtering, and
hybrid approaches.

Example:
    Basic usage::

        from recommendation_engine_client import RecommendationClient

        async with RecommendationClient(base_url="http://localhost:8080") as client:
            # Create an entity
            entity = await client.create_entity({
                "entity_id": "product_1",
                "entity_type": "product",
                "attributes": {"name": "Product 1", "price": 29.99}
            })

            # Get recommendations
            recs = await client.get_user_recommendations("user_123", {
                "algorithm": "hybrid",
                "count": 10
            })
"""

from .client import RecommendationClient
from .exceptions import NetworkError, RecommendationError, TimeoutError
from .types import (
    Algorithm,
    ApiError,
    Attributes,
    AttributeValue,
    BulkEntityItem,
    BulkImportEntitiesRequest,
    BulkImportError,
    BulkImportInteractionsRequest,
    BulkImportResponse,
    BulkInteractionItem,
    CreateEntityRequest,
    CreateInteractionRequest,
    Entity,
    EntityRecommendationsQuery,
    ErrorResponse,
    Interaction,
    InteractionType,
    RecommendationClientConfig,
    RecommendationResponse,
    ScoredEntity,
    TrendingEntitiesQuery,
    TrendingEntitiesResponse,
    UpdateEntityRequest,
    UserRecommendationsQuery,
)

__version__ = "1.0.0"

__all__ = [
    # Client
    "RecommendationClient",
    # Exceptions
    "RecommendationError",
    "TimeoutError",
    "NetworkError",
    # Types
    "Algorithm",
    "ApiError",
    "AttributeValue",
    "Attributes",
    "BulkEntityItem",
    "BulkImportEntitiesRequest",
    "BulkImportError",
    "BulkImportInteractionsRequest",
    "BulkImportResponse",
    "BulkInteractionItem",
    "CreateEntityRequest",
    "CreateInteractionRequest",
    "Entity",
    "EntityRecommendationsQuery",
    "ErrorResponse",
    "Interaction",
    "InteractionType",
    "RecommendationClientConfig",
    "RecommendationResponse",
    "ScoredEntity",
    "TrendingEntitiesQuery",
    "TrendingEntitiesResponse",
    "UpdateEntityRequest",
    "UserRecommendationsQuery",
]
