"""Basic usage example for the Recommendation Engine Python client."""

import asyncio

from recommendation_engine_client import RecommendationClient, RecommendationError


async def main() -> None:
    """Run the basic usage example."""
    # Initialize the client
    async with RecommendationClient(
        base_url="http://localhost:8080",
        # api_key="your-api-key",  # Optional
        timeout=30.0,
    ) as client:
        try:
            # Check if API is healthy
            print("Checking API health...")
            is_healthy = await client.is_healthy()
            print(f"✓ API is {'healthy' if is_healthy else 'unhealthy'}")

            is_ready = await client.is_ready()
            print(f"✓ API is {'ready' if is_ready else 'not ready'}")

            # Create entities
            print("\nCreating entities...")
            for i in range(1, 4):
                entity = await client.create_entity(
                    {
                        "entity_id": f"product_{i}",
                        "entity_type": "product",
                        "attributes": {
                            "name": f"Product {i}",
                            "category": "electronics" if i % 2 == 0 else "books",
                            "price": i * 10.0 + 9.99,
                            "rating": 4.0 + (i % 2) * 0.5,
                        },
                    }
                )
                print(f"✓ Created entity: {entity['entity_id']}")

            # Create interactions
            print("\nCreating interactions...")
            interactions_data = [
                ("user_1", "product_1", "view"),
                ("user_1", "product_2", "add_to_cart"),
                ("user_1", "product_3", "purchase"),
                ("user_2", "product_1", "view"),
                ("user_2", "product_3", "view"),
            ]

            for user_id, entity_id, interaction_type in interactions_data:
                interaction = await client.create_interaction(
                    {
                        "user_id": user_id,
                        "entity_id": entity_id,
                        "entity_type": "product",
                        "interaction_type": interaction_type,
                    }
                )
                print(
                    f"✓ Created interaction: {interaction['user_id']} "
                    f"-> {interaction['entity_id']} ({interaction['interaction_type']})"
                )

            # Get recommendations
            print("\nGetting recommendations for user_1...")
            recs = await client.get_user_recommendations(
                "user_1", {"algorithm": "hybrid", "count": 5}
            )
            print(f"Algorithm: {recs['algorithm']}")
            print(f"Cold start: {recs['cold_start']}")
            print("Recommendations:")
            for rec in recs["recommendations"]:
                print(
                    f"  - {rec['entity_id']}: score={rec['score']:.2f}, "
                    f"reason={rec.get('reason', 'N/A')}"
                )

            # Get similar entities
            print("\nGetting similar entities to product_1...")
            similar = await client.get_similar_entities(
                "product_1", {"algorithm": "content_based", "count": 3}
            )
            print("Similar entities:")
            for item in similar["recommendations"]:
                print(f"  - {item['entity_id']}: score={item['score']:.2f}")

            # Get trending entities
            print("\nGetting trending entities...")
            trending = await client.get_trending_entities({"entity_type": "product", "count": 10})
            print(f"Total trending: {trending['count']}")
            print("Trending entities:")
            for index, item in enumerate(trending["trending"][:5], 1):
                print(f"  {index}. {item['entity_id']}: score={item['score']:.2f}")

            # Get user interactions
            print("\nGetting interactions for user_1...")
            user_interactions = await client.get_user_interactions("user_1", limit=10)
            print(f"Total interactions: {len(user_interactions)}")
            for interaction in user_interactions:
                print(
                    f"  - {interaction['entity_id']}: {interaction['interaction_type']} "
                    f"(weight: {interaction['weight']})"
                )

        except RecommendationError as error:
            print(f"❌ Error: [{error.code}] {error.message}")
            if error.details:
                print(f"Details: {error.details}")


if __name__ == "__main__":
    asyncio.run(main())
