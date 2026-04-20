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

const ensureSeedUsers = async (): Promise<void> => {
  const adminHash = '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6'
  const testHash = '$2a$10$bt0AdKVHTbGLIwN44tp6dO9xMCf8vh2FSFje7iFt72zCfMgS0g6TK'

  try {
    await duckdb.query(`
      INSERT OR IGNORE INTO users (
        id, email, password_hash, first_name, last_name, age, profession,
        annual_income, net_worth, membership_tier, privacy_level,
        is_verified, verification_status, role, bio, interests,
        created_at, updated_at
      ) VALUES
        (
          1001,
          'superadmin@hesocial.com',
          '${adminHash}',
          'Super',
          'Admin',
          40,
          'System Developer',
          5000000,
          30000000,
          'Black Card',
          5,
          true,
          'approved',
          'super_admin',
          'Platform super administrator for development and system maintenance.',
          ['system development', 'database management', 'security auditing', 'feature development'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        ),
        (
          1002,
          'events@hesocial.com',
          '${adminHash}',
          'Event',
          'Manager',
          42,
          'Event Coordinator',
          5000000,
          30000000,
          'Diamond',
          3,
          true,
          'approved',
          'admin',
          'Specialized administrator for luxury event planning and member experience management.',
          ['luxury events', 'hospitality', 'member relations', 'venue management'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        ),
        (
          2001,
          'test.platinum@example.com',
          '${testHash}',
          'Test',
          'Platinum',
          45,
          'Business Owner',
          8000000,
          50000000,
          'Platinum',
          3,
          true,
          'approved',
          'user',
          'Test user account for Platinum tier membership testing and development.',
          ['business development', 'networking', 'fine dining', 'travel'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        ),
        (
          2002,
          'test.diamond@example.com',
          '${testHash}',
          'Test',
          'Diamond',
          50,
          'Investment Manager',
          15000000,
          120000000,
          'Diamond',
          4,
          true,
          'approved',
          'user',
          'Test user account for Diamond tier membership testing and premium feature validation.',
          ['investment strategy', 'luxury lifestyle', 'art collection', 'private events'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        ),
        (
          2003,
          'test.blackcard@example.com',
          '${testHash}',
          'Test',
          'BlackCard',
          55,
          'Private Equity Partner',
          50000000,
          500000000,
          'Black Card',
          5,
          true,
          'approved',
          'user',
          'Test user account for Black Card tier membership testing and exclusive feature validation.',
          ['private equity', 'exclusive events', 'yacht ownership', 'philanthropy'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        ),
        (
          2004,
          'test.pending@example.com',
          '${testHash}',
          'Test',
          'Pending',
          42,
          'Startup Founder',
          6000000,
          40000000,
          'Platinum',
          2,
          false,
          'pending',
          'user',
          'Test user account for testing the verification process and pending state handling.',
          ['startup ecosystem', 'technology', 'innovation', 'venture capital'],
          CURRENT_TIMESTAMP,
          CURRENT_TIMESTAMP
        )
    `)
  } catch (error) {
    logger.warn('Unable to ensure seed users:', error)
  }
}

const ensureSeedUserPasswords = async (): Promise<void> => {
  const adminHash = '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6'
  const testHash = '$2a$10$bt0AdKVHTbGLIwN44tp6dO9xMCf8vh2FSFje7iFt72zCfMgS0g6TK'

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

    const before = await duckdb.query(`
      SELECT email, LENGTH(password_hash) AS hash_len
      FROM users
      WHERE email IN (${adminList}, ${testList})
    `)
    logger.info('ensureSeedUserPasswords before:', before.rows)

    const adminResult = await duckdb.query(`
      UPDATE users
      SET password_hash = '${adminHash}'
      WHERE email IN (${adminList})
    `)
    logger.info('ensureSeedUserPasswords admin update result:', adminResult.rows)

    const testResult = await duckdb.query(`
      UPDATE users
      SET password_hash = '${testHash}'
      WHERE email IN (${testList})
    `)
    logger.info('ensureSeedUserPasswords test update result:', testResult.rows)
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
  await ensureSeedUsers()
  await ensureSeedUserPasswords()
  await ensureVisitorTrackingSequences()
}

export const closeDatabases = async (): Promise<void> => {
  await duckdb.close()
}
