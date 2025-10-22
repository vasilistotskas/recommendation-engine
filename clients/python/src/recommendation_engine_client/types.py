"""Type definitions for the Recommendation Engine API."""

from typing import Any, Literal, TypedDict

# Type aliases
type AttributeValue = str | int | float | bool | list[str]


class Attributes(TypedDict, total=False):
    """Flexible attributes dictionary."""


# Entity types
class Entity(TypedDict):
    """Entity model."""

    entity_id: str
    entity_type: str
    attributes: dict[str, AttributeValue]
    tenant_id: str | None
    created_at: str
    updated_at: str


class CreateEntityRequest(TypedDict, total=False):
    """Create entity request."""

    entity_id: str
    entity_type: str
    attributes: dict[str, AttributeValue]
    tenant_id: str | None


class UpdateEntityRequest(TypedDict, total=False):
    """Update entity request."""

    attributes: dict[str, AttributeValue]
    tenant_id: str | None


# Interaction types
type InteractionType = (
    Literal["view", "add_to_cart", "purchase", "like"]
    | dict[Literal["rating"], float]
    | dict[Literal["custom"], str]
)


class Interaction(TypedDict, total=False):
    """Interaction model."""

    id: int | None
    user_id: str
    entity_id: str
    interaction_type: InteractionType
    weight: float
    metadata: dict[str, str] | None
    tenant_id: str | None
    timestamp: str


class CreateInteractionRequest(TypedDict, total=False):
    """Create interaction request."""

    user_id: str
    entity_id: str
    entity_type: str
    interaction_type: InteractionType
    metadata: dict[str, str] | None
    tenant_id: str | None
    timestamp: str | None


# Recommendation types
class ScoredEntity(TypedDict, total=False):
    """Scored entity in recommendation."""

    entity_id: str
    entity_type: str
    score: float
    reason: str | None


class RecommendationResponse(TypedDict):
    """Recommendation response."""

    recommendations: list[ScoredEntity]
    algorithm: str
    cold_start: bool
    generated_at: str


class TrendingEntitiesResponse(TypedDict):
    """Trending entities response."""

    trending: list[ScoredEntity]
    count: int


# Bulk import types
class BulkEntityItem(TypedDict, total=False):
    """Bulk entity item."""

    entity_id: str
    entity_type: str
    attributes: dict[str, AttributeValue]


class BulkImportEntitiesRequest(TypedDict, total=False):
    """Bulk import entities request."""

    entities: list[BulkEntityItem]
    tenant_id: str | None


class BulkInteractionItem(TypedDict, total=False):
    """Bulk interaction item."""

    user_id: str
    entity_id: str
    entity_type: str
    interaction_type: InteractionType
    metadata: dict[str, str] | None
    timestamp: str | None


class BulkImportInteractionsRequest(TypedDict, total=False):
    """Bulk import interactions request."""

    interactions: list[BulkInteractionItem]
    tenant_id: str | None


class BulkImportError(TypedDict, total=False):
    """Bulk import error."""

    entity_id: str | None
    user_id: str | None
    error: str


class BulkImportResponse(TypedDict, total=False):
    """Bulk import response."""

    job_id: str
    status: str
    total_records: int
    processed: int
    successful: int
    failed: int
    errors: list[BulkImportError] | None


# Query types
type Algorithm = Literal["collaborative", "content_based", "hybrid"]


class UserRecommendationsQuery(TypedDict, total=False):
    """User recommendations query parameters."""

    algorithm: Algorithm
    count: int
    tenant_id: str | None


class EntityRecommendationsQuery(TypedDict, total=False):
    """Entity recommendations query parameters."""

    algorithm: Algorithm
    count: int
    tenant_id: str | None
    entity_type: str | None


class TrendingEntitiesQuery(TypedDict, total=False):
    """Trending entities query parameters."""

    entity_type: str | None
    count: int
    tenant_id: str | None


# Config types
class RecommendationClientConfig(TypedDict, total=False):
    """Client configuration."""

    base_url: str
    api_key: str | None
    timeout: float
    headers: dict[str, str] | None


# Error types
class ApiError(TypedDict):
    """API error."""

    code: int
    message: str
    details: Any | None


class ErrorResponse(TypedDict):
    """Error response."""

    error: ApiError
