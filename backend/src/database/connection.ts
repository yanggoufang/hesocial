import { Pool } from 'pg'
import { createClient } from 'redis'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

export const pool = new Pool({
  host: config.database.host,
  port: config.database.port,
  database: config.database.database,
  user: config.database.username,
  password: config.database.password,
  ssl: config.database.ssl ? { rejectUnauthorized: false } : false,
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
})

export const redisClient = createClient({
  socket: {
    host: config.redis.host,
    port: config.redis.port,
  },
  password: config.redis.password,
})

pool.on('connect', () => {
  logger.info('Connected to PostgreSQL database')
})

pool.on('error', (err) => {
  logger.error('PostgreSQL pool error:', err)
})

redisClient.on('connect', () => {
  logger.info('Connected to Redis')
})

redisClient.on('error', (err) => {
  logger.error('Redis error:', err)
})

export const connectDatabases = async (): Promise<void> => {
  try {
    await pool.connect()
    await redisClient.connect()
    logger.info('Database connections established')
  } catch (error) {
    logger.error('Failed to connect to databases:', error)
    throw error
  }
}

export const closeDatabases = async (): Promise<void> => {
  try {
    await pool.end()
    await redisClient.quit()
    logger.info('Database connections closed')
  } catch (error) {
    logger.error('Error closing database connections:', error)
    throw error
  }
}