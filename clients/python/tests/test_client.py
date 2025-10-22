"""Tests for the RecommendationClient."""

import pytest
from pytest_httpx import HTTPXMock

from recommendation_engine_client import (
    RecommendationClient,
    RecommendationError,
)


@pytest.fixture
def client() -> RecommendationClient:
    """Create a test client."""
    return RecommendationClient(
        base_url="http://test.example.com",
        api_key="test-api-key",
        timeout=5.0,
    )


class TestEntityOperations:
    """Tests for entity operations."""

    async def test_create_entity(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test creating an entity."""
        httpx_mock.add_response(
            method="POST",
            url="http://test.example.com/api/v1/entities",
            json={
                "entity_id": "product_1",
                "entity_type": "product",
                "attributes": {"name": "Test Product", "price": 29.99},
                "tenant_id": None,
                "created_at": "2025-10-22T10:00:00Z",
                "updated_at": "2025-10-22T10:00:00Z",
            },
        )

        entity = await client.create_entity(
            {
                "entity_id": "product_1",
                "entity_type": "product",
                "attributes": {"name": "Test Product", "price": 29.99},
            }
        )

        assert entity["entity_id"] == "product_1"
        assert entity["entity_type"] == "product"
        assert entity["attributes"]["name"] == "Test Product"

    async def test_get_entity(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test getting an entity."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/entities/product_1",
            json={
                "entity_id": "product_1",
                "entity_type": "product",
                "attributes": {"name": "Test Product"},
                "tenant_id": None,
                "created_at": "2025-10-22T10:00:00Z",
                "updated_at": "2025-10-22T10:00:00Z",
            },
        )

        entity = await client.get_entity("product_1")

        assert entity["entity_id"] == "product_1"

    async def test_update_entity(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test updating an entity."""
        httpx_mock.add_response(
            method="PUT",
            url="http://test.example.com/api/v1/entities/product_1",
            json={
                "entity_id": "product_1",
                "entity_type": "product",
                "attributes": {"name": "Updated Product", "price": 39.99},
                "tenant_id": None,
                "created_at": "2025-10-22T10:00:00Z",
                "updated_at": "2025-10-22T10:30:00Z",
            },
        )

        entity = await client.update_entity(
            "product_1",
            {"attributes": {"name": "Updated Product", "price": 39.99}},
        )

        assert entity["attributes"]["name"] == "Updated Product"
        assert entity["attributes"]["price"] == 39.99

    async def test_delete_entity(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test deleting an entity."""
        httpx_mock.add_response(
            method="DELETE",
            url="http://test.example.com/api/v1/entities/product_1",
            status_code=204,
        )

        result = await client.delete_entity("product_1")

        assert result is None


class TestInteractionOperations:
    """Tests for interaction operations."""

    async def test_create_interaction(
        self, client: RecommendationClient, httpx_mock: HTTPXMock
    ) -> None:
        """Test creating an interaction."""
        httpx_mock.add_response(
            method="POST",
            url="http://test.example.com/api/v1/interactions",
            json={
                "id": 1,
                "user_id": "user_123",
                "entity_id": "product_1",
                "interaction_type": "view",
                "weight": 1.0,
                "metadata": None,
                "tenant_id": None,
                "timestamp": "2025-10-22T10:00:00Z",
            },
        )

        interaction = await client.create_interaction(
            {
                "user_id": "user_123",
                "entity_id": "product_1",
                "entity_type": "product",
                "interaction_type": "view",
            }
        )

        assert interaction["user_id"] == "user_123"
        assert interaction["entity_id"] == "product_1"
        assert interaction["interaction_type"] == "view"

    async def test_get_user_interactions(
        self, client: RecommendationClient, httpx_mock: HTTPXMock
    ) -> None:
        """Test getting user interactions."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/interactions/user/user_123?limit=10",
            json=[
                {
                    "id": 1,
                    "user_id": "user_123",
                    "entity_id": "product_1",
                    "interaction_type": "view",
                    "weight": 1.0,
                    "metadata": None,
                    "tenant_id": None,
                    "timestamp": "2025-10-22T10:00:00Z",
                }
            ],
        )

        interactions = await client.get_user_interactions("user_123", limit=10)

        assert len(interactions) == 1
        assert interactions[0]["user_id"] == "user_123"


class TestRecommendationOperations:
    """Tests for recommendation operations."""

    async def test_get_user_recommendations(
        self, client: RecommendationClient, httpx_mock: HTTPXMock
    ) -> None:
        """Test getting user recommendations."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/recommendations/user/user_123?algorithm=hybrid&count=10",
            json={
                "recommendations": [
                    {
                        "entity_id": "product_2",
                        "entity_type": "product",
                        "score": 0.95,
                        "reason": "Similar users liked this",
                    }
                ],
                "algorithm": "hybrid",
                "cold_start": False,
                "generated_at": "2025-10-22T10:00:00Z",
            },
        )

        recs = await client.get_user_recommendations(
            "user_123", {"algorithm": "hybrid", "count": 10}
        )

        assert len(recs["recommendations"]) == 1
        assert recs["recommendations"][0]["entity_id"] == "product_2"
        assert recs["algorithm"] == "hybrid"
        assert not recs["cold_start"]

    async def test_get_similar_entities(
        self, client: RecommendationClient, httpx_mock: HTTPXMock
    ) -> None:
        """Test getting similar entities."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/recommendations/entity/product_1?algorithm=content_based&count=5",
            json={
                "recommendations": [
                    {
                        "entity_id": "product_3",
                        "entity_type": "product",
                        "score": 0.88,
                        "reason": "Similar attributes",
                    }
                ],
                "algorithm": "content_based",
                "cold_start": False,
                "generated_at": "2025-10-22T10:00:00Z",
            },
        )

        recs = await client.get_similar_entities(
            "product_1", {"algorithm": "content_based", "count": 5}
        )

        assert len(recs["recommendations"]) == 1
        assert recs["recommendations"][0]["entity_id"] == "product_3"

    async def test_get_trending_entities(
        self, client: RecommendationClient, httpx_mock: HTTPXMock
    ) -> None:
        """Test getting trending entities."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/recommendations/trending?entity_type=product&count=20",
            json={
                "trending": [{"entity_id": "product_5", "entity_type": "product", "score": 100.0}],
                "count": 1,
            },
        )

        trending = await client.get_trending_entities({"entity_type": "product", "count": 20})

        assert trending["count"] == 1
        assert trending["trending"][0]["entity_id"] == "product_5"


class TestHealthChecks:
    """Tests for health check operations."""

    async def test_is_healthy(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test health check."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/health",
            status_code=200,
        )

        is_healthy = await client.is_healthy()

        assert is_healthy is True

    async def test_is_ready(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test readiness check."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/ready",
            status_code=200,
        )

        is_ready = await client.is_ready()

        assert is_ready is True


class TestErrorHandling:
    """Tests for error handling."""

    async def test_api_error(self, client: RecommendationClient, httpx_mock: HTTPXMock) -> None:
        """Test API error handling."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/api/v1/entities/nonexistent",
            status_code=404,
            json={
                "error": {
                    "code": 404,
                    "message": "Entity not found",
                    "details": {"entity_id": "nonexistent"},
                }
            },
        )

        with pytest.raises(RecommendationError) as exc_info:
            await client.get_entity("nonexistent")

        assert exc_info.value.code == 404
        assert "Entity not found" in exc_info.value.message


class TestContextManager:
    """Tests for async context manager."""

    async def test_context_manager(self, httpx_mock: HTTPXMock) -> None:
        """Test using client as context manager."""
        httpx_mock.add_response(
            method="GET",
            url="http://test.example.com/health",
            status_code=200,
        )

        async with RecommendationClient(base_url="http://test.example.com") as client:
            is_healthy = await client.is_healthy()
            assert is_healthy is True
