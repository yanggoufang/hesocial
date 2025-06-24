import { runSeeds } from './migrate.js'
import logger from '@/utils/logger.js'
import { connectDatabases, closeDatabases } from './connection.js'

async function main(): Promise<void> {
  try {
    logger.info('Connecting to databases...')
    await connectDatabases()
    
    logger.info('Running database seeds...')
    await runSeeds()
    
    logger.info('✅ Database seeding completed successfully')
  } catch (error) {
    logger.error('❌ Seeding failed:', error)
    process.exit(1)
  } finally {
    await closeDatabases()
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main()
}