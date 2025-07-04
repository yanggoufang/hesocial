#!/usr/bin/env node

import { MigrationRunner } from './MigrationRunner.js';
import { connectDatabases, closeDatabases } from '../connection.js';
import logger from '../../utils/logger.js';

interface CLIOptions {
  command: string;
  targetVersion?: number;
  force?: boolean;
  dryRun?: boolean;
}

class MigrationCLI {
  private runner: MigrationRunner;

  constructor() {
    this.runner = new MigrationRunner();
  }

  async run(options: CLIOptions): Promise<void> {
    try {
      // Commands that don't need database connection
      if (options.command === 'create' || options.command === '' || !options.command) {
        switch (options.command) {
          case 'create':
            await this.createMigration();
            break;
          default:
            this.showHelp();
        }
        return;
      }

      // Connect to database for commands that need it
      await connectDatabases();
      await this.runner.initialize();

      switch (options.command) {
        case 'status':
          await this.showStatus();
          break;
        case 'migrate':
          await this.runMigrations(options.dryRun);
          break;
        case 'rollback':
          if (!options.targetVersion) {
            throw new Error('Target version required for rollback');
          }
          await this.runRollback(options.targetVersion, options.force, options.dryRun);
          break;
        case 'validate':
          await this.validateMigrations();
          break;
        case 'history':
          await this.showHistory();
          break;
        default:
          this.showHelp();
      }
    } catch (error) {
      logger.error('Migration CLI error', { 
        error: error instanceof Error ? error.message : error 
      });
      process.exit(1);
    } finally {
      // Always close database connections
      try {
        await closeDatabases();
      } catch (error) {
        logger.warn('Error closing database connections', { error });
      }
    }
  }

  private async showStatus(): Promise<void> {
    const status = await this.runner.getStatus();
    
    console.log('='.repeat(60));
    console.log('📊 MIGRATION STATUS');
    console.log('='.repeat(60));
    console.log(`Current Version: ${status.currentVersion}`);
    console.log(`Applied Migrations: ${status.appliedMigrations}`);
    console.log(`Pending Migrations: ${status.pendingMigrations}`);
    console.log(`Total Migrations: ${status.totalMigrations}`);
    
    if (status.lastMigration) {
      console.log(`Last Migration: ${status.lastMigration.name} (${status.lastMigration.applied_at.toISOString()})`);
    }
    
    if (status.pendingMigrations > 0) {
      console.log('');
      console.log('⚠️  There are pending migrations. Run "migrate" to apply them.');
    } else {
      console.log('');
      console.log('✅ Database is up to date.');
    }
    
    console.log('='.repeat(60));
  }

  private async runMigrations(dryRun?: boolean): Promise<void> {
    const plan = await this.runner.createMigrationPlan();
    
    if (plan.totalMigrations === 0) {
      console.log('✅ No pending migrations to execute.');
      return;
    }

    console.log('='.repeat(60));
    console.log('🚀 MIGRATION PLAN');
    console.log('='.repeat(60));
    console.log(`Migrations to apply: ${plan.totalMigrations}`);
    console.log(`Estimated time: ${Math.round(plan.estimatedTime / 1000)}s`);
    console.log(`Can rollback: ${plan.canRollback ? '✅' : '❌'}`);
    
    if (plan.dependencies.length > 0) {
      console.log('');
      console.log('❌ DEPENDENCY ISSUES:');
      plan.dependencies.forEach(dep => console.log(`   • ${dep}`));
      return;
    }

    console.log('');
    console.log('📋 MIGRATIONS:');
    plan.migrations.forEach(m => {
      console.log(`   ${m.version}: ${m.name} (${m.category})`);
    });

    if (dryRun) {
      console.log('');
      console.log('🔍 DRY RUN - No changes will be made.');
      return;
    }

    console.log('');
    console.log('⚡ Executing migrations...');
    
    const results = await this.runner.migrate();
    
    console.log('');
    console.log('='.repeat(60));
    console.log('📊 MIGRATION RESULTS');
    console.log('='.repeat(60));
    
    results.forEach(result => {
      const status = result.success ? '✅' : '❌';
      const time = `${result.executionTime}ms`;
      console.log(`${status} ${result.id} - ${time}`);
      
      if (!result.success && result.error) {
        console.log(`   Error: ${result.error}`);
      }
    });

    const successful = results.filter(r => r.success).length;
    console.log('');
    console.log(`✅ ${successful}/${results.length} migrations completed successfully.`);
  }

  private async runRollback(targetVersion: number, force?: boolean, dryRun?: boolean): Promise<void> {
    const plan = await this.runner.createRollbackPlan(targetVersion);
    
    console.log('='.repeat(60));
    console.log('⚠️  ROLLBACK PLAN');
    console.log('='.repeat(60));
    console.log(`Target Version: ${targetVersion}`);
    console.log(`Rollback Steps: ${plan.rollbackSteps}`);
    console.log(`Estimated time: ${Math.round(plan.estimatedTime / 1000)}s`);
    
    if (plan.riskyOperations.length > 0) {
      console.log('');
      console.log('🚨 RISKY OPERATIONS:');
      plan.riskyOperations.forEach(risk => console.log(`   • ${risk}`));
      
      if (!force) {
        console.log('');
        console.log('❌ Rollback blocked due to risky operations. Use --force to proceed.');
        return;
      }
    }

    console.log('');
    console.log('📋 ROLLBACK ORDER:');
    plan.migrations.forEach(m => {
      console.log(`   ${m.version}: ${m.name} (${m.category})`);
    });

    if (dryRun) {
      console.log('');
      console.log('🔍 DRY RUN - No changes will be made.');
      return;
    }

    console.log('');
    console.log('⚡ Executing rollback...');
    
    const results = await this.runner.rollback(targetVersion, force);
    
    console.log('');
    console.log('='.repeat(60));
    console.log('📊 ROLLBACK RESULTS');
    console.log('='.repeat(60));
    
    results.forEach(result => {
      const status = result.success ? '✅' : '❌';
      const time = `${result.executionTime}ms`;
      console.log(`${status} ${result.id} - ${time}`);
      
      if (!result.success && result.error) {
        console.log(`   Error: ${result.error}`);
      }
    });

    const successful = results.filter(r => r.success).length;
    console.log('');
    console.log(`✅ ${successful}/${results.length} rollbacks completed successfully.`);
  }

  private async createMigration(): Promise<void> {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
    const version = Date.now();
    
    console.log('🔧 Creating new migration template...');
    console.log('');
    console.log('Migration template:');
    console.log('');
    console.log(`// File: ${version}_example_migration.migration.ts`);
    console.log('');
    console.log(`import { BaseMigration } from '../Migration.js';

export default class ExampleMigration extends BaseMigration {
  id = '${version}_example_migration';
  version = ${version};
  name = 'Example Migration';
  description = 'Description of what this migration does';
  category = 'schema' as const;

  async up(): Promise<void> {
    // Add your migration logic here
    await this.executeSQL(\`
      -- Example: Add new column
      ALTER TABLE users ADD COLUMN new_column VARCHAR(255);
    \`);
  }

  async down(): Promise<void> {
    // Add your rollback logic here
    await this.executeSQL(\`
      -- Example: Remove the column
      ALTER TABLE users DROP COLUMN new_column;
    \`);
  }
}`);
    
    console.log('');
    console.log('📁 Save this as: src/database/migrations/{version}_your_migration_name.migration.ts');
    console.log('');
    console.log('✅ Migration template generated.');
  }

  private async validateMigrations(): Promise<void> {
    console.log('🔍 Validating migration integrity...');
    
    const validation = await this.runner.validateIntegrity();
    
    console.log('');
    console.log('='.repeat(60));
    console.log('🔍 VALIDATION RESULTS');
    console.log('='.repeat(60));
    
    if (validation.valid) {
      console.log('✅ All migrations are valid.');
    } else {
      console.log('❌ Migration validation failed:');
      validation.issues.forEach(issue => console.log(`   • ${issue}`));
    }
  }

  private async showHistory(): Promise<void> {
    const status = await this.runner.getStatus();
    
    console.log('='.repeat(60));
    console.log('📚 MIGRATION HISTORY');
    console.log('='.repeat(60));
    
    if (status.appliedMigrations === 0) {
      console.log('No migrations have been applied yet.');
      return;
    }

    // This would need access to migration history - could be added to runner
    console.log('Migration history display would show applied migrations with timestamps');
  }

  public showHelp(): void {
    console.log('='.repeat(60));
    console.log('🛠️  MIGRATION CLI HELP');
    console.log('='.repeat(60));
    console.log('');
    console.log('Commands:');
    console.log('  status           Show current migration status');
    console.log('  migrate          Apply pending migrations');
    console.log('  rollback <ver>   Rollback to specific version');
    console.log('  create           Generate migration template');
    console.log('  validate         Validate migration integrity');
    console.log('  history          Show migration history');
    console.log('');
    console.log('Options:');
    console.log('  --dry-run        Show what would be done without executing');
    console.log('  --force          Force execution (for risky operations)');
    console.log('');
    console.log('Examples:');
    console.log('  npm run migrate status');
    console.log('  npm run migrate migrate');
    console.log('  npm run migrate rollback 001');
    console.log('  npm run migrate rollback 001 --force');
    console.log('  npm run migrate create');
  }
}

// CLI entry point
async function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    const cli = new MigrationCLI();
    cli.showHelp();
    return;
  }

  const options: CLIOptions = {
    command: args[0],
    targetVersion: args[1] ? parseInt(args[1]) : undefined,
    force: args.includes('--force'),
    dryRun: args.includes('--dry-run')
  };

  const cli = new MigrationCLI();
  await cli.run(options);
}

// Run CLI if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(error => {
    console.error('CLI Error:', error);
    process.exit(1);
  });
}

export { MigrationCLI };