import { Migration } from './Migration';
import { duckdb } from '../duckdb-connection.js';

export class RefactorEventMedia implements Migration {
  public id = '005';
  public name = '005_refactor_event_media';
  public version = 5;
  public description = 'Refactors event media to a dedicated table';

  public async up(): Promise<void> {
    await duckdb.query(`
      CREATE TABLE event_media (
        id VARCHAR PRIMARY KEY,
        event_id VARCHAR NOT NULL REFERENCES events(id) ON DELETE CASCADE,
        source_type VARCHAR NOT NULL CHECK (source_type IN ('upload', 'url')),
        file_path_or_url VARCHAR NOT NULL,
        type VARCHAR NOT NULL CHECK (type IN ('image', 'document')),
        mime_type VARCHAR,
        uploaded_by VARCHAR REFERENCES users(id) ON DELETE SET NULL,
        sort_order INTEGER DEFAULT 0,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
      );
    `);

    // Note: In a production environment, we would migrate existing data from events.images here.
    // For development, we will start fresh.

    await duckdb.query(`ALTER TABLE events DROP COLUMN images;`);
  }

  public async down(): Promise<void> {
    await duckdb.query(`ALTER TABLE events ADD COLUMN images JSON;`);
    await duckdb.query(`DROP TABLE event_media;`);
  }
}
