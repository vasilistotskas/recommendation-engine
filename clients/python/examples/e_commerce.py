"""E-commerce recommendation example."""

import asyncio

from recommendation_engine_client import RecommendationClient


async def simulate_e_commerce() -> None:
    """Simulate an e-commerce recommendation flow."""
    async with RecommendationClient(base_url="http://localhost:8080") as client:
        print("🛒 E-Commerce Recommendation System Demo\n")

        # Sample product catalog
        products = [
            {
                "entity_id": "laptop_001",
                "entity_type": "product",
                "attributes": {
                    "name": "MacBook Pro 16",
                    "category": "laptops",
                    "brand": "Apple",
                    "price": 2499.99,
                    "specs": {"ram": "16GB", "storage": "512GB"},
                    "tags": ["professional", "high-end", "portable"],
                },
            },
            {
                "entity_id": "laptop_002",
                "entity_type": "product",
                "attributes": {
                    "name": "Dell XPS 15",
                    "category": "laptops",
                    "brand": "Dell",
                    "price": 1899.99,
                    "specs": {"ram": "16GB", "storage": "512GB"},
                    "tags": ["professional", "windows", "portable"],
                },
            },
            {
                "entity_id": "mouse_001",
                "entity_type": "product",
                "attributes": {
                    "name": "Logitech MX Master 3",
                    "category": "accessories",
                    "brand": "Logitech",
                    "price": 99.99,
                    "tags": ["wireless", "ergonomic", "productivity"],
                },
            },
            {
                "entity_id": "keyboard_001",
                "entity_type": "product",
                "attributes": {
                    "name": "Keychron K2",
                    "category": "accessories",
                    "brand": "Keychron",
                    "price": 89.99,
                    "tags": ["mechanical", "wireless", "compact"],
                },
            },
            {
                "entity_id": "monitor_001",
                "entity_type": "product",
                "attributes": {
                    "name": "LG UltraWide 34",
                    "category": "monitors",
                    "brand": "LG",
                    "price": 599.99,
                    "specs": {"resolution": "3440x1440", "size": "34-inch"},
                    "tags": ["ultrawide", "productivity", "high-resolution"],
                },
            },
        ]

        # Step 1: Import product catalog
        print("Step 1: Importing product catalog...")
        result = await client.bulk_import_entities({"entities": products})
        print(f"✓ Imported {result['successful']}/{result['total_records']} products\n")

        # Step 2: Simulate user browsing behavior
        print("Step 2: Simulating user browsing...")
        user_id = "customer_john_123"

        # User views a laptop
        await client.create_interaction(
            {
                "user_id": user_id,
                "entity_id": "laptop_001",
                "entity_type": "product",
                "interaction_type": "view",
                "metadata": {"source": "search", "device": "desktop"},
            }
        )
        print(f"✓ User {user_id} viewed MacBook Pro 16")

        # User adds laptop to cart
        await client.create_interaction(
            {
                "user_id": user_id,
                "entity_id": "laptop_001",
                "entity_type": "product",
                "interaction_type": "add_to_cart",
                "metadata": {"source": "product_page", "device": "desktop"},
            }
        )
        print(f"✓ User {user_id} added MacBook Pro 16 to cart\n")

        # Step 3: Get complementary product recommendations
        print("Step 3: Getting complementary product recommendations...")
        recs = await client.get_user_recommendations(user_id, {"algorithm": "hybrid", "count": 5})

        print(f"Recommended products for {user_id}:")
        for i, rec in enumerate(recs["recommendations"], 1):
            print(f"  {i}. {rec['entity_id']} (score: {rec['score']:.2f})")

        # Step 4: User purchases
        print("\nStep 4: User completes purchase...")
        await client.create_interaction(
            {
                "user_id": user_id,
                "entity_id": "laptop_001",
                "entity_type": "product",
                "interaction_type": "purchase",
                "metadata": {"order_id": "ORD-12345", "amount": "2499.99"},
            }
        )
        print(f"✓ User {user_id} purchased MacBook Pro 16\n")

        # Step 5: Get "frequently bought together" recommendations
        print("Step 5: Getting 'Frequently Bought Together' recommendations...")
        similar = await client.get_similar_entities(
            "laptop_001", {"algorithm": "content_based", "count": 3}
        )

        print("Customers who bought this also bought:")
        for item in similar["recommendations"]:
            print(f"  - {item['entity_id']} (relevance: {item['score']:.2f})")

        # Step 6: Show trending products
        print("\nStep 6: Displaying trending products...")
        trending = await client.get_trending_entities({"entity_type": "product", "count": 5})

        print("🔥 Trending Products:")
        for i, item in enumerate(trending["trending"], 1):
            print(f"  {i}. {item['entity_id']} (popularity: {item['score']:.2f})")

        # Step 7: Personalized email recommendations
        print(f"\nStep 7: Generating personalized email recommendations for {user_id}...")
        email_recs = await client.get_user_recommendations(
            user_id, {"algorithm": "collaborative", "count": 4}
        )

        print("📧 Email: 'Products you might like:'")
        for rec in email_recs["recommendations"]:
            print(f"  • {rec['entity_id']}")

        print("\n✅ E-commerce demo completed successfully!")


if __name__ == "__main__":
    asyncio.run(simulate_e_commerce())
