/**
 * Basic usage example for the Recommendation Engine TypeScript Client
 */

import { RecommendationClient } from '../src';

async function main() {
  // Initialize the client
  const client = new RecommendationClient({
    baseUrl: 'http://localhost:8080',
    // apiKey: 'your-api-key', // Uncomment if API key is required
  });

  console.log('=== Basic Usage Example ===\n');

  // 1. Check API health
  console.log('1. Checking API health...');
  const isHealthy = await client.isHealthy();
  console.log(`API healthy: ${isHealthy}\n`);

  if (!isHealthy) {
    console.error('API is not healthy. Please start the recommendation engine API.');
    process.exit(1);
  }

  // 2. Create some entities
  console.log('2. Creating entities...');
  const product1 = await client.createEntity({
    entity_id: 'product_101',
    entity_type: 'product',
    attributes: {
      name: 'Wireless Mouse',
      category: 'Electronics',
      brand: 'TechPro',
      price: 29.99,
      rating: 4.5,
      in_stock: true,
      tags: ['wireless', 'computer', 'accessories'],
    },
  });
  console.log(`Created: ${product1.entity_id}`);

  const product2 = await client.createEntity({
    entity_id: 'product_102',
    entity_type: 'product',
    attributes: {
      name: 'Mechanical Keyboard',
      category: 'Electronics',
      brand: 'TechPro',
      price: 89.99,
      rating: 4.8,
      in_stock: true,
      tags: ['mechanical', 'computer', 'gaming'],
    },
  });
  console.log(`Created: ${product2.entity_id}\n`);

  // 3. Create interactions
  console.log('3. Recording user interactions...');
  await client.createInteraction({
    user_id: 'user_demo',
    entity_id: 'product_101',
    entity_type: 'product',
    interaction_type: 'view',
    metadata: {
      source: 'web',
      device: 'desktop',
    },
  });
  console.log('Recorded: user_demo viewed product_101');

  await client.createInteraction({
    user_id: 'user_demo',
    entity_id: 'product_101',
    entity_type: 'product',
    interaction_type: 'add_to_cart',
  });
  console.log('Recorded: user_demo added product_101 to cart');

  await client.createInteraction({
    user_id: 'user_demo',
    entity_id: 'product_102',
    entity_type: 'product',
    interaction_type: 'view',
  });
  console.log('Recorded: user_demo viewed product_102\n');

  // 4. Get user interactions
  console.log('4. Fetching user interactions...');
  const interactions = await client.getUserInteractions('user_demo', {
    limit: 10,
  });
  console.log(`Found ${interactions.length} interactions for user_demo`);
  interactions.forEach((interaction) => {
    console.log(`  - ${interaction.interaction_type} on ${interaction.entity_id}`);
  });
  console.log();

  // 5. Get recommendations
  console.log('5. Getting recommendations for user_demo...');
  const recommendations = await client.getUserRecommendations('user_demo', {
    algorithm: 'hybrid',
    count: 5,
  });

  console.log(`Algorithm: ${recommendations.algorithm}`);
  console.log(`Cold start: ${recommendations.cold_start}`);
  console.log(`Generated at: ${recommendations.generated_at}`);
  console.log('Recommendations:');
  recommendations.recommendations.forEach((rec, index) => {
    console.log(`  ${index + 1}. ${rec.entity_id} (score: ${rec.score.toFixed(3)})`);
    if (rec.reason) {
      console.log(`     Reason: ${rec.reason}`);
    }
  });
  console.log();

  // 6. Get similar entities
  console.log('6. Getting entities similar to product_101...');
  const similar = await client.getSimilarEntities('product_101', {
    algorithm: 'content_based',
    count: 5,
  });

  console.log('Similar entities:');
  similar.recommendations.forEach((rec, index) => {
    console.log(`  ${index + 1}. ${rec.entity_id} (score: ${rec.score.toFixed(3)})`);
  });
  console.log();

  // 7. Get trending entities
  console.log('7. Getting trending entities...');
  const trending = await client.getTrendingEntities({
    entity_type: 'product',
    count: 10,
  });

  console.log(`Found ${trending.count} trending entities:`);
  trending.trending.forEach((item, index) => {
    console.log(`  ${index + 1}. ${item.entity_id} (score: ${item.score.toFixed(3)})`);
  });
  console.log();

  // 8. Update an entity
  console.log('8. Updating product_101...');
  const updated = await client.updateEntity('product_101', {
    attributes: {
      price: 24.99, // On sale!
      in_stock: true,
    },
  });
  console.log(`Updated price: ${updated.attributes.price}\n`);

  // 9. Bulk import example
  console.log('9. Bulk importing entities...');
  const bulkResult = await client.bulkImportEntities({
    entities: [
      {
        entity_id: 'product_201',
        entity_type: 'product',
        attributes: {
          name: 'USB Cable',
          price: 9.99,
          category: 'Electronics',
        },
      },
      {
        entity_id: 'product_202',
        entity_type: 'product',
        attributes: {
          name: 'HDMI Cable',
          price: 14.99,
          category: 'Electronics',
        },
      },
    ],
  });
  console.log(`Bulk import: ${bulkResult.successful}/${bulkResult.total_records} successful`);
  console.log(`Job ID: ${bulkResult.job_id}\n`);

  console.log('=== Example Complete! ===');
}

// Run the example
main().catch((error) => {
  console.error('Error:', error.message);
  if (error.code) {
    console.error('Error code:', error.code);
  }
  process.exit(1);
});
