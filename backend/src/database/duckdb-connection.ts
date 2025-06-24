import Database from 'duckdb'
import { readFile } from 'fs/promises'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import logger from '@/utils/logger.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

class DuckDBConnection {
  private db: Database.Database | null = null
  private connection: Database.Connection | null = null

  async connect(): Promise<void> {
    try {
      // Create DuckDB instance with file storage
      this.db = new Database.Database('./hesocial.duckdb')
      this.connection = this.db.connect()
      
      // UUID extension not available, will use alternative approach
      
      logger.info('Connected to DuckDB database')
    } catch (error) {
      logger.error('Failed to connect to DuckDB:', error)
      throw error
    }
  }

  async query(sql: string, params: any[] = []): Promise<any> {
    if (!this.connection) {
      throw new Error('Database not connected')
    }

    return new Promise((resolve, reject) => {
      this.connection!.all(sql, ...params, (err: Error | null, result: any) => {
        if (err) {
          logger.error('DuckDB query error:', { sql, params, error: err.message })
          reject(err)
        } else {
          resolve({ rows: result || [] })
        }
      })
    })
  }

  async close(): Promise<void> {
    if (this.connection) {
      this.connection.close()
      this.connection = null
    }
    if (this.db) {
      this.db.close()
      this.db = null
    }
    logger.info('DuckDB connection closed')
  }

  async runMigrations(): Promise<void> {
    try {
      logger.info('Running DuckDB migrations...')
      
      const schemaPath = join(__dirname, '../../../database/duckdb-schema.sql')
      const schemaSQL = await readFile(schemaPath, 'utf-8')
      
      // Split by semicolon and execute each statement
      const statements = schemaSQL
        .split(';')
        .map(stmt => stmt.trim())
        .filter(stmt => stmt.length > 0 && !stmt.startsWith('--'))

      for (const statement of statements) {
        if (statement.trim()) {
          await this.query(statement)
        }
      }
      
      logger.info('✅ DuckDB migrations completed successfully')
    } catch (error) {
      logger.error('❌ DuckDB migration failed:', error)
      throw error
    }
  }

  async seedData(): Promise<void> {
    try {
      logger.info('Seeding DuckDB with sample data...')
      
      const seedPath = join(__dirname, '../../../database/duckdb-seed.sql')
      const seedSQL = await readFile(seedPath, 'utf-8')
      
      // Split by semicolon and execute each statement
      const statements = seedSQL
        .split(';')
        .map(stmt => stmt.trim())
        .filter(stmt => stmt.length > 0 && !stmt.startsWith('--'))

      for (const statement of statements) {
        if (statement.trim() && statement.trim().length > 10) {
          try {
            await this.query(statement)
          } catch (error) {
            logger.error(`Failed to execute statement: ${statement.substring(0, 100)}...`, error)
            throw error
          }
        }
      }
      
      logger.info('✅ DuckDB seeding completed successfully')
    } catch (error) {
      logger.error('❌ DuckDB seeding failed:', error)
      throw error
    }
  }
}

// Export singleton instance
export const duckdb = new DuckDBConnection()

// Mock Redis functions for development
export const redisClient = {
  connect: async () => logger.info('Redis mock: connected'),
  quit: async () => logger.info('Redis mock: disconnected'),
  on: (event: string, callback: Function) => logger.info(`Redis mock: listening to ${event}`)
}

export const connectDatabases = async (): Promise<void> => {
  try {
    await duckdb.connect()
    await duckdb.runMigrations()
    await duckdb.seedData()
    logger.info('✅ DuckDB setup completed - Full functionality available')
  } catch (error) {
    logger.error('Failed to connect to databases:', error)
    throw error
  }
}

export const closeDatabases = async (): Promise<void> => {
  try {
    await duckdb.close()
    logger.info('Database connections closed')
  } catch (error) {
    logger.error('Error closing database connections:', error)
    throw error
  }
}