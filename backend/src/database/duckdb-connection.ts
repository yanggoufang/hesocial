import Database from 'duckdb'
import { readFile } from 'fs/promises'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import logger from '../utils/logger.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

class DuckDBConnection {
  private db: Database.Database | null = null
  private connection: Database.Connection | null = null

  async connect(): Promise<void> {
    try {
      // Create DuckDB instance with file storage using absolute path
      const dbPath = join(__dirname, '../../../hesocial.duckdb')
      logger.info(`Connecting to DuckDB at: ${dbPath}`)
      
      // Force persistent mode by explicitly setting file path
      this.db = new Database.Database(dbPath)
      this.connection = this.db.connect()
      
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
    try {
      if (this.connection) {
        this.connection = null
      }
      if (this.db) {
        this.db.close()
        this.db = null
      }
      logger.info('DuckDB connection closed')
    } catch (error) {
      logger.error('Error closing DuckDB connection:', error)
    }
  }

  async checkIfInitialized(): Promise<boolean> {
    try {
      // Check server_state table instead of users table
      logger.info('Checking if database is initialized...')
      const result = await this.query('SELECT COUNT(*) as count FROM server_state')
      const count = result.rows[0]?.count || 0
      logger.info(`Found ${count} server state records`)
      return count > 0
    } catch (error) {
      // If query fails (table doesn't exist), assume not initialized
      logger.info('Server state table does not exist, database not initialized')
      return false
    }
  }

  async runInit(): Promise<void> {
    try {
      logger.info('Running DuckDB initialization...')
      
      // Start explicit transaction
      await this.query('BEGIN TRANSACTION')
      
      const schemaPath = join(__dirname, '../../../database/duckdb-schema.sql')
      logger.info(`Reading schema from: ${schemaPath}`)
      const schemaSQL = await readFile(schemaPath, 'utf-8')
      logger.info(`Schema file length: ${schemaSQL.length} characters`)
      
      // Split by semicolon and execute each statement
      const rawStatements = schemaSQL.split(';')
      logger.info(`Raw statements after split: ${rawStatements.length}`)
      
      const trimmedStatements = rawStatements.map(stmt => stmt.trim())
      logger.info(`After trim: ${trimmedStatements.filter(s => s.length > 0).length} non-empty`)
      
      const statements = trimmedStatements.filter(stmt => {
        if (stmt.length === 0) return false
        
        // Remove comments and check if there's actual SQL content
        const sqlContent = stmt
          .split('\n')
          .map(line => line.trim())
          .filter(line => line.length > 0 && !line.startsWith('--'))
          .join('\n')
          .trim()
          
        return sqlContent.length > 0
      })
      
      logger.info(`After filtering comments: ${statements.length} statements`)
      
      // Debug: log first few statements
      statements.slice(0, 3).forEach((stmt, i) => {
        logger.info(`Statement ${i + 1}: ${stmt.substring(0, 100)}...`)
      })

      for (const statement of statements) {
        if (statement.trim()) {
          logger.info(`Executing: ${statement.substring(0, 50)}...`)
          await this.query(statement)
        }
      }
      
      // Commit transaction
      await this.query('COMMIT')
      
      // Verify tables were created
      const tables = await this.query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'main'")
      logger.info(`Tables created: ${tables.rows.map((r: any) => r.table_name).join(', ')}`)
      
      logger.info('✅ DuckDB schema initialization completed successfully')
    } catch (error) {
      logger.error('❌ DuckDB initialization failed:', error)
      throw error
    }
  }

  async seedData(): Promise<void> {
    try {
      logger.info('Seeding DuckDB with sample data...')
      
      // Start explicit transaction
      await this.query('BEGIN TRANSACTION')
      
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
            logger.info(`Seeding: ${statement.substring(0, 50)}...`)
            await this.query(statement)
          } catch (error) {
            logger.error(`Failed to execute statement: ${statement.substring(0, 100)}...`, error)
            await this.query('ROLLBACK')
            throw error
          }
        }
      }
      
      // Commit transaction
      await this.query('COMMIT')
      
      logger.info('✅ DuckDB seeding completed successfully')
    } catch (error) {
      logger.error('❌ DuckDB seeding failed:', error)
      throw error
    }
  }

  async recordServerStart(): Promise<void> {
    try {
      const now = new Date().toISOString()
      
      // Get current state
      const result = await this.query('SELECT * FROM server_state WHERE id = 1')
      
      if (result.rows.length === 0) {
        // First time startup - create initial record
        await this.query(`
          INSERT INTO server_state (
            id, start_count, first_start_time, last_start_time, 
            last_stop_time, last_session_duration, total_lifetime
          ) VALUES (1, 1, ?, ?, NULL, 0, 0)
        `, [now, now])
        logger.info('🎯 Server state: First startup recorded')
      } else {
        // Update existing record
        const state = result.rows[0]
        const newStartCount = (state.start_count || 0) + 1
        
        await this.query(`
          UPDATE server_state SET 
            start_count = ?,
            last_start_time = ?
          WHERE id = 1
        `, [newStartCount, now])
        
        logger.info(`🎯 Server state: Startup #${newStartCount} recorded`)
      }
    } catch (error) {
      logger.error('Failed to record server start:', error)
    }
  }

  async recordServerStop(): Promise<void> {
    try {
      const now = new Date().toISOString()
      
      // Get current state to calculate session duration
      const result = await this.query('SELECT * FROM server_state WHERE id = 1')
      
      if (result.rows.length > 0) {
        const state = result.rows[0]
        const lastStartTime = new Date(state.last_start_time)
        const stopTime = new Date(now)
        const sessionDuration = Math.floor((stopTime.getTime() - lastStartTime.getTime()) / 1000) // seconds
        const newTotalLifetime = (state.total_lifetime || 0) + sessionDuration
        
        await this.query(`
          UPDATE server_state SET 
            last_stop_time = ?,
            last_session_duration = ?,
            total_lifetime = ?
          WHERE id = 1
        `, [now, sessionDuration, newTotalLifetime])
        
        logger.info(`🎯 Server state: Shutdown recorded (session: ${sessionDuration}s, total: ${newTotalLifetime}s)`)
      }
    } catch (error) {
      logger.error('Failed to record server stop:', error)
    }
  }

  async getServerStats(): Promise<any> {
    try {
      const result = await this.query('SELECT * FROM server_state WHERE id = 1')
      return result.rows[0] || null
    } catch (error) {
      logger.error('Failed to get server stats:', error)
      return null
    }
  }
}

// Export singleton instance
export const duckdb = new DuckDBConnection()

// Export individual DB connection getter for compatibility
export const getDuckDBConnection = () => duckdb

// Mock Redis functions for development
export const redisClient = {
  connect: async () => logger.info('Redis mock: connected'),
  quit: async () => logger.info('Redis mock: disconnected'),
  on: (event: string, callback: Function) => logger.info(`Redis mock: listening to ${event}`)
}

export const connectDatabases = async (): Promise<void> => {
  try {
    await duckdb.connect()
    
    const isInitialized = await duckdb.checkIfInitialized()
    
    if (!isInitialized) {
      logger.info('Database not initialized, running setup...')
      await duckdb.runInit()
      await duckdb.seedData()
      logger.info('✅ DuckDB setup completed - Full functionality available')
    } else {
      logger.info('✅ DuckDB already initialized - Full functionality available')
    }
    
    // Record server startup
    await duckdb.recordServerStart()
    
    // Display server stats
    const stats = await duckdb.getServerStats()
    if (stats) {
      logger.info(`📊 Server Stats: Startup #${stats.start_count}, Total uptime: ${Math.floor(stats.total_lifetime / 60)}min`)
    }
    
  } catch (error) {
    logger.error('Failed to connect to databases:', error)
    throw error
  }
}

export const closeDatabases = async (): Promise<void> => {
  try {
    // Record server stop before closing
    await duckdb.recordServerStop()
    await duckdb.close()
    logger.info('Database connections closed')
  } catch (error) {
    logger.error('Error closing database connections:', error)
    throw error
  }
}