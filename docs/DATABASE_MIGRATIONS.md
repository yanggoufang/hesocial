# Database Migration System

## Overview

The HeSocial platform features a comprehensive database migration system that provides safe, version-controlled schema changes with rollback capabilities. This system ensures reliable database evolution across development, staging, and production environments.

## Features

### 🔄 **Version Control**
- Sequential versioning with unique migration IDs
- Dependency tracking between migrations
- Comprehensive migration history and audit trail
- Checksum validation to detect modified migrations

### 🛡️ **Safety & Reliability**
- Transaction-wrapped execution for atomicity
- Rollback capabilities with safety validation
- Risk assessment for dangerous operations
- Backup integration before risky changes
- Dry-run mode for preview without execution

### 🔧 **Developer Experience**
- CLI tools for migration management
- Template generation for new migrations
- Comprehensive error reporting
- Integration with startup health checks

## Quick Start

### 1. Check Migration Status
```bash
npm run migrate:status
```

### 2. Create New Migration
```bash
npm run migrate:create
```

### 3. Apply Pending Migrations
```bash
npm run migrate:up
```

### 4. Rollback to Previous Version
```bash
npm run migrate:rollback <version>
```

## Migration CLI Commands

### Status Commands
```bash
npm run migrate:status          # Show current migration status
npm run migrate:validate        # Validate migration integrity
npm run migrate                 # Show help and available commands
```

### Execution Commands
```bash
npm run migrate:up              # Apply all pending migrations
npm run migrate:up --dry-run    # Preview migrations without executing

npm run migrate:rollback <ver>  # Rollback to specific version
npm run migrate:rollback <ver> --force  # Force rollback (ignore risks)
npm run migrate:rollback <ver> --dry-run  # Preview rollback
```

### Development Commands
```bash
npm run migrate:create          # Generate new migration template
npm run migrate:validate        # Validate all migrations
```

## Creating Migrations

### 1. Generate Template
```bash
npm run migrate:create
```

This generates a template like:
```typescript
// File: 1234567890_example_migration.migration.ts
import { BaseMigration } from '../Migration.js';

export default class ExampleMigration extends BaseMigration {
  id = '1234567890_example_migration';
  version = 1234567890;
  name = 'Example Migration';
  description = 'Description of what this migration does';
  category = 'schema' as const;

  async up(): Promise<void> {
    // Add your migration logic here
    await this.executeSQL(`
      -- Example: Add new column
      ALTER TABLE users ADD COLUMN new_column VARCHAR(255);
    `);
  }

  async down(): Promise<void> {
    // Add your rollback logic here
    await this.executeSQL(`
      -- Example: Remove the column
      ALTER TABLE users DROP COLUMN new_column;
    `);
  }
}
```

### 2. Customize Migration
- **Update the name and description** to reflect your changes
- **Implement the `up()` method** with your schema changes
- **Implement the `down()` method** with rollback logic
- **Set the category** (`schema`, `data`, `index`, `constraint`)
- **Add dependencies** if this migration depends on others

### 3. Migration Categories
- **`schema`**: Table structure changes (CREATE, ALTER, DROP)
- **`data`**: Data modifications (INSERT, UPDATE, DELETE)
- **`index`**: Index management (CREATE INDEX, DROP INDEX)
- **`constraint`**: Constraint changes (ADD CONSTRAINT, DROP CONSTRAINT)

## Migration Examples

### Schema Migration
```typescript
export default class AddUserPreferences extends BaseMigration {
  id = '001_add_user_preferences';
  version = 1;
  name = 'Add User Preferences';
  description = 'Create user_preferences table for storing user settings';
  category = 'schema' as const;

  async up(): Promise<void> {
    await this.executeSQL(`
      CREATE TABLE user_preferences (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        preference_key VARCHAR(100) NOT NULL,
        preference_value TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        UNIQUE(user_id, preference_key)
      );
    `);

    await this.executeSQL(`
      CREATE INDEX idx_user_preferences_user_id 
      ON user_preferences(user_id);
    `);
  }

  async down(): Promise<void> {
    await this.executeSQL(`DROP INDEX IF EXISTS idx_user_preferences_user_id;`);
    await this.executeSQL(`DROP TABLE IF EXISTS user_preferences;`);
  }
}
```

### Data Migration
```typescript
export default class PopulateDefaultPreferences extends BaseMigration {
  id = '002_populate_default_preferences';
  version = 2;
  name = 'Populate Default Preferences';
  description = 'Add default preferences for existing users';
  category = 'data' as const;
  dependencies = ['1']; // Depends on user_preferences table

  async up(): Promise<void> {
    // Add default theme preference for all users
    await this.executeSQL(`
      INSERT INTO user_preferences (user_id, preference_key, preference_value)
      SELECT id, 'theme', 'dark' FROM users;
    `);

    // Add default notification preference
    await this.executeSQL(`
      INSERT INTO user_preferences (user_id, preference_key, preference_value)
      SELECT id, 'notifications', 'true' FROM users;
    `);
  }

  async down(): Promise<void> {
    await this.executeSQL(`
      DELETE FROM user_preferences 
      WHERE preference_key IN ('theme', 'notifications');
    `);
  }
}
```

## Safety Features

### Risk Assessment
The migration system automatically identifies risky operations:
- **Data migrations**: May cause data loss during rollback
- **Column drops**: Irreversible data loss
- **Table drops**: Complete data loss
- **Missing rollback methods**: Cannot be undone

### Safety Checks
- **Dependency validation**: Ensures required migrations are applied
- **Checksum verification**: Detects modified migrations
- **Production safeguards**: Extra validation in production environments
- **Backup integration**: Automatic backups before risky operations

### Force Mode
Use `--force` flag to override safety checks:
```bash
npm run migrate:rollback 1 --force
```
⚠️ **Warning**: Only use force mode when you understand the risks.

## Integration with Server

### Startup Integration
The migration system is integrated into the server startup sequence:

1. **Configuration validation**
2. **Service initialization**
3. **Smart backup restore**
4. **Database connection**
5. **🔄 Migration checks** ← Migration system runs here
6. **Health validation**
7. **Route loading**
8. **Server start**

### Auto-Migration
Configure automatic migration execution:
```bash
# Enable auto-migration (default: disabled)
AUTO_MIGRATE=true

# Require confirmation for risky operations (default: true)
MIGRATION_REQUIRE_CONFIRMATION=false

# Maximum migration time limit (default: 5 minutes)
MAX_MIGRATION_TIME=300000
```

### Health Checks
Migration status is included in health endpoints:
- `GET /api/health` - Basic health with migration status
- `GET /api/health/detailed` - Comprehensive status including migration details

## Best Practices

### 1. Migration Design
- **Keep migrations small** and focused on single concerns
- **Always include rollback logic** (`down()` method)
- **Test migrations** in development before applying to production
- **Use descriptive names** and descriptions
- **Consider data preservation** during schema changes

### 2. Schema Changes
```typescript
// Good: Add column with default value
await this.executeSQL(`
  ALTER TABLE users 
  ADD COLUMN phone_number VARCHAR(20) DEFAULT NULL;
`);

// Good: Create index concurrently (if supported)
await this.executeSQL(`
  CREATE INDEX CONCURRENTLY idx_users_email 
  ON users(email);
`);
```

### 3. Data Migrations
```typescript
// Good: Batch process large datasets
await this.executeSQL(`
  UPDATE users 
  SET status = 'active' 
  WHERE created_at > '2024-01-01' 
  AND status IS NULL;
`);
```

### 4. Rollback Planning
```typescript
// Good: Preserve data during rollback
async down(): Promise<void> {
  // Create backup table before dropping
  await this.executeSQL(`
    CREATE TABLE users_backup AS 
    SELECT * FROM users;
  `);
  
  // Then drop the original
  await this.executeSQL(`DROP TABLE users;`);
}
```

## Troubleshooting

### Migration Stuck
If a migration appears stuck:
1. Check database locks: `npm run migrate:status`
2. Review migration logs for errors
3. Manually inspect database state
4. Consider force rollback if necessary

### Failed Migration
If a migration fails:
1. Review error logs for specific issues
2. Check database state for partial changes
3. Fix the migration code
4. Re-run with `npm run migrate:up`

### Rollback Issues
If rollback fails:
1. Use `--force` flag to override safety checks
2. Manually restore from backup if necessary
3. Check for foreign key constraints
4. Verify rollback SQL syntax

### Checksum Mismatch
If checksums don't match:
1. A migration file was modified after being applied
2. Restore original migration file
3. Create new migration for additional changes
4. Never modify applied migrations

## File Structure

```
backend/src/database/migrations/
├── Migration.ts                           # Base classes and interfaces
├── MigrationRunner.ts                     # Core migration engine
├── cli.ts                                 # Command-line interface
├── 001_add_migration_tracking.migration.ts  # Migration tracking table
├── 002_enhance_user_profile.migration.ts    # User profile enhancements
└── [timestamp]_[name].migration.ts          # Your migrations

backend/src/services/
├── MigrationService.ts                    # Service layer integration
└── StartupHealthCheck.ts                  # Health check integration
```

## Environment Variables

```bash
# Migration configuration
AUTO_MIGRATE=false                    # Enable auto-migration
MIGRATION_REQUIRE_CONFIRMATION=true   # Require confirmation for risky ops
MAX_MIGRATION_TIME=300000             # Max migration time (5 minutes)

# Database configuration
DB_PATH=./hesocial.duckdb            # Database file path
BACKUP_ENABLED=true                   # Enable backup integration
```

## Monitoring

### Health Endpoint Response
```json
{
  "component": "Database Migrations",
  "status": "healthy",
  "message": "All migrations applied successfully",
  "details": {
    "currentVersion": 5,
    "totalMigrations": 5,
    "pendingMigrations": 0,
    "lastMigration": {
      "id": "005_add_audit_logging",
      "applied_at": "2024-01-15T10:30:00Z"
    }
  }
}
```

### CLI Status Output
```
============================================================
📊 MIGRATION STATUS
============================================================
Current Version: 5
Applied Migrations: 5
Pending Migrations: 0
Total Migrations: 5
Last Migration: Add Audit Logging (2024-01-15T10:30:00Z)

✅ Database is up to date.
============================================================
```

This migration system provides enterprise-grade database change management with safety, reliability, and comprehensive tracking for production environments.