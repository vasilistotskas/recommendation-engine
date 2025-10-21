-- Create entities table with pgvector column for feature vectors
CREATE TABLE entities (
    entity_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL DEFAULT 'default',
    attributes JSONB NOT NULL,
    feature_vector vector(512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, entity_id, entity_type)
);

-- Index for entity type lookups
CREATE INDEX idx_entities_type ON entities(tenant_id, entity_type);

-- Index for timestamp-based queries
CREATE INDEX idx_entities_created_at ON entities(tenant_id, created_at DESC);
CREATE INDEX idx_entities_updated_at ON entities(tenant_id, updated_at DESC);

-- HNSW index for vector similarity search using cosine distance
-- This enables sub-linear time nearest neighbor search
CREATE INDEX idx_entities_feature_vector_hnsw ON entities 
    USING hnsw (feature_vector vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Comment on table
COMMENT ON TABLE entities IS 'Stores domain-agnostic entities (products, articles, users, etc.) with their feature vectors for content-based filtering';
COMMENT ON COLUMN entities.feature_vector IS 'High-dimensional vector representation of entity attributes for similarity search';
COMMENT ON COLUMN entities.tenant_id IS 'Tenant identifier for multi-tenancy support';
