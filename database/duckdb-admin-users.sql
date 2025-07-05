-- Admin Users and Sample Data for HeSocial Platform
-- This file adds admin capabilities and default users for development and testing

-- Add admin role column to users table (if not exists)
-- Note: In production, create a migration for this
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(20) DEFAULT 'user' CHECK (role IN ('user', 'admin', 'super_admin'));

-- Default admin user for system administration
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession, 
    annual_income, net_worth, membership_tier, privacy_level, 
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    1000, 
    'admin@hesocial.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: admin123
    'System',
    'Administrator',
    45,
    'Platform Administrator',
    5000000,
    30000000,
    'Black Card',
    5,
    true,
    'approved',
    'super_admin',
    'HeSocial platform system administrator with full access to all features and user management capabilities.',
    ['platform management', 'user verification', 'system monitoring', 'data analytics'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Super admin user for development
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    1001,
    'superadmin@hesocial.com', 
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: admin123
    'Super',
    'Admin',
    40,
    'System Developer',
    5000000,
    30000000,
    'Black Card',
    5,
    true,
    'approved',
    'super_admin',
    'Platform super administrator for development and system maintenance.',
    ['system development', 'database management', 'security auditing', 'feature development'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Content admin for event management
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    1002,
    'events@hesocial.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: admin123
    'Event',
    'Manager',
    42,
    'Event Coordinator',
    5000000,
    30000000,
    'Diamond',
    3,
    true,
    'approved',
    'admin',
    'Specialized administrator for luxury event planning and member experience management.',
    ['luxury events', 'hospitality', 'member relations', 'venue management'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Sample test users for different membership tiers

-- Test user: Platinum tier
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    2001,
    'test.platinum@example.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: test123
    'Test',
    'Platinum',
    45,
    'Business Owner',
    8000000,
    50000000,
    'Platinum',
    3,
    true,
    'approved',
    'user',
    'Test user account for Platinum tier membership testing and development.',
    ['business development', 'networking', 'fine dining', 'travel'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Test user: Diamond tier
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    2002,
    'test.diamond@example.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: test123
    'Test',
    'Diamond',
    50,
    'Investment Manager',
    15000000,
    120000000,
    'Diamond',
    4,
    true,
    'approved',
    'user',
    'Test user account for Diamond tier membership testing and premium feature validation.',
    ['investment strategy', 'luxury lifestyle', 'art collection', 'private events'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Test user: Black Card tier
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    2003,
    'test.blackcard@example.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: test123
    'Test',
    'BlackCard',
    55,
    'Private Equity Partner',
    50000000,
    500000000,
    'Black Card',
    5,
    true,
    'approved',
    'user',
    'Test user account for Black Card tier membership testing and exclusive feature validation.',
    ['private equity', 'exclusive events', 'yacht ownership', 'philanthrophy'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Test user: Pending verification
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    created_at, updated_at
) VALUES (
    2004,
    'test.pending@example.com',
    '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', -- Password: test123
    'Test',
    'Pending',
    42,
    'Startup Founder',
    6000000,
    40000000,
    'Platinum',
    2,
    false,
    'pending',
    'user',
    'Test user account for testing the verification process and pending state handling.',
    ['startup ecosystem', 'technology', 'innovation', 'venture capital'],
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Test user: OAuth user (Google login simulation)
INSERT OR REPLACE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    profile_picture, created_at, updated_at
) VALUES (
    2005,
    'test.oauth@gmail.com',
    '', -- No password for OAuth users
    'OAuth',
    'TestUser',
    NULL, -- Needs to complete profile
    NULL, -- Needs to complete profile
    NULL, -- Needs to complete profile
    NULL, -- Needs to complete profile
    'Platinum',
    3,
    false,
    'pending',
    'user',
    'Test OAuth user created via Google authentication - profile needs completion.',
    ['social networking', 'digital lifestyle'],
    'https://lh3.googleusercontent.com/a/test-profile-photo',
    CURRENT_TIMESTAMP,
    CURRENT_TIMESTAMP
);

-- Create admin session for initial login tracking
INSERT OR REPLACE INTO server_state (id, start_count, first_start_time, last_start_time) 
VALUES (1, 0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);

-- Summary comment
-- 
-- DEFAULT USERS CREATED:
-- =====================
-- 
-- ADMIN ACCOUNTS:
-- - admin@hesocial.com (Password: admin123) - Super Admin
-- - superadmin@hesocial.com (Password: admin123) - Super Admin  
-- - events@hesocial.com (Password: admin123) - Event Admin
--
-- TEST ACCOUNTS:
-- - test.platinum@example.com (Password: test123) - Platinum Member
-- - test.diamond@example.com (Password: test123) - Diamond Member
-- - test.blackcard@example.com (Password: test123) - Black Card Member
-- - test.pending@example.com (Password: test123) - Pending Verification
-- - test.oauth@gmail.com (No password) - OAuth Test User
--
-- All users are pre-verified except the pending test user and OAuth user
-- Admin users have Black Card membership with full platform access