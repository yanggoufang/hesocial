-- Migration: Add Participant Access Control System
-- Description: Adds tables and columns for controlling participant viewing permissions based on payment status and privacy levels
-- Date: 2024-12-08

-- Add default privacy level and participant visibility settings to users table
ALTER TABLE users ADD COLUMN default_privacy_level INTEGER DEFAULT 2 CHECK (default_privacy_level BETWEEN 1 AND 5);
ALTER TABLE users ADD COLUMN show_in_participant_lists BOOLEAN DEFAULT true;
ALTER TABLE users ADD COLUMN allow_contact_requests BOOLEAN DEFAULT true;

-- Event Participant Access Control
-- Tracks who has paid access to view participants for specific events
CREATE TABLE IF NOT EXISTS event_participant_access (
  id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  event_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  payment_status TEXT NOT NULL CHECK (payment_status IN ('pending', 'paid', 'refunded')),
  access_level INTEGER NOT NULL DEFAULT 0,
  registration_id TEXT, -- Link to the registration record
  granted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP, -- For future time-limited access
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (registration_id) REFERENCES registrations(id) ON DELETE SET NULL,
  UNIQUE(event_id, user_id)
);

-- Participant View Logs
-- Track when users view participant information for analytics and abuse prevention
CREATE TABLE IF NOT EXISTS participant_view_logs (
  id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  viewer_id TEXT NOT NULL,
  viewed_participant_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  view_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  view_type TEXT NOT NULL CHECK (view_type IN ('list', 'profile', 'contact')),
  ip_address TEXT,
  user_agent TEXT,
  FOREIGN KEY (viewer_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (viewed_participant_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

-- Event Privacy Overrides
-- Allow users to set different privacy levels for specific events
CREATE TABLE IF NOT EXISTS event_privacy_overrides (
  id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
  user_id TEXT NOT NULL,
  event_id TEXT NOT NULL,
  privacy_level INTEGER NOT NULL CHECK (privacy_level BETWEEN 1 AND 5),
  allow_contact BOOLEAN DEFAULT true,
  show_in_list BOOLEAN DEFAULT true,
  valid_until TIMESTAMP,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
  UNIQUE(user_id, event_id)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_event_participant_access_event_user ON event_participant_access(event_id, user_id);
CREATE INDEX IF NOT EXISTS idx_event_participant_access_payment_status ON event_participant_access(payment_status);
CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewer ON participant_view_logs(viewer_id, event_id);
CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewed ON participant_view_logs(viewed_participant_id, event_id);
CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_user_event ON event_privacy_overrides(user_id, event_id);

-- Insert default privacy settings for existing users
UPDATE users 
SET default_privacy_level = 2, 
    show_in_participant_lists = true, 
    allow_contact_requests = true 
WHERE default_privacy_level IS NULL;

-- Create participant access records for existing registrations with paid status
INSERT INTO event_participant_access (event_id, user_id, payment_status, access_level, registration_id, granted_at)
SELECT 
  r.event_id,
  r.user_id,
  r.payment_status,
  CASE 
    WHEN r.payment_status = 'paid' THEN 3
    WHEN r.payment_status = 'pending' THEN 1
    ELSE 0
  END as access_level,
  r.id as registration_id,
  r.updated_at as granted_at
FROM registrations r
WHERE r.status != 'cancelled'
ON CONFLICT(event_id, user_id) DO UPDATE SET
  payment_status = excluded.payment_status,
  access_level = excluded.access_level,
  updated_at = CURRENT_TIMESTAMP;