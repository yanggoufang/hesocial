export interface Migration {
  id: string;
  version: number;
  name: string;
  description: string;
  up: () => Promise<void>;
  down: () => Promise<void>;
  dependencies?: string[];
  category?: 'schema' | 'data' | 'index' | 'constraint';
  executionTime?: number;
  checksum?: string;
}

export interface MigrationResult {
  id: string;
  version: number;
  success: boolean;
  executionTime: number;
  error?: string;
  rollbackAvailable: boolean;
}

export interface MigrationState {
  id: string;
  version: number;
  name: string;
  description: string;
  category: string;
  checksum: string;
  applied_at: Date;
  execution_time: number;
  applied_by: string;
  rollback_sql?: string;
}

export interface MigrationPlan {
  migrations: Migration[];
  totalMigrations: number;
  estimatedTime: number;
  canRollback: boolean;
  dependencies: string[];
}

export interface RollbackPlan {
  migrations: Migration[];
  targetVersion: number;
  rollbackSteps: number;
  estimatedTime: number;
  riskyOperations: string[];
}

export abstract class BaseMigration implements Migration {
  abstract id: string;
  abstract version: number;
  abstract name: string;
  abstract description: string;
  abstract category: 'schema' | 'data' | 'index' | 'constraint';
  
  dependencies?: string[] = [];
  executionTime?: number;
  checksum?: string;

  abstract up(): Promise<void>;
  abstract down(): Promise<void>;

  protected async executeSQL(sql: string): Promise<void> {
    // This will be implemented by the migration runner
    throw new Error('executeSQL must be implemented by migration runner');
  }

  protected async executeSQLWithResult(sql: string): Promise<any> {
    // This will be implemented by the migration runner
    throw new Error('executeSQLWithResult must be implemented by migration runner');
  }

  protected generateChecksum(): string {
    const content = `${this.id}_${this.version}_${this.name}_${this.description}`;
    // Simple checksum for now - could use crypto.createHash in production
    return Buffer.from(content).toString('base64');
  }

  protected validateMigration(): void {
    if (!this.id || !this.version || !this.name || !this.description) {
      throw new Error(`Invalid migration: ${this.id} - missing required fields`);
    }
    
    if (this.version <= 0) {
      throw new Error(`Invalid migration version: ${this.version} - must be positive`);
    }
  }
}