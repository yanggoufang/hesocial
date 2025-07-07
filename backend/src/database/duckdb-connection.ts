import Database from 'duckdb';
import { readFile } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import logger from '../utils/logger.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

class DuckDBConnectionManager {
  private static instance: DuckDBConnectionManager;
  private db: Database.Database | null = null;
  private connection: Database.Connection | null = null;

  private constructor() {}

  public static getInstance(): DuckDBConnectionManager {
    if (!DuckDBConnectionManager.instance) {
      DuckDBConnectionManager.instance = new DuckDBConnectionManager();
    }
    return DuckDBConnectionManager.instance;
  }

  public async connect(): Promise<void> {
    if (this.connection) {
      return;
    }

    try {
      const dbPath = join(__dirname, '../../../hesocial.duckdb');
      logger.info(`Connecting to DuckDB at: ${dbPath}`);
      this.db = new Database.Database(dbPath);
      this.connection = this.db.connect();
      logger.info('Connected to DuckDB database');
    } catch (error) {
      logger.error('Failed to connect to DuckDB:', error);
      throw error;
    }
  }

  public async query(sql: string, params: any[] = []): Promise<any> {
    if (!this.connection) {
      await this.connect();
    }

    return new Promise((resolve, reject) => {
      this.connection!.all(sql, ...params, (err: Error | null, result: any) => {
        if (err) {
          logger.error('DuckDB query error:', { sql, params, error: err.message });
          reject(err);
        } else {
          resolve({ rows: result || [] });
        }
      });
    });
  }

  public async prepare(sql: string): Promise<any> {
    if (!this.connection) {
      await this.connect();
    }
    return this.connection!.prepare(sql);
  }

  public async close(): Promise<void> {
    return new Promise((resolve) => {
      if (this.db) {
        this.db.close(() => {
          logger.info('DuckDB database closed.');
          this.db = null;
          this.connection = null;
          resolve();
        });
      } else {
        resolve();
      }
    });
  }

  public async getServerStats(): Promise<any> {
    if (!this.connection) {
      await this.connect();
    }
    // This is a placeholder implementation. You may need to adjust it based on your actual needs.
    return this.query('SELECT * FROM server_state WHERE id = 1');
  }

  public async seedData(): Promise<void> {
    if (!this.connection) {
      await this.connect();
    }
    // This is a placeholder implementation. You may need to adjust it based on your actual needs.
    const seedPath = join(__dirname, '../../../database/duckdb-seed.sql');
    const seedSQL = await readFile(seedPath, 'utf-8');
    const statements = seedSQL.split(';').filter(s => s.trim());
    for (const statement of statements) {
      await this.query(statement);
    }
  }
}

export const duckdb = DuckDBConnectionManager.getInstance();

export const connectDatabases = async (): Promise<void> => {
  await duckdb.connect();
  // Additional database setup logic can go here
};

export const closeDatabases = async (): Promise<void> => {
  await duckdb.close();
};