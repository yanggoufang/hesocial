import { MigrationRunner } from '../database/migrations/MigrationRunner.js';
import logger from '../utils/logger.js';

export interface MigrationServiceConfig {
  autoMigrate: boolean;
  requireConfirmation: boolean;
  maxMigrationTime: number; // in milliseconds
}

export class MigrationService {
  private runner: MigrationRunner;
  private config: MigrationServiceConfig;

  constructor(config?: Partial<MigrationServiceConfig>) {
    this.runner = new MigrationRunner();
    this.config = {
      autoMigrate: process.env.AUTO_MIGRATE === 'true',
      requireConfirmation: process.env.MIGRATION_REQUIRE_CONFIRMATION === 'true',
      maxMigrationTime: parseInt(process.env.MAX_MIGRATION_TIME || '300000'), // 5 minutes default
      ...config
    };
  }

  /**
   * Initialize migration service
   */
  async initialize(): Promise<void> {
    try {
      await this.runner.initialize();
      logger.info('Migration service initialized');
    } catch (error) {
      logger.error('Failed to initialize migration service', {
        error: error instanceof Error ? error.message : error
      });
      throw error;
    }
  }

  /**
   * Check migration status and optionally auto-migrate
   */
  async checkAndMigrate(): Promise<{
    status: 'up-to-date' | 'migrated' | 'pending' | 'failed';
    currentVersion: number;
    pendingCount: number;
    message: string;
  }> {
    try {
      const status = await this.runner.getStatus();
      
      if (status.pendingMigrations === 0) {
        logger.info('Database is up to date', {
          currentVersion: status.currentVersion,
          appliedMigrations: status.appliedMigrations
        });
        
        return {
          status: 'up-to-date',
          currentVersion: status.currentVersion,
          pendingCount: 0,
          message: 'Database schema is up to date'
        };
      }

      logger.info('Pending migrations detected', {
        pendingCount: status.pendingMigrations,
        currentVersion: status.currentVersion
      });

      if (!this.config.autoMigrate) {
        logger.warn('Auto-migration disabled - manual migration required');
        return {
          status: 'pending',
          currentVersion: status.currentVersion,
          pendingCount: status.pendingMigrations,
          message: `${status.pendingMigrations} pending migrations require manual execution`
        };
      }

      // Validate migrations before auto-migration
      const validation = await this.runner.validateIntegrity();
      if (!validation.valid) {
        logger.error('Migration validation failed', { issues: validation.issues });
        throw new Error(`Migration validation failed: ${validation.issues.join(', ')}`);
      }

      // Execute auto-migration
      const result = await this.executeAutoMigration();
      
      return {
        status: result.success ? 'migrated' : 'failed',
        currentVersion: result.finalVersion,
        pendingCount: result.remainingPending,
        message: result.message
      };

    } catch (error) {
      logger.error('Migration check failed', {
        error: error instanceof Error ? error.message : error
      });
      
      return {
        status: 'failed',
        currentVersion: this.runner.getCurrentVersion(),
        pendingCount: -1,
        message: `Migration check failed: ${error instanceof Error ? error.message : 'Unknown error'}`
      };
    }
  }

  /**
   * Execute auto-migration with safety checks
   */
  private async executeAutoMigration(): Promise<{
    success: boolean;
    finalVersion: number;
    remainingPending: number;
    message: string;
    executedMigrations?: number;
  }> {
    const startTime = Date.now();
    
    try {
      const plan = await this.runner.createMigrationPlan();
      
      // Safety checks
      if (plan.dependencies.length > 0) {
        throw new Error(`Migration dependencies not satisfied: ${plan.dependencies.join(', ')}`);
      }

      if (plan.estimatedTime > this.config.maxMigrationTime) {
        throw new Error(`Estimated migration time (${plan.estimatedTime}ms) exceeds maximum allowed (${this.config.maxMigrationTime}ms)`);
      }

      logger.info('Starting auto-migration', {
        migrationCount: plan.totalMigrations,
        estimatedTime: `${Math.round(plan.estimatedTime / 1000)}s`,
        canRollback: plan.canRollback
      });

      // Execute migrations
      const results = await this.runner.migrate();
      const successful = results.filter(r => r.success).length;
      const failed = results.length - successful;
      
      const executionTime = Date.now() - startTime;
      
      if (failed > 0) {
        logger.error('Auto-migration partially failed', {
          successful,
          failed,
          executionTime: `${executionTime}ms`
        });
        
        return {
          success: false,
          finalVersion: this.runner.getCurrentVersion(),
          remainingPending: failed,
          message: `${failed} migrations failed during auto-migration`,
          executedMigrations: successful
        };
      }

      logger.info('Auto-migration completed successfully', {
        migrationsExecuted: successful,
        executionTime: `${executionTime}ms`,
        finalVersion: this.runner.getCurrentVersion()
      });

      return {
        success: true,
        finalVersion: this.runner.getCurrentVersion(),
        remainingPending: 0,
        message: `Successfully executed ${successful} migrations`,
        executedMigrations: successful
      };

    } catch (error) {
      const executionTime = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      logger.error('Auto-migration failed', {
        error: errorMessage,
        executionTime: `${executionTime}ms`
      });

      return {
        success: false,
        finalVersion: this.runner.getCurrentVersion(),
        remainingPending: -1,
        message: `Auto-migration failed: ${errorMessage}`
      };
    }
  }

  /**
   * Get migration runner for manual operations
   */
  getRunner(): MigrationRunner {
    return this.runner;
  }

  /**
   * Get detailed migration status
   */
  async getDetailedStatus(): Promise<{
    currentVersion: number;
    appliedMigrations: number;
    pendingMigrations: number;
    totalMigrations: number;
    lastMigration?: any;
    validationStatus: { valid: boolean; issues: string[] };
    config: MigrationServiceConfig;
  }> {
    const status = await this.runner.getStatus();
    const validation = await this.runner.validateIntegrity();

    return {
      ...status,
      validationStatus: validation,
      config: this.config
    };
  }

  /**
   * Create migration health check result
   */
  async createHealthCheckResult(): Promise<{
    component: string;
    status: 'healthy' | 'warning' | 'error';
    message: string;
    details?: any;
  }> {
    try {
      const status = await this.runner.getStatus();
      const validation = await this.runner.validateIntegrity();

      if (!validation.valid) {
        return {
          component: 'Database Migrations',
          status: 'error',
          message: `Migration validation failed: ${validation.issues.length} issues`,
          details: { issues: validation.issues, currentVersion: status.currentVersion }
        };
      }

      if (status.pendingMigrations > 0) {
        return {
          component: 'Database Migrations',
          status: 'warning',
          message: `${status.pendingMigrations} pending migrations`,
          details: { 
            pendingCount: status.pendingMigrations,
            currentVersion: status.currentVersion,
            autoMigrate: this.config.autoMigrate
          }
        };
      }

      return {
        component: 'Database Migrations',
        status: 'healthy',
        message: 'All migrations applied successfully',
        details: {
          currentVersion: status.currentVersion,
          totalMigrations: status.totalMigrations
        }
      };

    } catch (error) {
      return {
        component: 'Database Migrations',
        status: 'error',
        message: `Migration system error: ${error instanceof Error ? error.message : 'Unknown error'}`,
        details: { error }
      };
    }
  }
}