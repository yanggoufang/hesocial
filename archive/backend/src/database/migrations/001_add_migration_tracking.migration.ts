import { BaseMigration } from './Migration.js';

export default class AddMigrationTracking extends BaseMigration {
  id = '001_add_migration_tracking';
  version = 1;
  name = 'Add Migration Tracking';
  description = 'Create schema_migrations table for tracking database changes';
  category = 'schema' as const;

  async up(): Promise<void> {
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS schema_migrations (
        id VARCHAR(255) PRIMARY KEY,
        version INTEGER NOT NULL,
        name VARCHAR(255) NOT NULL,
        description TEXT,
        category VARCHAR(50) DEFAULT 'schema',
        checksum VARCHAR(255) NOT NULL,
        applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        execution_time INTEGER DEFAULT 0,
        applied_by VARCHAR(100) DEFAULT 'system',
        rollback_sql TEXT,
        can_rollback BOOLEAN DEFAULT false
      );
    `);

    // Create index for faster lookups
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_schema_migrations_version 
      ON schema_migrations(version);
    `);

    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_schema_migrations_applied_at 
      ON schema_migrations(applied_at);
    `);
  }

  async down(): Promise<void> {
    // Drop indexes first
    await this.executeSQL(`DROP INDEX IF EXISTS idx_schema_migrations_applied_at;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_schema_migrations_version;`);
    
    // Drop the table
    await this.executeSQL(`DROP TABLE IF EXISTS schema_migrations;`);
  }
}