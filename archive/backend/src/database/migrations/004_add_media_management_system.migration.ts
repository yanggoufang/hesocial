import { BaseMigration } from './Migration.js'
import logger from '../../utils/logger.js'

export default class AddMediaManagementSystem extends BaseMigration {
  id = '004_add_media_management_system'
  version = 4
  name = 'Add Media Management System'
  description = 'Add event and venue media tables for R2 storage integration'
  category = 'schema' as const
  
  async up(): Promise<void> {
    // Create event media table
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS event_media (
        id VARCHAR PRIMARY KEY,
        event_id VARCHAR NOT NULL,
        type VARCHAR(20) NOT NULL CHECK (type IN ('image', 'document')),
        file_path VARCHAR NOT NULL,
        thumbnail_path TEXT,
        original_filename VARCHAR NOT NULL,
        file_size BIGINT NOT NULL,
        mime_type VARCHAR(100) NOT NULL,
        uploaded_by VARCHAR NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `)

    // Create venue media table  
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS venue_media (
        id VARCHAR PRIMARY KEY,
        venue_id VARCHAR NOT NULL,
        type VARCHAR(20) NOT NULL CHECK (type IN ('image', 'document')),
        file_path VARCHAR NOT NULL,
        thumbnail_path TEXT,
        original_filename VARCHAR NOT NULL,
        file_size BIGINT NOT NULL,
        mime_type VARCHAR(100) NOT NULL,
        uploaded_by VARCHAR NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `)

    // Create indexes for performance
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_event_media_event_id ON event_media(event_id)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_event_media_type ON event_media(type)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_event_media_uploaded_by ON event_media(uploaded_by)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_event_media_created_at ON event_media(created_at)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_venue_media_venue_id ON venue_media(venue_id)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_venue_media_type ON venue_media(type)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_venue_media_uploaded_by ON venue_media(uploaded_by)
    `)
    
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_venue_media_created_at ON venue_media(created_at)
    `)

    // Add media count columns to events table for quick statistics
    // (Note: We'll update these counts programmatically in the application)
    await this.executeSQL(`
      ALTER TABLE events ADD COLUMN IF NOT EXISTS media_count INTEGER DEFAULT 0
    `)
    
    await this.executeSQL(`
      ALTER TABLE events ADD COLUMN IF NOT EXISTS image_count INTEGER DEFAULT 0
    `)
    
    await this.executeSQL(`
      ALTER TABLE events ADD COLUMN IF NOT EXISTS document_count INTEGER DEFAULT 0
    `)

    // Add media count columns to venues table
    await this.executeSQL(`
      ALTER TABLE venues ADD COLUMN IF NOT EXISTS media_count INTEGER DEFAULT 0
    `)
    
    await this.executeSQL(`
      ALTER TABLE venues ADD COLUMN IF NOT EXISTS image_count INTEGER DEFAULT 0
    `)

    logger.info('✅ Created media management system tables and indexes')
  }

  async down(): Promise<void> {
    // Remove added columns from events table
    await this.executeSQL(`ALTER TABLE events DROP COLUMN IF EXISTS media_count`)
    await this.executeSQL(`ALTER TABLE events DROP COLUMN IF EXISTS image_count`)
    await this.executeSQL(`ALTER TABLE events DROP COLUMN IF EXISTS document_count`)
    
    // Remove added columns from venues table
    await this.executeSQL(`ALTER TABLE venues DROP COLUMN IF EXISTS media_count`)
    await this.executeSQL(`ALTER TABLE venues DROP COLUMN IF EXISTS image_count`)

    // Drop indexes
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_media_event_id`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_media_type`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_media_uploaded_by`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_media_created_at`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_venue_media_venue_id`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_venue_media_type`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_venue_media_uploaded_by`)
    await this.executeSQL(`DROP INDEX IF EXISTS idx_venue_media_created_at`)

    // Drop tables
    await this.executeSQL(`DROP TABLE IF EXISTS venue_media`)
    await this.executeSQL(`DROP TABLE IF EXISTS event_media`)

    logger.info('✅ Removed media management system tables and indexes')
  }
}