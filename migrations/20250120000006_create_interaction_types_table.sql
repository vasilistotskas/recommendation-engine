-- Create interaction_types table for custom interaction type registry
CREATE TABLE interaction_types (
    id BIGSERIAL PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL DEFAULT 'default',
    interaction_type VARCHAR(50) NOT NULL,
    weight FLOAT NOT NULL DEFAULT 1.0,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_interaction_type UNIQUE (tenant_id, interaction_type)
);

-- Index for tenant-based lookups
CREATE INDEX idx_interaction_types_tenant ON interaction_types(tenant_id);

-- Index for interaction type lookups
CREATE INDEX idx_interaction_types_type ON interaction_types(tenant_id, interaction_type);

-- Comment on table
COMMENT ON TABLE interaction_types IS 'Registry of custom interaction types with configurable weights per tenant';
COMMENT ON COLUMN interaction_types.weight IS 'Weight assigned to this interaction type for recommendation scoring';
COMMENT ON COLUMN interaction_types.description IS 'Optional description of what this interaction type represents';
COMMENT ON COLUMN interaction_types.tenant_id IS 'Tenant identifier for multi-tenancy support';

-- Insert default interaction types for the default tenant
INSERT INTO interaction_types (tenant_id, interaction_type, weight, description) VALUES
    ('default', 'view', 1.0, 'User viewed an entity'),
    ('default', 'add_to_cart', 3.0, 'User added entity to cart'),
    ('default', 'purchase', 5.0, 'User purchased entity'),
    ('default', 'like', 2.0, 'User liked entity');
