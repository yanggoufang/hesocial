-- Migration: Add visitor tracking tables
-- Description: Create tables for anonymous visitor tracking and analytics
-- Version: 005
-- Created: 2025-07-08

-- Visitor sessions table - tracks unique visitors and their sessions
CREATE TABLE IF NOT EXISTS visitor_sessions (
  id INTEGER PRIMARY KEY,
  visitor_id VARCHAR(50) UNIQUE NOT NULL,
  user_id INTEGER NULL, -- Links to users table when visitor converts
  ip_address VARCHAR(45) NOT NULL,
  user_agent TEXT NOT NULL,
  referer TEXT NULL,
  first_seen TIMESTAMP NOT NULL,
  last_seen TIMESTAMP NOT NULL,
  page_views INTEGER DEFAULT 1,
  session_count INTEGER DEFAULT 1,
  converted_at TIMESTAMP NULL, -- When visitor became registered user
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Individual page views table - detailed tracking for analytics
CREATE TABLE IF NOT EXISTS visitor_page_views (
  id INTEGER PRIMARY KEY,
  visitor_id VARCHAR(50) NOT NULL,
  path VARCHAR(500) NOT NULL,
  method VARCHAR(10) NOT NULL DEFAULT 'GET',
  query_params TEXT NULL,
  referer TEXT NULL,
  timestamp TIMESTAMP NOT NULL,
  ip_address VARCHAR(45) NOT NULL,
  user_agent TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Visitor events table - track specific actions (future use)
CREATE TABLE IF NOT EXISTS visitor_events (
  id INTEGER PRIMARY KEY,
  visitor_id VARCHAR(50) NOT NULL,
  event_type VARCHAR(50) NOT NULL, -- 'page_view', 'event_view', 'registration_attempt', etc.
  event_data TEXT NULL, -- JSON data for event details
  timestamp TIMESTAMP NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_visitor_sessions_visitor_id ON visitor_sessions(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_sessions_user_id ON visitor_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_visitor_sessions_last_seen ON visitor_sessions(last_seen);
CREATE INDEX IF NOT EXISTS idx_visitor_page_views_visitor_id ON visitor_page_views(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_page_views_timestamp ON visitor_page_views(timestamp);
CREATE INDEX IF NOT EXISTS idx_visitor_page_views_path ON visitor_page_views(path);
CREATE INDEX IF NOT EXISTS idx_visitor_events_visitor_id ON visitor_events(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_events_type ON visitor_events(event_type);
CREATE INDEX IF NOT EXISTS idx_visitor_events_timestamp ON visitor_events(timestamp);

-- Analytics views for common queries
CREATE VIEW IF NOT EXISTS visitor_analytics_daily AS
SELECT 
  DATE(last_seen) as date,
  COUNT(DISTINCT visitor_id) as unique_visitors,
  SUM(page_views) as total_page_views,
  COUNT(DISTINCT CASE WHEN user_id IS NOT NULL THEN visitor_id END) as converted_visitors,
  AVG(page_views) as avg_pages_per_visitor
FROM visitor_sessions 
GROUP BY DATE(last_seen)
ORDER BY date DESC;

CREATE VIEW IF NOT EXISTS popular_pages AS
SELECT 
  vpv.path,
  COUNT(*) as views,
  COUNT(DISTINCT vpv.visitor_id) as unique_visitors,
  AVG(CASE WHEN vs.user_id IS NOT NULL THEN 1.0 ELSE 0.0 END) as conversion_rate
FROM visitor_page_views vpv
LEFT JOIN visitor_sessions vs ON vpv.visitor_id = vs.visitor_id
GROUP BY vpv.path
ORDER BY views DESC;
