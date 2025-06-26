import { connectDatabases as connectPostgres, closeDatabases as closePostgres } from './connection.js'
import { connectDatabases as connectDuckDB, closeDatabases as closeDuckDB } from './duckdb-connection.js'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

export type DatabaseType = 'duckdb' | 'postgresql'

export const getDatabaseType = (): DatabaseType => {
  const dbType = process.env.DATABASE_TYPE?.toLowerCase() as DatabaseType
  return dbType === 'postgresql' ? 'postgresql' : 'duckdb' // Default to DuckDB
}

export const connectDatabases = async (): Promise<void> => {
  const dbType = getDatabaseType()
  
  logger.info(`🗄️  Connecting to ${dbType.toUpperCase()} database...`)
  
  if (dbType === 'postgresql') {
    return connectPostgres()
  } else {
    return connectDuckDB()
  }
}

export const closeDatabases = async (): Promise<void> => {
  const dbType = getDatabaseType()
  
  logger.info(`🗄️  Closing ${dbType.toUpperCase()} database connections...`)
  
  if (dbType === 'postgresql') {
    return closePostgres()
  } else {
    return closeDuckDB()
  }
}

export const getDatabaseInfo = (): { type: DatabaseType; description: string } => {
  const dbType = getDatabaseType()
  
  return {
    type: dbType,
    description: dbType === 'postgresql' 
      ? 'PostgreSQL with Redis - Full production setup'
      : 'DuckDB - Embedded database with full functionality'
  }
}