// Enhanced DuckDB Pool Adapter with PostgreSQL compatibility
// Provides comprehensive database interface with enhanced error handling

import { duckdb } from './duckdb-connection.js';
import logger from '../utils/logger.js';

export interface DatabasePool {
  query(sql: string, params?: any[]): Promise<any>;
  connect(): Promise<DatabaseConnection>;
  end(): Promise<void>;
}

export interface DatabaseConnection {
  query(sql: string, params?: any[]): Promise<any>;
  release(): void;
}

export class EnhancedDuckDBPool implements DatabasePool {
  private isConnected: boolean = false;
  private queryCount: number = 0;
  private startTime: Date = new Date();

  /**
   * Execute query with PostgreSQL-style parameter substitution
   * Converts $1, $2, etc. to DuckDB's ? placeholders
   */
  async query(sql: string, params: any[] = []): Promise<any> {
    const queryId = ++this.queryCount;
    const queryStart = Date.now();
    
    try {
      // Convert PostgreSQL-style parameters ($1, $2, etc.) to DuckDB-style (?)
      const convertedSql = this.convertParameters(sql, params);
      
      logger.debug('Executing query', { 
        queryId,
        sql: convertedSql, 
        paramCount: params?.length || 0,
        params: this.shouldLogParams(sql) ? params : '[REDACTED]'
      });

      const result = await duckdb.query(convertedSql, params);
      
      const queryDuration = Date.now() - queryStart;
      logger.debug('Query completed', { 
        queryId,
        duration: `${queryDuration}ms`,
        rowCount: result.rows?.length || 0
      });

      return result;
    } catch (error) {
      const queryDuration = Date.now() - queryStart;
      logger.error('Database query failed', { 
        queryId,
        sql: this.sanitizeSqlForLogging(sql),
        paramCount: params?.length || 0,
        duration: `${queryDuration}ms`,
        error: error instanceof Error ? error.message : error 
      });
      throw new Error(`Database query failed: ${error instanceof Error ? error.message : error}`);
    }
  }

  /**
   * Get a connection from the pool (for compatibility)
   * Since DuckDB is single-threaded, we return a wrapper
   */
  async connect(): Promise<DatabaseConnection> {
    return new DuckDBConnection(this);
  }

  /**
   * Close the database connection
   */
  async end(): Promise<void> {
    try {
      const uptime = Date.now() - this.startTime.getTime();
      logger.info('Closing database connection', { 
        totalQueries: this.queryCount,
        uptime: `${Math.floor(uptime / 1000)}s`
      });
      
      await duckdb.close();
      this.isConnected = false;
    } catch (error) {
      logger.error('Error closing database connection', { 
        error: error instanceof Error ? error.message : error 
      });
      throw error;
    }
  }

  /**
   * Convert PostgreSQL-style parameters to DuckDB-style
   * $1, $2, $3 -> ?, ?, ?
   * Also handles proper parameter ordering
   */
  private convertParameters(sql: string, params: any[]): string {
    if (!params || params.length === 0) {
      return sql;
    }

    // Replace $1, $2, etc. with ? placeholders in the correct order
    let convertedSql = sql;
    for (let i = params.length; i >= 1; i--) {
      convertedSql = convertedSql.replace(new RegExp(`\\$${i}`, 'g'), '?');
    }

    return convertedSql;
  }

  /**
   * Determine if query parameters should be logged (exclude sensitive data)
   */
  private shouldLogParams(sql: string): boolean {
    const sensitivePatterns = [
      /password/i,
      /secret/i,
      /token/i,
      /key/i,
      /auth/i,
      /hash/i
    ];
    
    return !sensitivePatterns.some(pattern => pattern.test(sql));
  }

  /**
   * Sanitize SQL for logging (remove sensitive data)
   */
  private sanitizeSqlForLogging(sql: string): string {
    // Remove potential sensitive data from SQL for logging
    return sql.replace(/(?:password|secret|token|key)\s*=\s*['"]\S*['"]/gi, 'REDACTED');
  }

  /**
   * Get pool statistics
   */
  getStats(): { queryCount: number; uptime: number } {
    return {
      queryCount: this.queryCount,
      uptime: Date.now() - this.startTime.getTime()
    };
  }
}

/**
 * Individual database connection wrapper
 */
export class DuckDBConnection implements DatabaseConnection {
  private pool: EnhancedDuckDBPool;

  constructor(pool: EnhancedDuckDBPool) {
    this.pool = pool;
  }

  /**
   * Execute query with parameter conversion
   */
  async query(sql: string, params?: any[]): Promise<any> {
    return this.pool.query(sql, params);
  }

  /**
   * Release connection (no-op for DuckDB)
   */
  release(): void {
    // No-op for DuckDB since it's single-threaded
  }
}

// Create enhanced pool instance
const enhancedPool = new EnhancedDuckDBPool();

// Export legacy compatible pool interface
export const pool = {
  async query(sql: string, params: any[] = []) {
    return enhancedPool.query(sql, params);
  },

  async connect() {
    return enhancedPool.connect();
  },

  async end() {
    return enhancedPool.end();
  }
};

// Export enhanced pool for new code
export const duckdbPool = enhancedPool;

// Helper functions
export async function executeQuery(sql: string, params?: any[]): Promise<any> {
  return enhancedPool.query(sql, params);
}

export async function executeQueryOne(sql: string, params?: any[]): Promise<any> {
  const result = await enhancedPool.query(sql, params);
  return result.rows && result.rows.length > 0 ? result.rows[0] : null;
}

export async function executeQueryAll(sql: string, params?: any[]): Promise<any[]> {
  const result = await enhancedPool.query(sql, params);
  return result.rows || [];
}

export async function testDatabaseConnection(): Promise<boolean> {
  try {
    const result = await enhancedPool.query('SELECT 1 as test');
    return result.rows && result.rows.length > 0 && result.rows[0].test === 1;
  } catch (error) {
    logger.error('Database connection test failed', { 
      error: error instanceof Error ? error.message : error 
    });
    return false;
  }
}

// Enhanced Redis client mock with better logging
export const redisClient = {
  connect: async () => {
    logger.info('Redis mock: connected');
    return Promise.resolve();
  },
  quit: async () => {
    logger.info('Redis mock: disconnected');
    return Promise.resolve();
  },
  on: (event: string, callback: Function) => {
    logger.debug(`Redis mock: listening to ${event}`);
    return this;
  }
};