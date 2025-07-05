-- Database Migration Support Schema for HeSocial
-- Enables rolling deployment and schema versioning

-- Schema version tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
    id INTEGER PRIMARY KEY,
    version VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    checksum VARCHAR(255), -- For integrity verification
    execution_time INTEGER DEFAULT 0 -- milliseconds
);

-- Schema compatibility table for rolling deployments
CREATE TABLE IF NOT EXISTS schema_compatibility (
    id INTEGER PRIMARY KEY,
    schema_version VARCHAR(50) NOT NULL,
    app_version VARCHAR(50) NOT NULL,
    compatibility_level VARCHAR(20) NOT NULL CHECK (compatibility_level IN ('backward', 'forward', 'breaking')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial schema version
INSERT OR IGNORE INTO schema_migrations (version, description) 
VALUES ('v1.0.0', 'Initial HeSocial schema - luxury social events platform');

-- Mark compatibility for current version
INSERT OR IGNORE INTO schema_compatibility (schema_version, app_version, compatibility_level)
VALUES ('v1.0.0', 'v1.0.0', 'backward');