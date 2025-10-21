-- Create user_profiles table with preference vectors for collaborative filtering
CREATE TABLE user_profiles (
    user_id VARCHAR(255) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL DEFAULT 'default',
    preference_vector vector(512),
    interaction_count INT NOT NULL DEFAULT 0,
    last_interaction_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, user_id)
);

-- Index for interaction count queries (cold start detection)
CREATE INDEX idx_user_profiles_interaction_count ON user_profiles(tenant_id, interaction_count);

-- Index for last interaction timestamp
CREATE INDEX idx_user_profiles_last_interaction ON user_profiles(tenant_id, last_interaction_at DESC);

-- HNSW index for user similarity search using cosine distance
-- This enables finding similar users for collaborative filtering
CREATE INDEX idx_user_profiles_preference_vector_hnsw ON user_profiles 
    USING hnsw (preference_vector vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Comment on table
COMMENT ON TABLE user_profiles IS 'Stores user preference vectors computed from interaction history for collaborative filtering';
COMMENT ON COLUMN user_profiles.preference_vector IS 'Aggregated preference vector representing user tastes based on interactions';
COMMENT ON COLUMN user_profiles.interaction_count IS 'Total number of interactions for cold start detection';
COMMENT ON COLUMN user_profiles.tenant_id IS 'Tenant identifier for multi-tenancy support';
