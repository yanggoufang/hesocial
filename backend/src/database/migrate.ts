import { readFile } from 'fs/promises'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { pool } from './connection.js'
import logger from '@/utils/logger.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

async function runMigrations(): Promise<void> {
  try {
    logger.info('Starting database migration...')
    
    const schemaPath = join(__dirname, '../../database/schema.sql')
    const schemaSQL = await readFile(schemaPath, 'utf-8')
    
    await pool.query(schemaSQL)
    
    logger.info('✅ Database schema migration completed successfully')
  } catch (error) {
    logger.error('❌ Database migration failed:', error)
    throw error
  }
}

async function runSeeds(): Promise<void> {
  try {
    logger.info('Starting database seeding...')
    
    const seedPath = join(__dirname, '../../database/seed.sql')
    const seedSQL = await readFile(seedPath, 'utf-8')
    
    await pool.query(seedSQL)
    
    logger.info('✅ Database seeding completed successfully')
  } catch (error) {
    logger.error('❌ Database seeding failed:', error)
    throw error
  }
}

async function main(): Promise<void> {
  try {
    await runMigrations()
    
    const shouldSeed = process.argv.includes('--seed')
    if (shouldSeed) {
      await runSeeds()
    }
    
    logger.info('🎉 Database setup completed')
    process.exit(0)
  } catch (error) {
    logger.error('Database setup failed:', error)
    process.exit(1)
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main()
}

export { runMigrations, runSeeds }