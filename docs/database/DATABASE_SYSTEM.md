# Database System

## Database Architecture

### Primary Database
- **Engine**: DuckDB with schema defined in `database/duckdb-schema.sql`
- **Production Persistence**: Cloudflare R2 backup system (COMPLETED - Production Ready)
- **Development**: Local DuckDB file for development

### Database Options
- **Migration System**: (COMPLETED - Production Ready)
  - Comprehensive migration framework with version control
  - Rollback capabilities with safety checks
  - CLI tools for migration management
  - Automatic migration validation and integrity checks
  - Smart restore logic with timestamp comparison
- **Production Persistence**: Cloudflare R2 backup system (COMPLETED - Production Ready)
  - Automatic backup on graceful shutdown with SIGTERM/SIGINT handling
  - Smart restore logic with timestamp comparison to prevent data loss
  - Manual backup API endpoints (`POST /api/admin/backup`, `GET /api/admin/backups`, `POST /api/admin/restore`)
  - Health monitoring endpoints (`/api/health/database`, `/api/health/r2-sync`, `/api/health/full`)
  - Environment separation (dev: `hesocial-duckdb-dev`, prod: `hesocial-duckdb`)
  - AWS SDK v3 integration with comprehensive error handling

### Key Tables
- **Users**: Multi-tier membership system (Platinum, Diamond, Black Card)
- **Events**: Complete event information with approval workflows
- **Venues**: Luxury venue database with capacity and amenities
- **Event Categories**: Pre-configured luxury event types
- **Registrations**: Member participation tracking
- **Financial verification**: Automated tier assignment
- **Server state tracking**: Operational metrics
- **Visitor tracking**: Anonymous visitor analytics and conversion tracking (visitor_sessions, visitor_page_views, visitor_events)

## Enhanced Server Features

- **9-Step Startup Sequence**: Configuration validation → Service initialization → Smart restore → Database connection → Migration checks → Health validation → Route loading → Server start
- **Health Monitoring**: Comprehensive startup health checks with visual indicators
- **Server State Tracking**: Operational metrics including uptime, session duration, and startup counts
- **Smart Backup/Restore**: Automatic timestamp-based restoration with data loss prevention
- **Periodic Backups**: Configurable automatic backups with interval scheduling (disabled by default, manual preferred)
- **Graceful Shutdown**: Clean shutdown with automatic backup creation and state recording

## Database Migration System

### Migration Framework (Production Ready)
A comprehensive database migration system that provides safe schema changes and deployments.

### Core Features
- **Version Control**: Each migration has unique ID and sequential versioning
- **Dependency Tracking**: Migrations can specify dependencies on other migrations
- **Rollback Capabilities**: Safe reversible operations with down() methods
- **Transaction Safety**: Atomic execution with comprehensive error handling
- **Integrity Validation**: Checksums and validation to detect modified migrations
- **Risk Assessment**: Identifies potentially dangerous operations before execution

### CLI Commands
```bash
# Migration status and management
npm run migrate:status          # Show current migration status
npm run migrate:up              # Apply all pending migrations
npm run migrate:rollback <ver>  # Rollback to specific version
npm run migrate:create          # Generate new migration template
npm run migrate:validate        # Validate migration integrity

# Advanced options
npm run migrate:rollback <ver> --force    # Force rollback (ignoring risks)
npm run migrate:up --dry-run              # Preview without executing
```

### Migration File Structure
```typescript
// Example: 001_add_user_preferences.migration.ts
import { BaseMigration } from '../Migration.js';

export default class AddUserPreferences extends BaseMigration {
  id = '001_add_user_preferences';
  version = 1;
  name = 'Add User Preferences';
  description = 'Add user preferences table and columns';
  category = 'schema' as const;

  async up(): Promise<void> {
    await this.executeSQL(`
      CREATE TABLE user_preferences (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        preference_key VARCHAR(100) NOT NULL,
        preference_value TEXT
      );
    `);
  }

  async down(): Promise<void> {
    await this.executeSQL(`DROP TABLE user_preferences;`);
  }
}
```

### Health Check Integration
- **Startup Validation**: Checks migration status during server startup
- **Auto-Migration**: Optional automatic migration execution (configurable)
- **Health Endpoints**: Migration status included in health check responses
- **Error Prevention**: Prevents startup with critical migration issues

### Safety Features
- **Backup Integration**: Automatic backups before risky operations
- **Production Checks**: Enhanced safety validations for production environments
- **Dependency Resolution**: Automatic validation of migration dependencies
- **Rollback Planning**: Safe rollback validation with risk assessment
- **Dry Run Mode**: Preview changes without executing them

## Database Schema Overview

### User Management
- Role-based access control (user, admin, super_admin)
- Default admin accounts for system administration
- Comprehensive test users for development and testing
- Financial verification and automatic tier assignment

### Event Content Management System ✅
Complete luxury event lifecycle management:
- **Events Table**: Complete event information with approval workflows and membership-based pricing
- **Venues Table**: Luxury venue database with capacity, amenities, and location data
- **Event Categories Table**: Pre-configured luxury event types with membership tier targeting
- **Event Registrations**: Member participation tracking with payment and status management
- **Event Feedback**: Rating and review system for event quality control
- **Event Waitlist**: Queue management for popular events

### System Operations
- **Registration System**: Event registration with approval workflows
- **Migration System**: Version-controlled schema changes with rollback capabilities
- **Operational Tracking**: Server state, migration history, and audit trails

## Related Documentation
- [Database Migrations](../DATABASE_MIGRATIONS.md)
- [R2 Backup Implementation](../R2_BACKUP_IMPLEMENTATION.md)
- [Configuration Guide](../configuration/R2_CONFIGURATION.md)
- [System Health](../systems/SYSTEM_HEALTH_DASHBOARD.md)