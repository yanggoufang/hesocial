import { Migration, MigrationResult, MigrationState, MigrationPlan, RollbackPlan } from './Migration.js';
import { duckdb } from '../duckdb-connection.js';
import logger from '../../utils/logger.js';
import { readdir } from 'fs/promises';
import { join } from 'path';

export class MigrationRunner {
  private db: any;
  private migrations: Map<string, Migration> = new Map();
  private migrationHistory: MigrationState[] = [];

  constructor() {
    this.db = duckdb;
  }

  /**
   * Initialize migration system
   */
  async initialize(): Promise<void> {
    try {
      await this.ensureMigrationTable();
      await this.loadMigrationHistory();
      await this.loadMigrations();
      logger.info('Migration system initialized', {
        migrationsLoaded: this.migrations.size,
        historyEntries: this.migrationHistory.length
      });
    } catch (error) {
      logger.error('Failed to initialize migration system', { 
        error: error instanceof Error ? error.message : error 
      });
      throw error;
    }
  }

  /**
   * Create migrations table if it doesn't exist
   */
  private async ensureMigrationTable(): Promise<void> {
    const sql = `
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
    `;

    await this.db.query(sql);
    logger.debug('Migration table ensured');
  }

  /**
   * Load migration history from database
   */
  private async loadMigrationHistory(): Promise<void> {
    const result = await this.db.query(`
      SELECT * FROM schema_migrations 
      ORDER BY version ASC
    `);

    this.migrationHistory = result.rows.map((row: any) => ({
      id: row.id,
      version: row.version,
      name: row.name,
      description: row.description,
      category: row.category,
      checksum: row.checksum,
      applied_at: new Date(row.applied_at),
      execution_time: row.execution_time,
      applied_by: row.applied_by,
      rollback_sql: row.rollback_sql
    }));

    logger.debug('Migration history loaded', { count: this.migrationHistory.length });
  }

  /**
   * Load migration files from directory
   */
  private async loadMigrations(): Promise<void> {
    try {
      const migrationsDir = join(process.cwd(), 'src/database/migrations');
      const files = await readdir(migrationsDir);
      
      const migrationFiles = files
        .filter(file => file.endsWith('.migration.ts') || file.endsWith('.migration.js'))
        .sort();

      for (const file of migrationFiles) {
        try {
          const modulePath = join(migrationsDir, file);
          const module = await import(modulePath);
          
          if (module.default && typeof module.default === 'function') {
            const migrationClass = module.default;
            const migration = new migrationClass(this);
            
            migration.validateMigration();
            this.migrations.set(migration.id, migration);
            
            logger.debug('Loaded migration', { 
              id: migration.id, 
              version: migration.version, 
              name: migration.name 
            });
          }
        } catch (error) {
          logger.error(`Failed to load migration ${file}`, { 
            error: error instanceof Error ? error.message : error 
          });
        }
      }
    } catch (error) {
      logger.warn('No migrations directory found or error loading migrations', { 
        error: error instanceof Error ? error.message : error 
      });
    }
  }

  /**
   * Create migration execution plan
   */
  async createMigrationPlan(): Promise<MigrationPlan> {
    const appliedVersions = new Set(this.migrationHistory.map(h => h.version));
    const pendingMigrations = Array.from(this.migrations.values())
      .filter(m => !appliedVersions.has(m.version))
      .sort((a, b) => a.version - b.version);

    // Check dependencies
    const dependencies: string[] = [];
    for (const migration of pendingMigrations) {
      if (migration.dependencies) {
        for (const dep of migration.dependencies) {
          if (!appliedVersions.has(parseInt(dep))) {
            dependencies.push(`Migration ${migration.id} depends on ${dep} which is not applied`);
          }
        }
      }
    }

    const estimatedTime = pendingMigrations.reduce((total, m) => total + (m.executionTime || 1000), 0);

    return {
      migrations: pendingMigrations,
      totalMigrations: pendingMigrations.length,
      estimatedTime,
      canRollback: pendingMigrations.every(m => typeof m.down === 'function'),
      dependencies
    };
  }

  /**
   * Execute pending migrations
   */
  async migrate(): Promise<MigrationResult[]> {
    const plan = await this.createMigrationPlan();
    
    if (plan.dependencies.length > 0) {
      throw new Error(`Migration dependencies not satisfied: ${plan.dependencies.join(', ')}`);
    }

    if (plan.migrations.length === 0) {
      logger.info('No pending migrations to execute');
      return [];
    }

    logger.info('Starting migration execution', {
      totalMigrations: plan.totalMigrations,
      estimatedTime: `${Math.round(plan.estimatedTime / 1000)}s`
    });

    const results: MigrationResult[] = [];

    for (const migration of plan.migrations) {
      const result = await this.executeMigration(migration);
      results.push(result);
      
      if (!result.success) {
        logger.error('Migration failed, stopping execution', { 
          failedMigration: migration.id,
          error: result.error 
        });
        break;
      }
    }

    const successful = results.filter(r => r.success).length;
    logger.info('Migration execution completed', {
      successful,
      total: results.length,
      failed: results.length - successful
    });

    return results;
  }

  /**
   * Execute single migration
   */
  private async executeMigration(migration: Migration): Promise<MigrationResult> {
    const startTime = Date.now();
    
    try {
      logger.info('Executing migration', { 
        id: migration.id, 
        version: migration.version, 
        name: migration.name 
      });

      // Create migration context with SQL execution methods
      const migrationContext = this.createMigrationContext(migration);
      
      // Execute the migration
      await migration.up.call(migrationContext);
      
      const executionTime = Date.now() - startTime;
      
      // Record successful migration
      await this.recordMigration(migration, executionTime);
      
      logger.info('Migration completed successfully', { 
        id: migration.id, 
        executionTime: `${executionTime}ms` 
      });

      return {
        id: migration.id,
        version: migration.version,
        success: true,
        executionTime,
        rollbackAvailable: typeof migration.down === 'function'
      };
    } catch (error) {
      const executionTime = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      logger.error('Migration failed', { 
        id: migration.id, 
        error: errorMessage,
        executionTime: `${executionTime}ms`
      });

      return {
        id: migration.id,
        version: migration.version,
        success: false,
        executionTime,
        error: errorMessage,
        rollbackAvailable: false
      };
    }
  }

  /**
   * Create migration context with SQL execution methods
   */
  private createMigrationContext(migration: Migration): any {
    return {
      ...migration,
      executeSQL: async (sql: string) => {
        logger.debug('Executing SQL', { migrationId: migration.id, sql: sql.substring(0, 100) + '...' });
        await this.db.query(sql);
      },
      executeSQLWithResult: async (sql: string) => {
        logger.debug('Executing SQL with result', { migrationId: migration.id, sql: sql.substring(0, 100) + '...' });
        return await this.db.query(sql);
      }
    };
  }

  /**
   * Record successful migration in database
   */
  private async recordMigration(migration: Migration, executionTime: number): Promise<void> {
    const checksum = migration.checksum || this.generateChecksum(migration);
    
    await this.db.query(`
      INSERT INTO schema_migrations (
        id, version, name, description, category, checksum, 
        execution_time, applied_by, can_rollback
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `, [
      migration.id,
      migration.version,
      migration.name,
      migration.description,
      migration.category || 'schema',
      checksum,
      executionTime,
      'system',
      typeof migration.down === 'function'
    ]);

    // Update local history
    this.migrationHistory.push({
      id: migration.id,
      version: migration.version,
      name: migration.name,
      description: migration.description,
      category: migration.category || 'schema',
      checksum,
      applied_at: new Date(),
      execution_time: executionTime,
      applied_by: 'system'
    });
  }

  /**
   * Create rollback plan
   */
  async createRollbackPlan(targetVersion: number): Promise<RollbackPlan> {
    const currentVersion = this.getCurrentVersion();
    
    if (targetVersion >= currentVersion) {
      throw new Error(`Target version ${targetVersion} must be less than current version ${currentVersion}`);
    }

    const migrationsToRollback = this.migrationHistory
      .filter(h => h.version > targetVersion)
      .reverse() // Rollback in reverse order
      .map(h => this.migrations.get(h.id))
      .filter((m): m is Migration => m !== undefined);

    const riskyOperations: string[] = [];
    
    // Check for risky rollback operations
    for (const migration of migrationsToRollback) {
      if (!migration.down) {
        riskyOperations.push(`Migration ${migration.id} has no rollback method`);
      }
      if (migration.category === 'data') {
        riskyOperations.push(`Migration ${migration.id} involves data changes - rollback may cause data loss`);
      }
    }

    const estimatedTime = migrationsToRollback.reduce((total, m) => total + (m.executionTime || 1000), 0);

    return {
      migrations: migrationsToRollback,
      targetVersion,
      rollbackSteps: migrationsToRollback.length,
      estimatedTime,
      riskyOperations
    };
  }

  /**
   * Execute rollback to target version
   */
  async rollback(targetVersion: number, force: boolean = false): Promise<MigrationResult[]> {
    const plan = await this.createRollbackPlan(targetVersion);
    
    if (plan.riskyOperations.length > 0 && !force) {
      throw new Error(`Rollback has risky operations: ${plan.riskyOperations.join(', ')}. Use force=true to proceed.`);
    }

    logger.warn('Starting rollback operation', {
      targetVersion,
      rollbackSteps: plan.rollbackSteps,
      riskyOperations: plan.riskyOperations
    });

    const results: MigrationResult[] = [];

    for (const migration of plan.migrations) {
      const result = await this.executeRollback(migration);
      results.push(result);
      
      if (!result.success) {
        logger.error('Rollback failed, stopping execution', { 
          failedMigration: migration.id,
          error: result.error 
        });
        break;
      }
    }

    const successful = results.filter(r => r.success).length;
    logger.warn('Rollback execution completed', {
      successful,
      total: results.length,
      failed: results.length - successful
    });

    return results;
  }

  /**
   * Execute single rollback
   */
  private async executeRollback(migration: Migration): Promise<MigrationResult> {
    const startTime = Date.now();
    
    try {
      logger.warn('Rolling back migration', { 
        id: migration.id, 
        version: migration.version, 
        name: migration.name 
      });

      if (!migration.down) {
        throw new Error(`Migration ${migration.id} has no rollback method`);
      }

      const migrationContext = this.createMigrationContext(migration);
      await migration.down.call(migrationContext);
      
      const executionTime = Date.now() - startTime;
      
      // Remove migration from history
      await this.removeMigrationRecord(migration.id);
      
      logger.warn('Migration rolled back successfully', { 
        id: migration.id, 
        executionTime: `${executionTime}ms` 
      });

      return {
        id: migration.id,
        version: migration.version,
        success: true,
        executionTime,
        rollbackAvailable: false
      };
    } catch (error) {
      const executionTime = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      
      logger.error('Rollback failed', { 
        id: migration.id, 
        error: errorMessage,
        executionTime: `${executionTime}ms`
      });

      return {
        id: migration.id,
        version: migration.version,
        success: false,
        executionTime,
        error: errorMessage,
        rollbackAvailable: false
      };
    }
  }

  /**
   * Remove migration record from database
   */
  private async removeMigrationRecord(migrationId: string): Promise<void> {
    await this.db.query('DELETE FROM schema_migrations WHERE id = ?', [migrationId]);
    
    // Update local history
    this.migrationHistory = this.migrationHistory.filter(h => h.id !== migrationId);
  }

  /**
   * Get current schema version
   */
  getCurrentVersion(): number {
    if (this.migrationHistory.length === 0) {
      return 0;
    }
    return Math.max(...this.migrationHistory.map(h => h.version));
  }

  /**
   * Get migration status
   */
  async getStatus(): Promise<{
    currentVersion: number;
    appliedMigrations: number;
    pendingMigrations: number;
    totalMigrations: number;
    lastMigration?: MigrationState;
  }> {
    const plan = await this.createMigrationPlan();
    const currentVersion = this.getCurrentVersion();
    const lastMigration = this.migrationHistory[this.migrationHistory.length - 1];

    return {
      currentVersion,
      appliedMigrations: this.migrationHistory.length,
      pendingMigrations: plan.totalMigrations,
      totalMigrations: this.migrations.size,
      lastMigration
    };
  }

  /**
   * Generate checksum for migration
   */
  private generateChecksum(migration: Migration): string {
    const content = `${migration.id}_${migration.version}_${migration.name}_${migration.description}`;
    return Buffer.from(content).toString('base64');
  }

  /**
   * Validate migration integrity
   */
  async validateIntegrity(): Promise<{ valid: boolean; issues: string[] }> {
    const issues: string[] = [];

    // Check for duplicate versions
    const versions = Array.from(this.migrations.values()).map(m => m.version);
    const duplicateVersions = versions.filter((v, i) => versions.indexOf(v) !== i);
    
    if (duplicateVersions.length > 0) {
      issues.push(`Duplicate migration versions: ${duplicateVersions.join(', ')}`);
    }

    // Check for missing dependencies
    for (const migration of this.migrations.values()) {
      if (migration.dependencies) {
        for (const dep of migration.dependencies) {
          const depVersion = parseInt(dep);
          if (!versions.includes(depVersion)) {
            issues.push(`Migration ${migration.id} depends on missing migration ${dep}`);
          }
        }
      }
    }

    // Check checksum integrity
    for (const historyEntry of this.migrationHistory) {
      const migration = this.migrations.get(historyEntry.id);
      if (migration) {
        const expectedChecksum = this.generateChecksum(migration);
        if (expectedChecksum !== historyEntry.checksum) {
          issues.push(`Migration ${historyEntry.id} checksum mismatch - migration may have been modified after application`);
        }
      }
    }

    return {
      valid: issues.length === 0,
      issues
    };
  }
}