// Blue-Green Database Deployment Manager
// Provides zero-downtime schema migrations with atomic database switching

import { EnhancedDuckDBPool } from './duckdb-pool.js';
import { duckdb as baseDuckDB } from './duckdb-connection.js';
import logger from '../utils/logger.js';
import fs from 'fs/promises';
import path from 'path';

export interface DatabaseEnvironment {
  name: 'blue' | 'green';
  path: string;
  pool: EnhancedDuckDBPool;
  isActive: boolean;
  schemaVersion: number;
  createdAt: Date;
  healthStatus: 'healthy' | 'unhealthy' | 'unknown';
}

export interface MigrationPlan {
  targetSchemaVersion: number;
  migrations: Array<{
    version: number;
    name: string;
    sql: string;
    rollbackSql?: string;
  }>;
  estimatedDuration: number;
  riskLevel: 'low' | 'medium' | 'high';
}

export interface DeploymentResult {
  success: boolean;
  activeEnvironment: 'blue' | 'green';
  previousEnvironment: 'blue' | 'green';
  schemaVersion: number;
  duration: number;
  rollbackAvailable: boolean;
  error?: string;
}

export class BlueGreenDatabaseManager {
  private blue: DatabaseEnvironment | null = null;
  private green: DatabaseEnvironment | null = null;
  private activeEnvironment: 'blue' | 'green' = 'blue';
  private isDeploymentInProgress = false;
  private rollbackTimeout = 5 * 60 * 1000; // 5 minutes rollback window
  private rollbackTimer: NodeJS.Timeout | null = null;

  constructor(
    private baseDbPath: string = '/home/yanggf/a/hesocial/hesocial.duckdb',
    private schemaPath: string = '/home/yanggf/a/hesocial/database/duckdb-schema.sql'
  ) {}

  /**
   * Initialize the blue-green system with current database
   */
  async initialize(): Promise<void> {
    try {
      logger.info('🔵 Initializing Blue-Green Database Manager...');
      
      // Initialize blue environment with current database
      await this.initializeBlueEnvironment();
      
      // Set blue as active
      this.activeEnvironment = 'blue';
      
      logger.info('✅ Blue-Green Database Manager initialized', {
        activeEnvironment: this.activeEnvironment,
        blueSchemaVersion: this.blue?.schemaVersion
      });
    } catch (error) {
      logger.error('Failed to initialize Blue-Green Database Manager', { error });
      throw error;
    }
  }

  /**
   * Get current active database pool
   */
  getActivePool(): EnhancedDuckDBPool {
    const activeEnv = this.getActiveEnvironment();
    if (!activeEnv) {
      throw new Error('No active database environment available');
    }
    return activeEnv.pool;
  }

  /**
   * Deploy new schema with zero downtime
   */
  async deploySchema(migrationPlan: MigrationPlan): Promise<DeploymentResult> {
    if (this.isDeploymentInProgress) {
      throw new Error('Deployment already in progress');
    }

    this.isDeploymentInProgress = true;
    const deploymentStart = Date.now();
    const previousEnv = this.activeEnvironment;
    const targetEnv = this.activeEnvironment === 'blue' ? 'green' : 'blue';

    try {
      logger.info('🚀 Starting Blue-Green deployment', {
        from: previousEnv,
        to: targetEnv,
        targetSchemaVersion: migrationPlan.targetSchemaVersion,
        estimatedDuration: migrationPlan.estimatedDuration
      });

      // Step 1: Prepare target environment
      await this.prepareTargetEnvironment(targetEnv);

      // Step 2: Copy data from active to target
      await this.copyDatabaseData(previousEnv, targetEnv);

      // Step 3: Apply migrations to target
      await this.applyMigrations(targetEnv, migrationPlan);

      // Step 4: Validate target environment
      await this.validateEnvironment(targetEnv);

      // Step 5: Atomic switch
      await this.switchActiveEnvironment(targetEnv);

      // Step 6: Schedule rollback window
      this.scheduleRollbackWindow(previousEnv);

      const duration = Date.now() - deploymentStart;

      logger.info('✅ Blue-Green deployment successful', {
        activeEnvironment: this.activeEnvironment,
        previousEnvironment: previousEnv,
        duration: `${duration}ms`,
        schemaVersion: migrationPlan.targetSchemaVersion
      });

      return {
        success: true,
        activeEnvironment: this.activeEnvironment,
        previousEnvironment: previousEnv,
        schemaVersion: migrationPlan.targetSchemaVersion,
        duration,
        rollbackAvailable: true
      };

    } catch (error) {
      logger.error('❌ Blue-Green deployment failed', { error });
      
      // Cleanup failed environment
      await this.cleanupEnvironment(targetEnv);
      
      return {
        success: false,
        activeEnvironment: this.activeEnvironment,
        previousEnvironment: previousEnv,
        schemaVersion: this.getActiveEnvironment()?.schemaVersion || 0,
        duration: Date.now() - deploymentStart,
        rollbackAvailable: false,
        error: error instanceof Error ? error.message : String(error)
      };
    } finally {
      this.isDeploymentInProgress = false;
    }
  }

  /**
   * Instant rollback to previous environment
   */
  async rollback(): Promise<DeploymentResult> {
    const currentEnv = this.activeEnvironment;
    const rollbackEnv = currentEnv === 'blue' ? 'green' : 'blue';
    const rollbackTarget = this.getEnvironment(rollbackEnv);

    if (!rollbackTarget || rollbackTarget.healthStatus !== 'healthy') {
      throw new Error(`Rollback environment ${rollbackEnv} not available or unhealthy`);
    }

    const rollbackStart = Date.now();

    try {
      logger.warn('🔄 Performing instant rollback', {
        from: currentEnv,
        to: rollbackEnv,
        rollbackSchemaVersion: rollbackTarget.schemaVersion
      });

      // Cancel rollback timer
      if (this.rollbackTimer) {
        clearTimeout(this.rollbackTimer);
        this.rollbackTimer = null;
      }

      // Instant switch
      await this.switchActiveEnvironment(rollbackEnv);

      const duration = Date.now() - rollbackStart;

      logger.info('✅ Rollback completed', {
        activeEnvironment: this.activeEnvironment,
        previousEnvironment: currentEnv,
        duration: `${duration}ms`
      });

      return {
        success: true,
        activeEnvironment: this.activeEnvironment,
        previousEnvironment: currentEnv,
        schemaVersion: rollbackTarget.schemaVersion,
        duration,
        rollbackAvailable: false
      };

    } catch (error) {
      logger.error('❌ Rollback failed', { error });
      throw error;
    }
  }

  /**
   * Get health status of both environments
   */
  async getSystemHealth(): Promise<{
    blue: DatabaseEnvironment | null;
    green: DatabaseEnvironment | null;
    active: 'blue' | 'green';
    rollbackAvailable: boolean;
    deploymentInProgress: boolean;
  }> {
    // Update health status
    if (this.blue) {
      this.blue.healthStatus = await this.checkEnvironmentHealth(this.blue);
    }
    if (this.green) {
      this.green.healthStatus = await this.checkEnvironmentHealth(this.green);
    }

    return {
      blue: this.blue,
      green: this.green,
      active: this.activeEnvironment,
      rollbackAvailable: this.rollbackTimer !== null,
      deploymentInProgress: this.isDeploymentInProgress
    };
  }

  // Private methods

  private async initializeBlueEnvironment(): Promise<void> {
    const bluePath = `${this.baseDbPath}.blue`;
    
    // Copy current database to blue if it doesn't exist
    if (!(await this.fileExists(bluePath))) {
      await this.copyFile(this.baseDbPath, bluePath);
    }

    this.blue = {
      name: 'blue',
      path: bluePath,
      pool: new EnhancedDuckDBPool(),
      isActive: true,
      schemaVersion: await this.detectSchemaVersion(bluePath),
      createdAt: new Date(),
      healthStatus: 'unknown'
    };

    // Initialize connection to blue database
    await this.initializeEnvironmentConnection(this.blue);
  }

  private async prepareTargetEnvironment(target: 'blue' | 'green'): Promise<void> {
    const targetPath = `${this.baseDbPath}.${target}`;
    
    // Clean up existing target environment
    await this.cleanupEnvironment(target);
    
    // Create new environment
    const env: DatabaseEnvironment = {
      name: target,
      path: targetPath,
      pool: new EnhancedDuckDBPool(),
      isActive: false,
      schemaVersion: 0,
      createdAt: new Date(),
      healthStatus: 'unknown'
    };

    if (target === 'blue') {
      this.blue = env;
    } else {
      this.green = env;
    }

    // Initialize with base schema
    await this.initializeEnvironmentConnection(env);
    await this.createBaseSchema(env);
  }

  private async copyDatabaseData(source: 'blue' | 'green', target: 'blue' | 'green'): Promise<void> {
    const sourceEnv = this.getEnvironment(source);
    const targetEnv = this.getEnvironment(target);

    if (!sourceEnv || !targetEnv) {
      throw new Error(`Environment not available: source=${source}, target=${target}`);
    }

    logger.info('📋 Copying database data', { from: source, to: target });

    // Export data from source and import to target
    // For DuckDB, we can use EXPORT/IMPORT or direct table copying
    await this.copyAllTables(sourceEnv, targetEnv);
  }

  private async applyMigrations(target: 'blue' | 'green', plan: MigrationPlan): Promise<void> {
    const targetEnv = this.getEnvironment(target);
    if (!targetEnv) {
      throw new Error(`Target environment ${target} not available`);
    }

    logger.info('🔧 Applying migrations', { 
      target, 
      migrations: plan.migrations.length,
      targetVersion: plan.targetSchemaVersion 
    });

    for (const migration of plan.migrations) {
      try {
        await targetEnv.pool.query(migration.sql);
        logger.debug('Migration applied', { version: migration.version, name: migration.name });
      } catch (error) {
        logger.error('Migration failed', { 
          version: migration.version, 
          name: migration.name, 
          error 
        });
        throw error;
      }
    }

    targetEnv.schemaVersion = plan.targetSchemaVersion;
  }

  private async validateEnvironment(target: 'blue' | 'green'): Promise<void> {
    const targetEnv = this.getEnvironment(target);
    if (!targetEnv) {
      throw new Error(`Target environment ${target} not available`);
    }

    logger.info('🔍 Validating environment', { target });

    // Run validation queries
    const validations = [
      'SELECT COUNT(*) FROM users',
      'SELECT COUNT(*) FROM events',
      'SELECT COUNT(*) FROM registrations',
      'SELECT COUNT(*) FROM visitor_sessions'
    ];

    for (const sql of validations) {
      try {
        await targetEnv.pool.query(sql);
      } catch (error) {
        logger.error('Environment validation failed', { target, sql, error });
        throw new Error(`Environment validation failed: ${error}`);
      }
    }

    targetEnv.healthStatus = 'healthy';
    logger.info('✅ Environment validation passed', { target });
  }

  private async switchActiveEnvironment(target: 'blue' | 'green'): Promise<void> {
    logger.info('🔀 Switching active environment', { 
      from: this.activeEnvironment, 
      to: target 
    });

    // Atomic switch
    this.activeEnvironment = target;
    
    const activeEnv = this.getEnvironment(target);
    if (activeEnv) {
      activeEnv.isActive = true;
    }

    const inactiveEnv = this.getEnvironment(target === 'blue' ? 'green' : 'blue');
    if (inactiveEnv) {
      inactiveEnv.isActive = false;
    }

    logger.info('✅ Environment switch completed', { activeEnvironment: this.activeEnvironment });
  }

  private scheduleRollbackWindow(previousEnv: 'blue' | 'green'): void {
    logger.info('⏰ Scheduling rollback window', { 
      duration: `${this.rollbackTimeout / 1000}s`,
      previousEnvironment: previousEnv 
    });

    this.rollbackTimer = setTimeout(async () => {
      logger.info('🧹 Rollback window expired, cleaning up previous environment', { 
        previousEnvironment: previousEnv 
      });
      
      await this.cleanupEnvironment(previousEnv);
      this.rollbackTimer = null;
    }, this.rollbackTimeout);
  }

  private async cleanupEnvironment(env: 'blue' | 'green'): Promise<void> {
    const environment = this.getEnvironment(env);
    if (!environment) return;

    try {
      await environment.pool.end();
      await this.deleteFile(environment.path);
      
      if (env === 'blue') {
        this.blue = null;
      } else {
        this.green = null;
      }

      logger.info('🧹 Environment cleaned up', { environment: env });
    } catch (error) {
      logger.warn('Failed to cleanup environment', { environment: env, error });
    }
  }

  private getActiveEnvironment(): DatabaseEnvironment | null {
    return this.activeEnvironment === 'blue' ? this.blue : this.green;
  }

  private getEnvironment(env: 'blue' | 'green'): DatabaseEnvironment | null {
    return env === 'blue' ? this.blue : this.green;
  }

  private async checkEnvironmentHealth(env: DatabaseEnvironment): Promise<'healthy' | 'unhealthy'> {
    try {
      await env.pool.query('SELECT 1');
      return 'healthy';
    } catch {
      return 'unhealthy';
    }
  }

  private async detectSchemaVersion(dbPath: string): Promise<number> {
    // TODO: Implement schema version detection
    // For now, return 1 as base version
    return 1;
  }

  private async initializeEnvironmentConnection(env: DatabaseEnvironment): Promise<void> {
    // TODO: Initialize DuckDB connection for specific database file
    // This requires extending the DuckDB connection to support multiple files
  }

  private async createBaseSchema(env: DatabaseEnvironment): Promise<void> {
    const schemaContent = await fs.readFile(this.schemaPath, 'utf8');
    await env.pool.query(schemaContent);
  }

  private async copyAllTables(source: DatabaseEnvironment, target: DatabaseEnvironment): Promise<void> {
    // TODO: Implement table copying between DuckDB instances
    // This could use DuckDB's EXPORT/IMPORT functionality
  }

  private async fileExists(path: string): Promise<boolean> {
    try {
      await fs.access(path);
      return true;
    } catch {
      return false;
    }
  }

  private async copyFile(source: string, target: string): Promise<void> {
    await fs.copyFile(source, target);
  }

  private async deleteFile(path: string): Promise<void> {
    try {
      await fs.unlink(path);
    } catch {
      // Ignore if file doesn't exist
    }
  }
}

// Export singleton instance
export const blueGreenManager = new BlueGreenDatabaseManager();