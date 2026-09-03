import { readFile } from 'fs/promises'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { duckdb, ensureSeedUsers } from './duckdb-connection.js'
import logger from '../utils/logger.js'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

async function runSeedFile(filename: string) {
  try {
    const seedPath = join(__dirname, '../../../database', filename)
    logger.info(`Running seed file: ${filename}`)
    
    const seedSQL = await readFile(seedPath, 'utf-8')
    const statements = seedSQL
      .split(';')
      .map(s => s.trim())
      .filter(s => s.length > 0 && !s.startsWith('--'))
    
    for (const statement of statements) {
      if (statement.trim()) {
        try {
          await duckdb.query(statement)
        } catch (error) {
          logger.warn(`Statement failed (continuing): ${statement.substring(0, 100)}...`, error)
        }
      }
    }
    
    logger.info(`✅ Completed seed file: ${filename}`)
  } catch (error) {
    logger.error(`❌ Failed to run seed file ${filename}:`, error)
    throw error
  }
}

async function seed() {
  try {
    logger.info('🌱 Starting database seeding...')

    if (process.env.NODE_ENV === 'production') {
      logger.warn('⚠️  Seeding in production plants KNOWN default credentials (admin123 / test123). Rotate all seeded passwords immediately after seeding.')
    }
    
    // Connect to database
    await duckdb.connect()
    
    // Run seed files in order
    await runSeedFile('duckdb-schema.sql')
    await runSeedFile('event-management-schema.sql')
    await runSeedFile('duckdb-admin-users.sql')
    await runSeedFile('duckdb-seed-realistic.sql')
    await ensureSeedUsers()
    
    logger.info('🎉 Database seeding completed successfully!')
    
  } catch (error) {
    logger.error('❌ Database seeding failed:', error)
    process.exit(1)
  } finally {
    await duckdb.close()
  }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  seed()
}

export default seed
