-- Create interactions table for tracking user-entity interactions
CREATE TABLE interactions (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL DEFAULT 'default',
    interaction_type VARCHAR(50) NOT NULL,
    weight FLOAT NOT NULL DEFAULT 1.0,
    metadata JSONB,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_interaction UNIQUE (tenant_id, user_id, entity_id, interaction_type, timestamp)
);

-- Index for user interaction history queries
CREATE INDEX idx_interactions_user ON interactions(tenant_id, user_id, timestamp DESC);

-- Index for entity interaction queries
CREATE INDEX idx_interactions_entity ON interactions(tenant_id, entity_id, timestamp DESC);

-- Index for interaction type filtering
CREATE INDEX idx_interactions_type ON interactions(tenant_id, interaction_type);

-- Index for time-based queries (trending calculations)
CREATE INDEX idx_interactions_timestamp ON interactions(tenant_id, timestamp DESC);

-- Composite index for user-entity lookups (deduplication)
CREATE INDEX idx_interactions_user_entity ON interactions(tenant_id, user_id, entity_id, interaction_type);

-- Comment on table
COMMENT ON TABLE interactions IS 'Stores user-entity interaction events for collaborative filtering and behavior tracking';
COMMENT ON COLUMN interactions.weight IS 'Configurable weight for different interaction types (view=1.0, purchase=5.0, etc.)';
COMMENT ON COLUMN interactions.metadata IS 'Optional additional context about the interaction (e.g., read_time, rating_value)';
COMMENT ON COLUMN interactions.tenant_id IS 'Tenant identifier for multi-tenancy support';
