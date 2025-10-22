use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::Parser;
use fake::Fake;
use fake::faker::lorem::en::{Sentence, Words};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(name = "seed-data")]
#[command(about = "Seed the recommendation engine with sample data", long_about = None)]
struct Args {
    /// Base URL of the recommendation engine API
    #[arg(long, default_value = "http://localhost:8080")]
    api_url: String,

    /// API key for authentication (can also use API_KEY env var)
    #[arg(long)]
    api_key: Option<String>,

    /// Tenant ID for multi-tenancy
    #[arg(long)]
    tenant_id: Option<String>,

    /// Number of products to generate
    #[arg(long, default_value = "100")]
    num_products: usize,

    /// Number of articles to generate
    #[arg(long, default_value = "50")]
    num_articles: usize,

    /// Number of users to generate
    #[arg(long, default_value = "50")]
    num_users: usize,

    /// Number of interactions to generate
    #[arg(long, default_value = "500")]
    num_interactions: usize,

    /// Batch size for bulk imports
    #[arg(long, default_value = "100")]
    batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Debug, Clone, Serialize)]
struct BulkEntityItem {
    entity_id: String,
    entity_type: String,
    attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Serialize)]
struct BulkImportEntitiesRequest {
    entities: Vec<BulkEntityItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BulkInteractionItem {
    user_id: String,
    entity_id: String,
    entity_type: String,
    interaction_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
struct BulkImportInteractionsRequest {
    interactions: Vec<BulkInteractionItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BulkImportResponse {
    job_id: String,
    status: String,
    total_records: usize,
    processed: usize,
    successful: usize,
    failed: usize,
}

struct SampleDataGenerator {
    client: reqwest::Client,
    api_url: String,
    api_key: Option<String>,
    tenant_id: Option<String>,
}

impl SampleDataGenerator {
    fn new(api_url: String, api_key: Option<String>, tenant_id: Option<String>) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_url,
            api_key,
            tenant_id,
        }
    }

    fn generate_products(&self, count: usize) -> Vec<BulkEntityItem> {
        let categories = [
            "Electronics",
            "Clothing",
            "Books",
            "Home & Kitchen",
            "Sports",
            "Toys",
            "Beauty",
            "Automotive",
        ];
        let brands = [
            "TechPro",
            "StyleMax",
            "HomeEssentials",
            "SportFit",
            "BeautyGlow",
            "AutoMaster",
        ];
        let colors = ["Red", "Blue", "Green", "Black", "White", "Silver", "Gold"];

        let mut rng = rand::rng();
        let mut products = Vec::new();

        for i in 0..count {
            let category = categories.choose(&mut rng).unwrap();
            let brand = brands.choose(&mut rng).unwrap();
            let price = rng.random_range(10.0..1000.0);
            let rating = rng.random_range(3.0..5.0);
            let num_tags = rng.random_range(2..5);
            let tags: Vec<String> = (0..num_tags)
                .map(|_| Words(1..2).fake::<Vec<String>>().join(""))
                .collect();

            let mut attributes = HashMap::new();
            attributes.insert(
                "name".to_string(),
                AttributeValue::String(format!(
                    "{} {} {}",
                    brand,
                    category,
                    Words(2..4).fake::<Vec<String>>().join(" ")
                )),
            );
            attributes.insert(
                "description".to_string(),
                AttributeValue::String(Sentence(10..20).fake::<String>()),
            );
            attributes.insert(
                "category".to_string(),
                AttributeValue::String(category.to_string()),
            );
            attributes.insert(
                "brand".to_string(),
                AttributeValue::String(brand.to_string()),
            );
            attributes.insert("price".to_string(), AttributeValue::Number(price));
            attributes.insert("rating".to_string(), AttributeValue::Number(rating));
            attributes.insert(
                "in_stock".to_string(),
                AttributeValue::Boolean(rng.random_bool(0.9)),
            );
            attributes.insert("tags".to_string(), AttributeValue::Array(tags));

            if rng.random_bool(0.5) {
                let color = colors.choose(&mut rng).unwrap();
                attributes.insert(
                    "color".to_string(),
                    AttributeValue::String(color.to_string()),
                );
            }

            products.push(BulkEntityItem {
                entity_id: format!("product_{}", i + 1),
                entity_type: "product".to_string(),
                attributes,
            });
        }

        products
    }

    fn generate_articles(&self, count: usize) -> Vec<BulkEntityItem> {
        let categories = [
            "Technology",
            "Business",
            "Science",
            "Health",
            "Entertainment",
            "Sports",
            "Politics",
            "Lifestyle",
        ];
        let authors = [
            "John Smith",
            "Jane Doe",
            "Michael Johnson",
            "Sarah Williams",
            "David Brown",
            "Emily Davis",
        ];

        let mut rng = rand::rng();
        let mut articles = Vec::new();

        for i in 0..count {
            let category = categories.choose(&mut rng).unwrap();
            let author = authors.choose(&mut rng).unwrap();
            let read_time = rng.random_range(3..15);
            let views = rng.random_range(100..10000);
            let num_tags = rng.random_range(3..6);
            let tags: Vec<String> = (0..num_tags)
                .map(|_| Words(1..2).fake::<Vec<String>>().join(""))
                .collect();

            let mut attributes = HashMap::new();
            attributes.insert(
                "title".to_string(),
                AttributeValue::String(Sentence(5..10).fake::<String>()),
            );
            attributes.insert(
                "content".to_string(),
                AttributeValue::String(Sentence(50..100).fake::<String>()),
            );
            attributes.insert(
                "category".to_string(),
                AttributeValue::String(category.to_string()),
            );
            attributes.insert(
                "author".to_string(),
                AttributeValue::String(author.to_string()),
            );
            attributes.insert(
                "read_time_minutes".to_string(),
                AttributeValue::Number(read_time as f64),
            );
            attributes.insert("views".to_string(), AttributeValue::Number(views as f64));
            attributes.insert("tags".to_string(), AttributeValue::Array(tags));

            articles.push(BulkEntityItem {
                entity_id: format!("article_{}", i + 1),
                entity_type: "article".to_string(),
                attributes,
            });
        }

        articles
    }

    fn generate_interactions(
        &self,
        count: usize,
        num_users: usize,
        num_products: usize,
        num_articles: usize,
    ) -> Vec<BulkInteractionItem> {
        let interaction_types = ["view", "add_to_cart", "purchase", "like"];
        let mut rng = rand::rng();
        let mut interactions = Vec::new();

        for _ in 0..count {
            let user_id = format!("user_{}", rng.random_range(1..=num_users));

            // 70% products, 30% articles
            let (entity_id, entity_type) = if rng.random_bool(0.7) {
                (
                    format!("product_{}", rng.random_range(1..=num_products)),
                    "product".to_string(),
                )
            } else {
                (
                    format!("article_{}", rng.random_range(1..=num_articles)),
                    "article".to_string(),
                )
            };

            let interaction_type = interaction_types.choose(&mut rng).unwrap().to_string();

            // Generate timestamps from the last 30 days
            let days_ago = rng.random_range(0..30);
            let hours_ago = rng.random_range(0..24);
            let timestamp = Utc::now() - Duration::days(days_ago) - Duration::hours(hours_ago);

            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "web".to_string());
            metadata.insert(
                "device".to_string(),
                if rng.random_bool(0.6) {
                    "desktop"
                } else {
                    "mobile"
                }
                .to_string(),
            );

            interactions.push(BulkInteractionItem {
                user_id,
                entity_id,
                entity_type,
                interaction_type,
                metadata: Some(metadata),
                timestamp: Some(timestamp),
            });
        }

        // Sort by timestamp to simulate realistic interaction order
        interactions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        interactions
    }

    async fn bulk_import_entities(
        &self,
        entities: Vec<BulkEntityItem>,
    ) -> Result<BulkImportResponse> {
        let request = BulkImportEntitiesRequest {
            entities,
            tenant_id: self.tenant_id.clone(),
        };

        let url = format!("{}/api/v1/entities/bulk", self.api_url);
        let mut req = self.client.post(&url).json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .context("Failed to send bulk import request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Bulk import failed with status {}: {}", status, body);
        }

        let result = response
            .json::<BulkImportResponse>()
            .await
            .context("Failed to parse bulk import response")?;

        Ok(result)
    }

    async fn bulk_import_interactions(
        &self,
        interactions: Vec<BulkInteractionItem>,
    ) -> Result<BulkImportResponse> {
        let request = BulkImportInteractionsRequest {
            interactions,
            tenant_id: self.tenant_id.clone(),
        };

        let url = format!("{}/api/v1/interactions/bulk", self.api_url);
        let mut req = self.client.post(&url).json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .context("Failed to send bulk import request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Bulk import failed with status {}: {}", status, body);
        }

        let result = response
            .json::<BulkImportResponse>()
            .await
            .context("Failed to parse bulk import response")?;

        Ok(result)
    }

    async fn seed_entities(&self, entities: Vec<BulkEntityItem>, batch_size: usize) -> Result<()> {
        println!(
            "Seeding {} entities in batches of {}...",
            entities.len(),
            batch_size
        );

        let total_batches = entities.len().div_ceil(batch_size);
        let mut successful = 0;
        let mut failed = 0;

        for (i, chunk) in entities.chunks(batch_size).enumerate() {
            print!(
                "Processing batch {}/{} ({} entities)... ",
                i + 1,
                total_batches,
                chunk.len()
            );

            match self.bulk_import_entities(chunk.to_vec()).await {
                Ok(response) => {
                    println!(
                        "✓ Success: {}/{} entities imported",
                        response.successful, response.total_records
                    );
                    successful += response.successful;
                    failed += response.failed;
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                    failed += chunk.len();
                }
            }
        }

        println!("\nEntity seeding complete:");
        println!("  Total: {}", entities.len());
        println!("  Successful: {}", successful);
        println!("  Failed: {}", failed);

        Ok(())
    }

    async fn seed_interactions(
        &self,
        interactions: Vec<BulkInteractionItem>,
        batch_size: usize,
    ) -> Result<()> {
        println!(
            "\nSeeding {} interactions in batches of {}...",
            interactions.len(),
            batch_size
        );

        let total_batches = interactions.len().div_ceil(batch_size);
        let mut successful = 0;
        let mut failed = 0;

        for (i, chunk) in interactions.chunks(batch_size).enumerate() {
            print!(
                "Processing batch {}/{} ({} interactions)... ",
                i + 1,
                total_batches,
                chunk.len()
            );

            match self.bulk_import_interactions(chunk.to_vec()).await {
                Ok(response) => {
                    println!(
                        "✓ Success: {}/{} interactions imported",
                        response.successful, response.total_records
                    );
                    successful += response.successful;
                    failed += response.failed;
                }
                Err(e) => {
                    println!("✗ Error: {}", e);
                    failed += chunk.len();
                }
            }
        }

        println!("\nInteraction seeding complete:");
        println!("  Total: {}", interactions.len());
        println!("  Successful: {}", successful);
        println!("  Failed: {}", failed);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    // Allow API_KEY to be set via environment variable
    if args.api_key.is_none() {
        args.api_key = std::env::var("API_KEY").ok();
    }

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   Recommendation Engine - Sample Data Seeder             ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  API URL: {}", args.api_url);
    println!(
        "  Tenant ID: {}",
        args.tenant_id.as_deref().unwrap_or("default")
    );
    println!("  Products: {}", args.num_products);
    println!("  Articles: {}", args.num_articles);
    println!("  Users: {}", args.num_users);
    println!("  Interactions: {}", args.num_interactions);
    println!("  Batch Size: {}", args.batch_size);
    println!();

    let generator = SampleDataGenerator::new(args.api_url, args.api_key, args.tenant_id);

    // Generate sample data
    println!("Generating sample data...");
    let products = generator.generate_products(args.num_products);
    println!("✓ Generated {} products", products.len());

    let articles = generator.generate_articles(args.num_articles);
    println!("✓ Generated {} articles", articles.len());

    let interactions = generator.generate_interactions(
        args.num_interactions,
        args.num_users,
        args.num_products,
        args.num_articles,
    );
    println!("✓ Generated {} interactions", interactions.len());
    println!();

    // Seed entities
    let mut all_entities = Vec::new();
    all_entities.extend(products);
    all_entities.extend(articles);

    generator
        .seed_entities(all_entities, args.batch_size)
        .await?;

    // Seed interactions
    generator
        .seed_interactions(interactions, args.batch_size)
        .await?;

    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   Seeding Complete! 🎉                                    ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("You can now:");
    println!("  1. Get recommendations: GET /api/v1/recommendations/user/user_1");
    println!("  2. View trending: GET /api/v1/recommendations/trending");
    println!("  3. Find similar items: GET /api/v1/recommendations/entity/product_1");

    Ok(())
}
