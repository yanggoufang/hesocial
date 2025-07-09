-- HeSocial Platform Database Schema for DuckDB
-- High-end social event platform for affluent individuals

-- Server state tracking table
CREATE TABLE IF NOT EXISTS server_state (
    id INTEGER PRIMARY KEY,
    start_count INTEGER NOT NULL DEFAULT 0,
    first_start_time TIMESTAMP,
    last_start_time TIMESTAMP,
    last_stop_time TIMESTAMP,
    last_session_duration INTEGER DEFAULT 0, -- in seconds
    total_lifetime INTEGER DEFAULT 0 -- in seconds
);

-- Users table with luxury membership tiers
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    age INTEGER NOT NULL CHECK (age >= 18 AND age <= 100),
    profession VARCHAR(200) NOT NULL,
    annual_income BIGINT NOT NULL CHECK (annual_income >= 5000000), -- NT$5M minimum
    net_worth BIGINT NOT NULL CHECK (net_worth >= 30000000), -- NT$30M minimum
    membership_tier VARCHAR(20) NOT NULL CHECK (membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
    privacy_level INTEGER NOT NULL DEFAULT 3 CHECK (privacy_level >= 1 AND privacy_level <= 5),
    is_verified BOOLEAN DEFAULT FALSE,
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (verification_status IN ('pending', 'approved', 'rejected')),
    profile_picture TEXT,
    bio TEXT,
    interests VARCHAR[],
    stripe_customer_id VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP
);

-- Financial verification table
CREATE TABLE IF NOT EXISTS financial_verifications (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    income_proof_url TEXT NOT NULL,
    asset_documents JSON NOT NULL DEFAULT '{}',
    verification_date TIMESTAMP,
    verified_by INTEGER,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    notes TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- OAuth providers table
CREATE TABLE IF NOT EXISTS oauth_providers (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    provider VARCHAR(50) NOT NULL, -- 'google', 'linkedin'
    provider_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_id)
);

-- Venues table for luxury locations
CREATE TABLE IF NOT EXISTS venues (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    amenities VARCHAR[],
    images VARCHAR[],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Event categories
CREATE TABLE IF NOT EXISTS event_categories (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    icon VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Events table for luxury social events
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    date_time TIMESTAMP NOT NULL,
    registration_deadline TIMESTAMP NOT NULL,
    venue_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    organizer_id INTEGER NOT NULL,
    pricing JSON NOT NULL DEFAULT '{}', -- {vip: 10000, vvip: 15000, currency: "TWD", installmentOptions: [6,12,24]}
    exclusivity_level VARCHAR(20) NOT NULL CHECK (exclusivity_level IN ('VIP', 'VVIP', 'Invitation Only')),
    dress_code INTEGER NOT NULL CHECK (dress_code >= 1 AND dress_code <= 5),
    capacity INTEGER NOT NULL CHECK (capacity > 0),
    current_attendees INTEGER DEFAULT 0 CHECK (current_attendees >= 0),
    amenities VARCHAR[],
    privacy_guarantees VARCHAR[],
    images VARCHAR[],
    video_url TEXT,
    requirements JSON DEFAULT '[]', -- [{type: "age", value: "45-65", description: "Age requirement"}]
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CHECK (registration_deadline < date_time)
);

-- Event registrations
CREATE TABLE IF NOT EXISTS registrations (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    event_id INTEGER NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled')),
    payment_status VARCHAR(20) DEFAULT 'pending' CHECK (payment_status IN ('pending', 'paid', 'refunded')),
    payment_intent_id VARCHAR(255),
    special_requests TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, event_id)
);

-- User sessions for security
CREATE TABLE IF NOT EXISTS user_sessions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Audit log for security and compliance
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY,
    user_id INTEGER,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id INTEGER,
    details JSON DEFAULT '{}',
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

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