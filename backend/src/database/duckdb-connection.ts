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
      const dbPath = process.env.DUCKDB_PATH || join(__dirname, '../../../hesocial.duckdb')
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
export const getDuckDBConnection = (): DuckDBConnection => duckDBConnection

const ensureUserRoleColumn = async (): Promise<void> => {
  try {
    await duckdb.query(`
      ALTER TABLE users
      ADD COLUMN IF NOT EXISTS role VARCHAR DEFAULT 'user'
    `)

    await duckdb.query(`
      UPDATE users
      SET role = CASE
        WHEN email IN ('admin@hesocial.com', 'superadmin@hesocial.com') THEN 'super_admin'
        WHEN role IS NULL THEN 'user'
        ELSE role
      END
      WHERE role IS NULL
        OR (email IN ('admin@hesocial.com', 'superadmin@hesocial.com') AND role = 'user')
    `)
  } catch (error) {
    logger.warn('Unable to ensure users.role column:', error)
  }
}

const ensureSeedUserPasswords = async (): Promise<void> => {
  const adminHash = '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6'
  const testHash = '$2a$10$bt0AdKVHTbGLIwN44tp6dO9xMCf8vh2FSFje7iFt72zCfMgS0g6TK'
  const legacyPlaceholder = '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K'

  const adminEmails = ['admin@hesocial.com', 'superadmin@hesocial.com', 'events@hesocial.com']
  const testEmails = [
    'test.platinum@example.com',
    'test.diamond@example.com',
    'test.blackcard@example.com',
    'test.pending@example.com'
  ]

  try {
    const adminList = adminEmails.map(e => `'${e}'`).join(', ')
    const testList = testEmails.map(e => `'${e}'`).join(', ')

    await duckdb.query(`
      UPDATE users
      SET password_hash = '${adminHash}'
      WHERE email IN (${adminList})
        AND (password_hash = '${legacyPlaceholder}' OR password_hash IS NULL OR password_hash = '')
    `)

    await duckdb.query(`
      UPDATE users
      SET password_hash = '${testHash}'
      WHERE email IN (${testList})
        AND (password_hash = '${legacyPlaceholder}' OR password_hash IS NULL OR password_hash = '')
    `)
  } catch (error) {
    logger.warn('Unable to ensure seed user passwords:', error)
  }
}

const ensureVisitorTrackingSequences = async (): Promise<void> => {
  const sequences = [
    { table: 'visitor_sessions', sequence: 'visitor_sessions_id_seq' },
    { table: 'visitor_page_views', sequence: 'visitor_page_views_id_seq' },
    { table: 'visitor_events', sequence: 'visitor_events_id_seq' }
  ]

  for (const { table, sequence } of sequences) {
    try {
      const result = await duckdb.query(`SELECT COALESCE(MAX(id), 0) + 1 as next_id FROM ${table}`)
      const nextId = Number(result.rows[0]?.next_id || 1)

      await duckdb.query(`DROP SEQUENCE IF EXISTS ${sequence}`)
      await duckdb.query(`CREATE SEQUENCE ${sequence} START ${Math.max(1, nextId)}`)
    } catch (error) {
      logger.warn(`Unable to ensure ${sequence}:`, error)
    }
  }
}

export const connectDatabases = async (): Promise<void> => {
  await duckdb.connect()
  await ensureUserRoleColumn()
  await ensureSeedUserPasswords()
  await ensureVisitorTrackingSequences()
}

export const closeDatabases = async (): Promise<void> => {
  await duckdb.close()
}
