-- Create trending_entities table for caching trending calculations
CREATE TABLE trending_entities (
    entity_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL DEFAULT 'default',
    score FLOAT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, entity_id, window_start)
);

-- Index for trending lookups by entity type and time window
CREATE INDEX idx_trending_lookup ON trending_entities(tenant_id, entity_type, window_start, score DESC);

-- Index for time-based cleanup of old trending data
CREATE INDEX idx_trending_window_end ON trending_entities(window_end);

-- Comment on table
COMMENT ON TABLE trending_entities IS 'Caches trending entity calculations for cold start scenarios and popular recommendations';
COMMENT ON COLUMN trending_entities.score IS 'Trending score based on interaction frequency in the time window';
COMMENT ON COLUMN trending_entities.window_start IS 'Start of the time window for trending calculation (typically last 7 days)';
COMMENT ON COLUMN trending_entities.window_end IS 'End of the time window for trending calculation';
COMMENT ON COLUMN trending_entities.tenant_id IS 'Tenant identifier for multi-tenancy support';
