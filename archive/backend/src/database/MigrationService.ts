// Migration Service for Blue-Green Deployments
// Handles schema migrations with zero-downtime deployment

import { BlueGreenDatabaseManager, MigrationPlan, DeploymentResult } from './BlueGreenDatabaseManager.js';
import logger from '../utils/logger.js';
import fs from 'fs/promises';
import path from 'path';

interface Migration {
  version: number;
  name: string;
  description: string;
  sql: string;
  rollbackSql?: string;
  riskLevel: 'low' | 'medium' | 'high';
  estimatedDuration: number; // in milliseconds
}

interface SchemaVersion {
  version: number;
  appliedAt: Date;
  migrations: string[];
}

export class MigrationService {
  private migrationsPath: string;
  private currentSchemaVersion = 1; // Base schema version

  constructor(
    private blueGreenManager: BlueGreenDatabaseManager,
    migrationsPath: string = '/home/yanggf/a/hesocial/database/migrations'
  ) {
    this.migrationsPath = migrationsPath;
  }

  /**
   * Create migration plan for visitor tracking tables
   */
  async createVisitorTrackingMigrationPlan(): Promise<MigrationPlan> {
    const migration: Migration = {
      version: 2,
      name: 'add_visitor_tracking_tables',
      description: 'Add visitor tracking tables for analytics',
      sql: `
        -- Visitor sessions table - tracks unique visitors and their sessions
        CREATE TABLE IF NOT EXISTS visitor_sessions (
          id INTEGER PRIMARY KEY,
          visitor_id VARCHAR(50) UNIQUE NOT NULL,
          user_id INTEGER NULL, -- Links to users table when visitor converts
          ip_address VARCHAR(45) NOT NULL,
          user_agent TEXT NOT NULL,
          referer TEXT NULL,
          first_seen TIMESTAMP NOT NULL,
          last_seen TIMESTAMP NOT NULL,
          page_views INTEGER DEFAULT 1,
          session_count INTEGER DEFAULT 1,
          converted_at TIMESTAMP NULL, -- When visitor became registered user
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
          updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Individual page views table - detailed tracking for analytics
        CREATE TABLE IF NOT EXISTS visitor_page_views (
          id INTEGER PRIMARY KEY,
          visitor_id VARCHAR(50) NOT NULL,
          path VARCHAR(500) NOT NULL,
          method VARCHAR(10) NOT NULL DEFAULT 'GET',
          query_params TEXT NULL,
          referer TEXT NULL,
          timestamp TIMESTAMP NOT NULL,
          ip_address VARCHAR(45) NOT NULL,
          user_agent TEXT NOT NULL,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Visitor events table - track specific actions (future use)
        CREATE TABLE IF NOT EXISTS visitor_events (
          id INTEGER PRIMARY KEY,
          visitor_id VARCHAR(50) NOT NULL,
          event_type VARCHAR(50) NOT NULL, -- 'page_view', 'event_view', 'registration_attempt', etc.
          event_data TEXT NULL, -- JSON data for event details
          timestamp TIMESTAMP NOT NULL,
          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        -- Indexes for performance
        CREATE INDEX IF NOT EXISTS idx_visitor_sessions_visitor_id ON visitor_sessions(visitor_id);
        CREATE INDEX IF NOT EXISTS idx_visitor_sessions_user_id ON visitor_sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_visitor_sessions_last_seen ON visitor_sessions(last_seen);
        CREATE INDEX IF NOT EXISTS idx_visitor_page_views_visitor_id ON visitor_page_views(visitor_id);
        CREATE INDEX IF NOT EXISTS idx_visitor_page_views_timestamp ON visitor_page_views(timestamp);
        CREATE INDEX IF NOT EXISTS idx_visitor_page_views_path ON visitor_page_views(path);
        CREATE INDEX IF NOT EXISTS idx_visitor_events_visitor_id ON visitor_events(visitor_id);
        CREATE INDEX IF NOT EXISTS idx_visitor_events_type ON visitor_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_visitor_events_timestamp ON visitor_events(timestamp);
      `,
      rollbackSql: `
        DROP TABLE IF EXISTS visitor_events;
        DROP TABLE IF EXISTS visitor_page_views;
        DROP TABLE IF EXISTS visitor_sessions;
      `,
      riskLevel: 'low',
      estimatedDuration: 5000 // 5 seconds
    };

    return {
      targetSchemaVersion: 2,
      migrations: [migration],
      estimatedDuration: migration.estimatedDuration,
      riskLevel: migration.riskLevel
    };
  }

  /**
   * Deploy visitor tracking tables with zero downtime
   */
  async deployVisitorTracking(): Promise<DeploymentResult> {
    try {
      logger.info('🚀 Starting visitor tracking deployment...');

      const migrationPlan = await this.createVisitorTrackingMigrationPlan();
      
      logger.info('📋 Migration plan created', {
        targetVersion: migrationPlan.targetSchemaVersion,
        migrations: migrationPlan.migrations.length,
        estimatedDuration: `${migrationPlan.estimatedDuration}ms`,
        riskLevel: migrationPlan.riskLevel
      });

      // Execute blue-green deployment
      const result = await this.blueGreenManager.deploySchema(migrationPlan);

      if (result.success) {
        logger.info('✅ Visitor tracking deployment successful', {
          activeEnvironment: result.activeEnvironment,
          schemaVersion: result.schemaVersion,
          duration: `${result.duration}ms`
        });
      } else {
        logger.error('❌ Visitor tracking deployment failed', {
          error: result.error,
          duration: `${result.duration}ms`
        });
      }

      return result;
    } catch (error) {
      logger.error('Failed to deploy visitor tracking', { error });
      throw error;
    }
  }

  /**
   * Create migration plan for analytics query fixes
   */
  async createAnalyticsFixMigrationPlan(): Promise<MigrationPlan> {
    const migration: Migration = {
      version: 3,
      name: 'fix_analytics_queries',
      description: 'Fix analytics queries for DuckDB compatibility',
      sql: `
        -- Create a view to help with analytics queries using proper DuckDB syntax
        CREATE OR REPLACE VIEW analytics_events AS
        SELECT 
          e.*,
          ec.name as category_name,
          v.name as venue_name,
          CAST(json_extract(e.pricing, '$.platinum') AS INTEGER) as platinum_price,
          CAST(json_extract(e.pricing, '$.diamond') AS INTEGER) as diamond_price,
          CAST(json_extract(e.pricing, '$.black_card') AS INTEGER) as black_card_price
        FROM events e
        JOIN event_categories ec ON e.category_id = ec.id
        JOIN venues v ON e.venue_id = v.id;

        -- Create analytics helper view for member engagement
        CREATE OR REPLACE VIEW member_engagement AS
        SELECT 
          u.id as user_id,
          u.membership_tier,
          u.created_at as member_since,
          COUNT(r.id) as total_registrations,
          COUNT(CASE WHEN r.created_at >= CURRENT_TIMESTAMP - INTERVAL '30 days' THEN 1 END) as recent_registrations,
          MAX(r.created_at) as last_registration
        FROM users u
        LEFT JOIN registrations r ON u.id = r.user_id AND r.status = 'confirmed'
        GROUP BY u.id, u.membership_tier, u.created_at;
      `,
      rollbackSql: `
        DROP VIEW IF EXISTS member_engagement;
        DROP VIEW IF EXISTS analytics_events;
      `,
      riskLevel: 'low',
      estimatedDuration: 2000 // 2 seconds
    };

    return {
      targetSchemaVersion: 3,
      migrations: [migration],
      estimatedDuration: migration.estimatedDuration,
      riskLevel: migration.riskLevel
    };
  }

  /**
   * Get current system status including deployment state
   */
  async getSystemStatus(): Promise<{
    schemaVersion: number;
    blueGreenStatus: any;
    migrationsAvailable: string[];
    canDeploy: boolean;
  }> {
    const blueGreenStatus = await this.blueGreenManager.getSystemHealth();
    const migrationsAvailable = await this.getAvailableMigrations();

    return {
      schemaVersion: this.currentSchemaVersion,
      blueGreenStatus,
      migrationsAvailable,
      canDeploy: !blueGreenStatus.deploymentInProgress
    };
  }

  /**
   * Perform instant rollback if available
   */
  async rollback(): Promise<DeploymentResult> {
    logger.warn('🔄 Initiating rollback...');
    return await this.blueGreenManager.rollback();
  }

  /**
   * Get health status of all database environments
   */
  async getHealthStatus() {
    return await this.blueGreenManager.getSystemHealth();
  }

  /**
   * Emergency: Apply schema directly to active database (bypass blue-green)
   * Only use when blue-green system is not available
   */
  async emergencyApplyVisitorTracking(): Promise<void> {
    logger.warn('⚠️ Emergency schema application - bypassing blue-green deployment');
    
    const activePool = this.blueGreenManager.getActivePool();
    const migrationPlan = await this.createVisitorTrackingMigrationPlan();
    
    for (const migration of migrationPlan.migrations) {
      try {
        await activePool.query(migration.sql);
        logger.info('Emergency migration applied', { name: migration.name });
      } catch (error) {
        logger.error('Emergency migration failed', { name: migration.name, error });
        throw error;
      }
    }
    
    logger.info('✅ Emergency schema application completed');
  }

  // Private methods

  private async getAvailableMigrations(): Promise<string[]> {
    try {
      const files = await fs.readdir(this.migrationsPath);
      return files.filter(file => file.endsWith('.sql')).sort();
    } catch {
      return [];
    }
  }

  private async loadMigrationFile(filename: string): Promise<string> {
    const filePath = path.join(this.migrationsPath, filename);
    return await fs.readFile(filePath, 'utf8');
  }
}

// Export singleton instance
export const migrationService = new MigrationService(
  // We'll inject the blueGreenManager after it's initialized
  {} as BlueGreenDatabaseManager
);