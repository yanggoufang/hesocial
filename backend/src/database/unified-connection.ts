import { connectDatabases as connectDuckDB, closeDatabases as closeDuckDB } from './duckdb-connection.js'
import logger from '@/utils/logger.js'

export type DatabaseType = 'duckdb'

export const getDatabaseType = (): DatabaseType => {
  return 'duckdb'
}

export const connectDatabases = async (): Promise<void> => {
  logger.info('🗄️  Connecting to DuckDB database...')
  return connectDuckDB()
}

export const closeDatabases = async (): Promise<void> => {
  logger.info('🗄️  Closing DuckDB database connections...')
  return closeDuckDB()
}

export const getDatabaseInfo = (): { type: DatabaseType; description: string } => {
  return {
    type: 'duckdb',
    description: 'DuckDB - Production-ready embedded database'
  }
}