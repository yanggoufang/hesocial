import { BaseMigration } from './Migration.js';

export default class AddParticipantPrivacySystem extends BaseMigration {
  id = '006_add_participant_privacy_system';
  version = 6;
  name = 'Add Participant Privacy System';
  description = 'Add event participant privacy and access control tables for social networking features';
  category = 'schema' as const;
  override dependencies = ['4']; // Depends on migration version 4 (media management)

  async up(): Promise<void> {
    // Event privacy overrides table - allows users to set different privacy levels for specific events
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS event_privacy_overrides (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        event_id INTEGER NOT NULL,
        privacy_level INTEGER NOT NULL DEFAULT 3 CHECK (privacy_level >= 1 AND privacy_level <= 5),
        allow_contact BOOLEAN DEFAULT TRUE,
        show_in_list BOOLEAN DEFAULT TRUE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(user_id, event_id)
      );
    `);

    // Event participant access tracking table - tracks who can view participants
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS event_participant_access (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        event_id INTEGER NOT NULL,
        has_access BOOLEAN DEFAULT FALSE,
        access_granted_at TIMESTAMP,
        payment_status VARCHAR(20) DEFAULT 'pending',
        access_level VARCHAR(20) DEFAULT 'basic',
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        UNIQUE(user_id, event_id)
      );
    `);

    // Participant view logs table - security logging for participant viewing activities
    await this.executeSQL(`
      CREATE TABLE IF NOT EXISTS participant_view_logs (
        id INTEGER PRIMARY KEY,
        viewer_id INTEGER NOT NULL,
        participant_id INTEGER NOT NULL,
        event_id INTEGER NOT NULL,
        access_level INTEGER NOT NULL,
        ip_address VARCHAR(45),
        user_agent TEXT,
        viewed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Add indexes for performance
    await this.executeSQL(`
      CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_user_event ON event_privacy_overrides(user_id, event_id);
      CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_user_id ON event_privacy_overrides(user_id);
      CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_event_id ON event_privacy_overrides(event_id);
      CREATE INDEX IF NOT EXISTS idx_event_participant_access_user_event ON event_participant_access(user_id, event_id);
      CREATE INDEX IF NOT EXISTS idx_event_participant_access_event_id ON event_participant_access(event_id);
      CREATE INDEX IF NOT EXISTS idx_event_participant_access_payment_status ON event_participant_access(payment_status);
      CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewer ON participant_view_logs(viewer_id);
      CREATE INDEX IF NOT EXISTS idx_participant_view_logs_participant ON participant_view_logs(participant_id);
      CREATE INDEX IF NOT EXISTS idx_participant_view_logs_event ON participant_view_logs(event_id);
      CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewed_at ON participant_view_logs(viewed_at);
    `);

    // Add foreign key constraints (DuckDB compatible)
    await this.executeSQL(`
      -- Note: DuckDB doesn't fully support foreign key constraints like PostgreSQL,
      -- but these references are documented for data integrity
      -- The application layer should enforce these relationships
    `);
  }

  async down(): Promise<void> {
    // Drop indexes first
    await this.executeSQL(`DROP INDEX IF EXISTS idx_participant_view_logs_viewed_at;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_participant_view_logs_event;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_participant_view_logs_participant;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_participant_view_logs_viewer;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_participant_access_payment_status;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_participant_access_event_id;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_participant_access_user_event;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_privacy_overrides_event_id;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_privacy_overrides_user_id;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_event_privacy_overrides_user_event;`);

    // Drop tables in reverse order
    await this.executeSQL(`DROP TABLE IF EXISTS participant_view_logs;`);
    await this.executeSQL(`DROP TABLE IF EXISTS event_participant_access;`);
    await this.executeSQL(`DROP TABLE IF EXISTS event_privacy_overrides;`);
  }
}