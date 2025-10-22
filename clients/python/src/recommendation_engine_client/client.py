"""Client for the Recommendation Engine API."""

from typing import Any

import httpx

from .exceptions import NetworkError, RecommendationError, TimeoutError
from .types import (
    BulkImportEntitiesRequest,
    BulkImportInteractionsRequest,
    BulkImportResponse,
    CreateEntityRequest,
    CreateInteractionRequest,
    Entity,
    EntityRecommendationsQuery,
    ErrorResponse,
    Interaction,
    RecommendationResponse,
    TrendingEntitiesQuery,
    TrendingEntitiesResponse,
    UpdateEntityRequest,
    UserRecommendationsQuery,
)


class RecommendationClient:
    """Client for the Recommendation Engine API."""

    def __init__(
        self,
        base_url: str,
        *,
        api_key: str | None = None,
        timeout: float = 30.0,
        headers: dict[str, str] | None = None,
    ) -> None:
        """Initialize the client.

        Args:
            base_url: Base URL of the Recommendation Engine API
            api_key: Optional API key for authentication
            timeout: Request timeout in seconds (default: 30.0)
            headers: Optional custom headers
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

        self._headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            **(headers or {}),
        }

        if api_key:
            self._headers["Authorization"] = f"Bearer {api_key}"

        self._client = httpx.AsyncClient(
            base_url=self.base_url,
            headers=self._headers,
            timeout=httpx.Timeout(timeout),
        )

    async def __aenter__(self) -> RecommendationClient:
        """Enter async context manager."""
        return self

    async def __aexit__(self, *args: Any) -> None:
        """Exit async context manager."""
        await self.close()

    async def close(self) -> None:
        """Close the HTTP client."""
        await self._client.aclose()

    async def _request(
        self,
        method: str,
        path: str,
        *,
        params: dict[str, Any] | None = None,
        json: dict[str, Any] | None = None,
    ) -> Any:
        """Make an HTTP request with error handling.

        Args:
            method: HTTP method
            path: API path
            params: Query parameters
            json: JSON body

        Returns:
            Response data

        Raises:
            RecommendationError: API error
            TimeoutError: Request timeout
            NetworkError: Network error
        """
        try:
            response = await self._client.request(
                method=method,
                url=path,
                params=params,
                json=json,
            )

            # Handle 204 No Content
            if response.status_code == 204:
                return None

            # Parse response
            if not response.is_success:
                try:
                    error_data: ErrorResponse = response.json()
                    error = error_data["error"]
                    raise RecommendationError(
                        message=error["message"],
                        code=error["code"],
                        details=error.get("details"),
                    )
                except (KeyError, ValueError):
                    raise RecommendationError(
                        message=response.text or response.reason_phrase,
                        code=response.status_code,
                    ) from None

            return response.json()

        except httpx.TimeoutException as exc:
            raise TimeoutError(self.timeout) from exc
        except httpx.NetworkError as exc:
            raise NetworkError(str(exc)) from exc
        except RecommendationError:
            raise
        except Exception as exc:
            raise RecommendationError(str(exc)) from exc

    # ==================== Entity Operations ====================

    async def create_entity(self, request: CreateEntityRequest) -> Entity:
        """Create a new entity.

        Args:
            request: Entity creation request

        Returns:
            The created entity

        Raises:
            RecommendationError: API error
        """
        return await self._request("POST", "/api/v1/entities", json=request)

    async def get_entity(self, entity_id: str, tenant_id: str | None = None) -> Entity:
        """Get an entity by ID.

        Args:
            entity_id: The entity ID
            tenant_id: Optional tenant ID

        Returns:
            The entity

        Raises:
            RecommendationError: API error
        """
        params = {}
        if tenant_id:
            params["tenant_id"] = tenant_id

        return await self._request("GET", f"/api/v1/entities/{entity_id}", params=params)

    async def update_entity(self, entity_id: str, request: UpdateEntityRequest) -> Entity:
        """Update an entity.

        Args:
            entity_id: The entity ID
            request: Entity update request

        Returns:
            The updated entity

        Raises:
            RecommendationError: API error
        """
        return await self._request("PUT", f"/api/v1/entities/{entity_id}", json=request)

    async def delete_entity(self, entity_id: str, tenant_id: str | None = None) -> None:
        """Delete an entity.

        Args:
            entity_id: The entity ID
            tenant_id: Optional tenant ID

        Raises:
            RecommendationError: API error
        """
        params = {}
        if tenant_id:
            params["tenant_id"] = tenant_id

        await self._request("DELETE", f"/api/v1/entities/{entity_id}", params=params)

    async def bulk_import_entities(self, request: BulkImportEntitiesRequest) -> BulkImportResponse:
        """Bulk import entities.

        Args:
            request: Bulk import request

        Returns:
            Import status

        Raises:
            RecommendationError: API error
        """
        return await self._request("POST", "/api/v1/entities/bulk", json=request)

    # ==================== Interaction Operations ====================

    async def create_interaction(self, request: CreateInteractionRequest) -> Interaction:
        """Create a new interaction.

        Args:
            request: Interaction creation request

        Returns:
            The created interaction

        Raises:
            RecommendationError: API error
        """
        return await self._request("POST", "/api/v1/interactions", json=request)

    async def get_user_interactions(
        self,
        user_id: str,
        *,
        limit: int | None = None,
        offset: int | None = None,
        tenant_id: str | None = None,
    ) -> list[Interaction]:
        """Get user interactions.

        Args:
            user_id: The user ID
            limit: Optional limit
            offset: Optional offset
            tenant_id: Optional tenant ID

        Returns:
            Array of interactions

        Raises:
            RecommendationError: API error
        """
        params = {}
        if limit is not None:
            params["limit"] = str(limit)
        if offset is not None:
            params["offset"] = str(offset)
        if tenant_id:
            params["tenant_id"] = tenant_id

        return await self._request("GET", f"/api/v1/interactions/user/{user_id}", params=params)

    async def bulk_import_interactions(
        self, request: BulkImportInteractionsRequest
    ) -> BulkImportResponse:
        """Bulk import interactions.

        Args:
            request: Bulk import request

        Returns:
            Import status

        Raises:
            RecommendationError: API error
        """
        return await self._request("POST", "/api/v1/interactions/bulk", json=request)

    # ==================== Recommendation Operations ====================

    async def get_user_recommendations(
        self,
        user_id: str,
        query: UserRecommendationsQuery | None = None,
    ) -> RecommendationResponse:
        """Get recommendations for a user.

        Args:
            user_id: The user ID
            query: Query parameters

        Returns:
            Recommendation response

        Raises:
            RecommendationError: API error
        """
        params = {}
        if query:
            params.update({k: str(v) for k, v in query.items() if v is not None})

        return await self._request("GET", f"/api/v1/recommendations/user/{user_id}", params=params)

    async def get_similar_entities(
        self,
        entity_id: str,
        query: EntityRecommendationsQuery | None = None,
    ) -> RecommendationResponse:
        """Get similar entities (content-based recommendations).

        Args:
            entity_id: The entity ID
            query: Query parameters

        Returns:
            Recommendation response

        Raises:
            RecommendationError: API error
        """
        params = {}
        if query:
            params.update({k: str(v) for k, v in query.items() if v is not None})

        return await self._request(
            "GET", f"/api/v1/recommendations/entity/{entity_id}", params=params
        )

    async def get_trending_entities(
        self,
        query: TrendingEntitiesQuery | None = None,
    ) -> TrendingEntitiesResponse:
        """Get trending entities.

        Args:
            query: Query parameters

        Returns:
            Trending entities response

        Raises:
            RecommendationError: API error
        """
        params = {}
        if query:
            params.update({k: str(v) for k, v in query.items() if v is not None})

        return await self._request("GET", "/api/v1/recommendations/trending", params=params)

    # ==================== Health & Status ====================

    async def is_healthy(self) -> bool:
        """Check if the API is healthy.

        Returns:
            True if healthy, False otherwise
        """
        try:
            response = await self._client.get("/health", timeout=5.0)
            return response.is_success
        except Exception:
            return False

    async def is_ready(self) -> bool:
        """Check if the API is ready.

        Returns:
            True if ready, False otherwise
        """
        try:
            response = await self._client.get("/ready", timeout=5.0)
            return response.is_success
        except Exception:
            return False
