import duckdbModule from 'duckdb'
import { readFile } from 'fs/promises'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import logger from '../utils/logger.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

class DuckDBConnection {
  private db: duckdbModule.Database | null = null
  private connection: duckdbModule.Connection | null = null

  async connect(): Promise<void> {
    if (this.connection) {
      return
    }

    try {
      // Create DuckDB instance with file storage using absolute path
      const dbPath = join(__dirname, '../../../hesocial.duckdb')
      logger.info(`Connecting to DuckDB at: ${dbPath}`)
      
      // Force persistent mode by explicitly setting file path
      this.db = new duckdbModule.Database(dbPath)
      this.connection = this.db.connect()
      
      // Test the connection with a simple query
      await this.query('SELECT 1 as test')
      
      logger.info('Connected to DuckDB database')
    } catch (error) {
      logger.error('Failed to connect to DuckDB:', error)
      throw error
    }
  }

  async query(sql: string, params: any[] = []): Promise<any> {
    if (!this.connection) {
      await this.connect()
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

  prepare(sql: string) {
    if (!this.connection) {
      throw new Error('Database not connected')
    }
    
    return {
      run: async (...params: any[]) => {
        return new Promise((resolve, reject) => {
          this.connection!.run(sql, ...params, (err: Error | null, result: any) => {
            if (err) {
              logger.error('DuckDB prepare run error:', { sql, params, error: err.message })
              reject(err)
            } else {
              resolve(result)
            }
          })
        })
      },
      get: async (...params: any[]) => {
        return new Promise((resolve, reject) => {
          this.connection!.all(sql, ...params, (err: Error | null, result: any) => {
            if (err) {
              logger.error('DuckDB prepare get error:', { sql, params, error: err.message })
              reject(err)
            } else {
              resolve(result && result.length > 0 ? result[0] : null)
            }
          })
        })
      },
      all: async (...params: any[]) => {
        return new Promise((resolve, reject) => {
          this.connection!.all(sql, ...params, (err: Error | null, result: any) => {
            if (err) {
              logger.error('DuckDB prepare all error:', { sql, params, error: err.message })
              reject(err)
            } else {
              resolve(result || [])
            }
          })
        })
      }
    }
  }

  async close(): Promise<void> {
    try {
      if (this.connection) {
        this.connection = null
      }
      if (this.db) {
        this.db.close(() => {
          logger.info('DuckDB database closed.')
          this.db = null
        })
      }
    } catch (error) {
      logger.error('Error closing DuckDB connection:', error)
      throw error
    }
  }

  async getServerStats(): Promise<any> {
    if (!this.connection) {
      await this.connect()
    }
    // This is a placeholder implementation. You may need to adjust it based on your actual needs.
    return this.query('SELECT * FROM server_state WHERE id = 1')
  }

  async seedData(): Promise<void> {
    if (!this.connection) {
      await this.connect()
    }
    // This is a placeholder implementation. You may need to adjust it based on your actual needs.
    const seedPath = join(__dirname, '../../../database/duckdb-seed.sql')
    const seedSQL = await readFile(seedPath, 'utf-8')
    const statements = seedSQL.split(';').filter(s => s.trim())
    for (const statement of statements) {
      await this.query(statement)
    }
  }
}

const duckDBConnection = new DuckDBConnection()
export const duckdb = duckDBConnection

export const connectDatabases = async (): Promise<void> => {
  await duckdb.connect()
  // Additional database setup logic can go here
}

export const closeDatabases = async (): Promise<void> => {
  await duckdb.close()
}