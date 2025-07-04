import { BaseMigration } from './Migration.js';

export default class EnhanceUserProfile extends BaseMigration {
  id = '002_enhance_user_profile';
  version = 2;
  name = 'Enhance User Profile';
  description = 'Add additional profile fields for enhanced user experience';
  category = 'schema' as const;
  override dependencies = ['1']; // Depends on migration version 1

  async up(): Promise<void> {
    // Add new columns to users table
    await this.executeSQL(`
      ALTER TABLE users 
      ADD COLUMN phone_number VARCHAR(20),
      ADD COLUMN date_of_birth DATE,
      ADD COLUMN linkedin_profile VARCHAR(255),
      ADD COLUMN company VARCHAR(255),
      ADD COLUMN job_title VARCHAR(255),
      ADD COLUMN preferred_language VARCHAR(10) DEFAULT 'en',
      ADD COLUMN timezone VARCHAR(50) DEFAULT 'UTC',
      ADD COLUMN profile_visibility VARCHAR(20) DEFAULT 'members' 
        CHECK (profile_visibility IN ('public', 'members', 'private')),
      ADD COLUMN email_notifications BOOLEAN DEFAULT true,
      ADD COLUMN push_notifications BOOLEAN DEFAULT true,
      ADD COLUMN marketing_emails BOOLEAN DEFAULT false,
      ADD COLUMN last_login_at TIMESTAMP,
      ADD COLUMN login_count INTEGER DEFAULT 0;
    `);

    // Create user preferences table
    await this.executeSQL(`
      CREATE TABLE user_preferences (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        preference_key VARCHAR(100) NOT NULL,
        preference_value TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        UNIQUE(user_id, preference_key)
      );
    `);

    // Create indexes for better performance
    await this.executeSQL(`
      CREATE INDEX idx_user_preferences_user_id 
      ON user_preferences(user_id);
    `);

    await this.executeSQL(`
      CREATE INDEX idx_users_last_login 
      ON users(last_login_at);
    `);

    await this.executeSQL(`
      CREATE INDEX idx_users_company 
      ON users(company);
    `);

    // Add some default preferences for existing users
    await this.executeSQL(`
      INSERT INTO user_preferences (user_id, preference_key, preference_value)
      SELECT id, 'theme', 'dark' FROM users WHERE id IS NOT NULL;
    `);

    await this.executeSQL(`
      INSERT INTO user_preferences (user_id, preference_key, preference_value)
      SELECT id, 'dashboard_layout', 'grid' FROM users WHERE id IS NOT NULL;
    `);
  }

  async down(): Promise<void> {
    // Drop indexes
    await this.executeSQL(`DROP INDEX IF EXISTS idx_users_company;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_users_last_login;`);
    await this.executeSQL(`DROP INDEX IF EXISTS idx_user_preferences_user_id;`);

    // Drop the preferences table
    await this.executeSQL(`DROP TABLE IF EXISTS user_preferences;`);

    // Remove added columns from users table
    await this.executeSQL(`
      ALTER TABLE users 
      DROP COLUMN IF EXISTS phone_number,
      DROP COLUMN IF EXISTS date_of_birth,
      DROP COLUMN IF EXISTS linkedin_profile,
      DROP COLUMN IF EXISTS company,
      DROP COLUMN IF EXISTS job_title,
      DROP COLUMN IF EXISTS preferred_language,
      DROP COLUMN IF EXISTS timezone,
      DROP COLUMN IF EXISTS profile_visibility,
      DROP COLUMN IF EXISTS email_notifications,
      DROP COLUMN IF EXISTS push_notifications,
      DROP COLUMN IF EXISTS marketing_emails,
      DROP COLUMN IF EXISTS last_login_at,
      DROP COLUMN IF EXISTS login_count;
    `);
  }
}