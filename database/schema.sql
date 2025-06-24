-- HeSocial Platform Database Schema
-- High-end social event platform for affluent individuals

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table with luxury membership tiers
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
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
    interests TEXT[] DEFAULT '{}',
    stripe_customer_id VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Financial verification table
CREATE TABLE financial_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    income_proof_url TEXT NOT NULL,
    asset_documents JSONB NOT NULL DEFAULT '{}',
    verification_date TIMESTAMP WITH TIME ZONE,
    verified_by UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- OAuth providers table
CREATE TABLE oauth_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- 'google', 'linkedin'
    provider_id VARCHAR(255) NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(provider, provider_id)
);

-- Venues table for luxury locations
CREATE TABLE venues (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    amenities TEXT[] DEFAULT '{}',
    images TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Event categories
CREATE TABLE event_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    icon VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Events table for luxury social events
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    date_time TIMESTAMP WITH TIME ZONE NOT NULL,
    registration_deadline TIMESTAMP WITH TIME ZONE NOT NULL,
    venue_id UUID NOT NULL REFERENCES venues(id),
    category_id UUID NOT NULL REFERENCES event_categories(id),
    organizer_id UUID NOT NULL REFERENCES users(id),
    pricing JSONB NOT NULL DEFAULT '{}', -- {vip: 10000, vvip: 15000, currency: "TWD", installmentOptions: [6,12,24]}
    exclusivity_level VARCHAR(20) NOT NULL CHECK (exclusivity_level IN ('VIP', 'VVIP', 'Invitation Only')),
    dress_code INTEGER NOT NULL CHECK (dress_code >= 1 AND dress_code <= 5),
    capacity INTEGER NOT NULL CHECK (capacity > 0),
    current_attendees INTEGER DEFAULT 0 CHECK (current_attendees >= 0),
    amenities TEXT[] DEFAULT '{}',
    privacy_guarantees TEXT[] DEFAULT '{}',
    images TEXT[] DEFAULT '{}',
    video_url TEXT,
    requirements JSONB DEFAULT '[]', -- [{type: "age", value: "45-65", description: "Age requirement"}]
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_registration_deadline CHECK (registration_deadline < date_time)
);

-- Event registrations
CREATE TABLE registrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'cancelled')),
    payment_status VARCHAR(20) DEFAULT 'pending' CHECK (payment_status IN ('pending', 'paid', 'refunded')),
    payment_intent_id VARCHAR(255),
    special_requests TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, event_id)
);

-- User sessions for security
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Audit log for security and compliance
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_membership_tier ON users(membership_tier);
CREATE INDEX idx_users_verification_status ON users(verification_status);
CREATE INDEX idx_users_deleted_at ON users(deleted_at);

CREATE INDEX idx_events_date_time ON events(date_time);
CREATE INDEX idx_events_venue_id ON events(venue_id);
CREATE INDEX idx_events_category_id ON events(category_id);
CREATE INDEX idx_events_exclusivity_level ON events(exclusivity_level);
CREATE INDEX idx_events_is_active ON events(is_active);

CREATE INDEX idx_registrations_user_id ON registrations(user_id);
CREATE INDEX idx_registrations_event_id ON registrations(event_id);
CREATE INDEX idx_registrations_status ON registrations(status);
CREATE INDEX idx_registrations_payment_status ON registrations(payment_status);

CREATE INDEX idx_financial_verifications_user_id ON financial_verifications(user_id);
CREATE INDEX idx_financial_verifications_status ON financial_verifications(status);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);

CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_events_updated_at BEFORE UPDATE ON events FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_venues_updated_at BEFORE UPDATE ON venues FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_event_categories_updated_at BEFORE UPDATE ON event_categories FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_registrations_updated_at BEFORE UPDATE ON registrations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_financial_verifications_updated_at BEFORE UPDATE ON financial_verifications FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to update event attendee count
CREATE OR REPLACE FUNCTION update_event_attendee_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.status = 'approved' THEN
        UPDATE events 
        SET current_attendees = current_attendees + 1 
        WHERE id = NEW.event_id;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.status != 'approved' AND NEW.status = 'approved' THEN
            UPDATE events 
            SET current_attendees = current_attendees + 1 
            WHERE id = NEW.event_id;
        ELSIF OLD.status = 'approved' AND NEW.status != 'approved' THEN
            UPDATE events 
            SET current_attendees = current_attendees - 1 
            WHERE id = NEW.event_id;
        END IF;
    ELSIF TG_OP = 'DELETE' AND OLD.status = 'approved' THEN
        UPDATE events 
        SET current_attendees = current_attendees - 1 
        WHERE id = OLD.event_id;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ language 'plpgsql';

CREATE TRIGGER update_event_attendee_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON registrations
    FOR EACH ROW EXECUTE FUNCTION update_event_attendee_count();