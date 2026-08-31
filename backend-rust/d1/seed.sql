-- HeSocial D1 seed - Rust migration Phase 0.5
-- Idempotent: INSERT OR IGNORE everywhere, safe to re-run.
-- Convention: no ';' inside string literals — the contract-test loader
-- (backend/test/contract/rust.setup.ts) splits statements on ';'.
--
-- Accounts use the EXACT bcrypt hashes from
-- backend/src/database/duckdb-connection.ts (ensureSeedUsers):
--   $2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6 = admin123
--   $2a$10$bt0AdKVHTbGLIwN44tp6dO9xMCf8vh2FSFje7iFt72zCfMgS0g6TK = test123
--
-- No visitor/analytics rows: locked decision #3 keeps those out of D1.

INSERT OR IGNORE INTO server_state (id, start_count, first_start_time, last_start_time, last_session_duration, total_lifetime)
VALUES (1, 1, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z', 0, 0);

INSERT OR IGNORE INTO users (
    id, email, password_hash, first_name, last_name, age, profession,
    annual_income, net_worth, membership_tier, privacy_level,
    is_verified, verification_status, role, bio, interests,
    email_notifications, push_notifications, marketing_emails,
    login_count, created_at, updated_at
) VALUES
    (
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        'admin@hesocial.com',
        '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6',
        'Admin', 'User', 40, 'System Administrator',
        5000000, 30000000, 'Black Card', 5,
        1, 'approved', 'super_admin',
        'Platform super administrator for development and system maintenance.',
        '["system administration"]',
        1, 1, 0,
        0, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'
    ),
    (
        '9c858901-8a57-4791-81fe-4c455b099bc9',
        'test.platinum@example.com',
        '$2a$10$bt0AdKVHTbGLIwN44tp6dO9xMCf8vh2FSFje7iFt72zCfMgS0g6TK',
        'Test', 'Platinum', 45, 'Business Owner',
        8000000, 50000000, 'Platinum', 3,
        1, 'approved', 'user',
        'Test user account for Platinum tier membership testing and development.',
        '["business development", "networking", "fine dining", "travel"]',
        1, 1, 0,
        0, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'
    );

INSERT OR IGNORE INTO venues (
    id, name, description, address, city, district, venue_type,
    capacity_min, capacity_max, price_tier, amenities, latitude, longitude, rating,
    is_active, created_at, updated_at
) VALUES
    (
        1, 'Taipei Private Dining Room', 'Michelin-led private dining suite.',
        'No. 101, Dunhua S. Rd, Da''an District', 'Taipei', 'Da''an', 'restaurant',
        6, 20, 'ultra_luxury', '["valet", "wine_cellar", "private_chef"]',
        25.03300000, 121.56540000, 5,
        1, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'
    ),
    (
        2, 'Keelung Luxury Yacht', '60-ft motor yacht for private charters.',
        'Keelung Harbor Pier 8', 'Keelung', 'Zhongzheng', 'yacht',
        10, 30, 'luxury', '["parking", "security"]',
        25.13000000, 121.73900000, 4,
        1, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'
    );

INSERT OR IGNORE INTO event_categories (
    id, name, slug, description, icon, color, target_membership_tiers,
    typical_duration_hours, typical_capacity, is_active, sort_order, created_at, updated_at
) VALUES
    (1, '私人晚宴', 'dinner', '獨家私人晚宴體驗，由米其林主廚精心設計', 'chef-hat', '#D4AF37', '["Platinum", "Diamond", "Black Card"]', 4, '{"min": 6, "max": 12}', 1, 1, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'),
    (2, '遊艇派對', 'yacht', '豪華遊艇上的頂級社交聚會', 'anchor', '#1E40AF', '["Diamond", "Black Card"]', 6, '{"min": 10, "max": 30}', 1, 2, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'),
    (3, '藝術沙龍', 'art', '私人畫廊與藝術收藏品鑑會', 'palette', '#7C2D12', '["Platinum", "Diamond", "Black Card"]', 3, '{"min": 8, "max": 15}', 1, 3, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z'),
    (4, '商務社交', 'business', '高淨值專業人士的商務交流場合', 'briefcase', '#334155', '["Platinum", "Diamond", "Black Card"]', 2, '{"min": 15, "max": 40}', 1, 4, '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z');

INSERT OR IGNORE INTO events (
    id, title, slug, description, detailed_description,
    category_id, venue_id, organizer_id,
    start_datetime, end_datetime, timezone,
    capacity_min, capacity_max, current_registrations,
    price_platinum, price_diamond, price_black_card, currency,
    status, approval_status, approved_by, approved_at,
    required_membership_tiers, required_verification, age_restriction,
    dress_code, language, special_requirements, inclusions, exclusions,
    registration_opens_at, registration_closes_at, cancellation_deadline,
    waitlist_enabled, auto_approval,
    meta_title, meta_description, featured_image, gallery_images,
    internal_notes, cost_breakdown, profit_margin,
    created_at, updated_at, published_at
) VALUES
    (
        1, 'Michelin Private Dinner', 'michelin-private-dinner-2026-09',
        'Eight-seat tasting menu with a two-star Michelin chef.',
        'A closed-door tasting menu. Guests are seated at a single counter.',
        1, 1, 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2026-09-18T18:30:00.000Z', '2026-09-18T22:00:00.000Z', 'Asia/Taipei',
        6, 12, 0,
        12000.00, 12000.00, 12000.00, 'TWD',
        'pending_review', 'pending', NULL, NULL,
        '["Platinum", "Diamond", "Black Card"]', 1, '{"min": 25, "max": null}',
        'Smart Casual', 'Traditional Chinese', NULL,
        '["meals", "beverages"]', '["transportation"]',
        '2026-08-30T00:00:00.000Z', '2026-09-15T23:59:59.000Z', '2026-09-16T23:59:59.000Z',
        1, 0,
        'Michelin Private Dinner', 'Eight-seat tasting menu in Taipei.', NULL, NULL,
        NULL, NULL, NULL,
        '2026-08-30T00:00:00.000Z', '2026-08-30T00:00:00.000Z', NULL
    ),
    (
        2, 'Autumn Yacht Social', 'autumn-yacht-social-2026-10',
        'Sunset cruise around Keelung Harbor with a curated guest list.',
        'Four hours on a chartered yacht, catering and sommelier included.',
        2, 2, 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2026-10-10T09:00:00.000Z', '2026-10-10T15:00:00.000Z', 'Asia/Taipei',
        10, 30, 1,
        18000.00, 18000.00, 18000.00, 'TWD',
        'published', 'approved', 'f47ac10b-58cc-4372-a567-0e02b2c3d479', '2026-08-30T01:00:00.000Z',
        '["Diamond", "Black Card"]', 1, '{"min": 30, "max": null}',
        'Resort Casual', 'Traditional Chinese', NULL,
        '["transportation", "meals", "beverages", "gifts"]', '["photography"]',
        '2026-08-30T00:00:00.000Z', '2026-10-05T23:59:59.000Z', '2026-10-07T23:59:59.000Z',
        1, 0,
        'Autumn Yacht Social', 'Sunset cruise around Keelung Harbor.', NULL, NULL,
        NULL, NULL, 12.50,
        '2026-08-30T00:00:00.000Z', '2026-08-30T01:00:00.000Z', '2026-08-30T01:00:00.000Z'
    ),
    (
        3, 'Emerging Collectors Salon', 'emerging-collectors-salon-2026-11',
        'Gallery walkthrough of emerging Taiwanese artists.',
        'A private salon with the gallery owner, followed by a closed bidding preview.',
        3, 1, 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2026-11-07T13:00:00.000Z', '2026-11-07T17:00:00.000Z', 'Asia/Taipei',
        8, 15, 0,
        8000.00, 8000.00, 8000.00, 'TWD',
        'draft', 'rejected', 'f47ac10b-58cc-4372-a567-0e02b2c3d479', '2026-08-30T02:00:00.000Z',
        '["Platinum", "Diamond", "Black Card"]', 1, '{"min": 25, "max": null}',
        'Smart Casual', 'Traditional Chinese', NULL,
        '["meals"]', '["transportation"]',
        NULL, '2026-11-01T23:59:59.000Z', '2026-11-03T23:59:59.000Z',
        1, 0,
        'Emerging Collectors Salon', 'Gallery walkthrough of emerging artists.', NULL, NULL,
        'Rejected: venue insurance certificate missing.', NULL, NULL,
        '2026-08-30T00:00:00.000Z', '2026-08-30T02:00:00.000Z', NULL
    );

INSERT OR IGNORE INTO registrations (
    id, event_id, user_id, status, registration_type, guest_count,
    price_paid, currency, payment_status,
    confirmation_sent, reminder_sent, feedback_collected,
    created_at, updated_at
) VALUES
    (
        1, 2, '9c858901-8a57-4791-81fe-4c455b099bc9',
        'pending', 'member', 0,
        NULL, 'TWD', 'paid',
        0, 0, 0,
        '2026-08-30T03:00:00.000Z', '2026-08-30T03:00:00.000Z'
    );

-- Phase 2e participant-privacy fixture. The Platinum viewer has a paid
-- registration-backed access row. The administrator is the visible level-1
-- participant, so contracts exercise masking without weakening user defaults.
INSERT OR IGNORE INTO event_participant_access (
    id, user_id, event_id, registration_id, has_access, access_granted_at,
    payment_status, access_level, created_at, updated_at
) VALUES
    (
        1, '9c858901-8a57-4791-81fe-4c455b099bc9', 2, 1, 1,
        '2026-08-30T03:05:00.000Z', 'paid', '3',
        '2026-08-30T03:05:00.000Z', '2026-08-30T03:05:00.000Z'
    ),
    (
        2, 'f47ac10b-58cc-4372-a567-0e02b2c3d479', 2, NULL, 1,
        '2026-08-30T03:05:00.000Z', 'paid', '4',
        '2026-08-30T03:05:00.000Z', '2026-08-30T03:05:00.000Z'
    );

INSERT OR IGNORE INTO event_privacy_overrides (
    id, user_id, event_id, privacy_level, allow_contact, show_in_list, created_at, updated_at
) VALUES (
    1, 'f47ac10b-58cc-4372-a567-0e02b2c3d479', 2, 1, 1, 1,
    '2026-08-30T03:05:00.000Z', '2026-08-30T03:05:00.000Z'
);

-- Phase 2f sales CRM fixture. Lead 9001 sits inside the current reporting
-- window, 9002 is deliberately outside every period bucket, and 9003 is
-- childless so the delete route can run without cascading. Lead 9004 is the
-- update target (DuckDB cannot update a parent row that still has children,
-- which is also why opportunity 9103 carries no activity row).
-- Stage 9405 and team member 9302 are inactive and must be filtered out by the
-- read routes.
INSERT OR IGNORE INTO sales_leads (
    id, first_name, last_name, email, phone, company, job_title,
    annual_income, net_worth, source, referral_code, lead_score, status,
    interested_membership_tier, budget_range, timeline, pain_points, interests,
    notes, last_contact_date, next_follow_up_date, assigned_to, created_at, updated_at
) VALUES
    (
        9001, 'Seeded', 'Contract', 'crm-active@hesocial.test', '+886900000001',
        'Contract Holdings', 'Principal', 25000000, 120000000, 'referral', 'CRM2F',
        100, 'new', 'Black Card', '5-10M', 'this-quarter', 'Discreet networking',
        '["fine dining","yachting"]', 'Active contract lead',
        NULL, '2026-09-07', 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        9002, 'Legacy', 'Won', 'crm-won@hesocial.test', '+886900000002',
        'Founding Member Co', 'Chair', 40000000, 200000000, 'event', NULL,
        100, 'closed_won', 'Black Card', '10M+', 'closed', 'Privacy',
        '["art", "yachting"]', 'Historical closed-won lead outside the window',
        '2020-01-15', NULL, 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2020-01-01T00:00:00.000Z', '2020-01-20T00:00:00.000Z'
    ),
    (
        9003, 'Deletable', 'Row', 'crm-deletable@hesocial.test', NULL,
        NULL, NULL, NULL, NULL, 'website', NULL,
        0, 'new', NULL, NULL, NULL, NULL,
        '[]', 'Childless row reserved for the delete route',
        NULL, NULL, NULL,
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        9004, 'Updatable', 'Target', 'crm-updatable@hesocial.test', '+886900000004',
        'Renewal Co', 'Director', 8000000, 40000000, 'website', NULL,
        40, 'contacted', 'Platinum', '1-5M', 'next-quarter', 'Sparse network',
        '["networking"]', 'Childless row reserved for the update route',
        NULL, NULL, 'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

INSERT OR IGNORE INTO sales_opportunities (
    id, lead_id, name, description, stage, probability, value,
    expected_close_date, actual_close_date, membership_tier, payment_terms,
    close_reason, assigned_to, created_at, updated_at
) VALUES
    (
        9101, 9002, 'Legacy Black Card Founding Seat',
        'Closed-won historical deal outside the reporting window',
        'closed_won', 100, 250000, '2020-02-01', '2020-01-20',
        'Black Card', 'annual-prepaid', 'Signed',
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2020-01-01T00:00:00.000Z', '2020-01-20T00:00:00.000Z'
    ),
    (
        9102, 9001, 'Diamond Membership Renewal',
        'Open deal the contract reads through the stage filter',
        'proposal', 60, 480000, '2026-12-01', NULL,
        'Diamond', 'semi-annual', NULL,
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    ),
    (
        9103, 9002, 'Negotiation Seat (childless)',
        'Open deal with no logged activity, reserved for the stage-transition test',
        'negotiation', 80, 120000, '2026-10-31', NULL,
        'Platinum', 'one-time', NULL,
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

INSERT OR IGNORE INTO sales_activities (
    id, lead_id, opportunity_id, activity_type, subject, description, outcome,
    duration_minutes, scheduled_at, completed_at, created_by, created_at, updated_at
) VALUES
    (
        9201, 9002, 9101, 'meeting', 'Founding seat presentation',
        'Historical close meeting', 'signed', 60,
        '2020-01-15T02:00:00.000Z', '2020-01-15T03:00:00.000Z',
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        '2020-01-15T03:00:00.000Z', '2020-01-15T03:00:00.000Z'
    ),
    (
        9202, 9001, 9102, 'call', 'Renewal discovery call',
        'Confirmed the Diamond tier budget', 'reached', 30,
        NULL, NULL,
        'f47ac10b-58cc-4372-a567-0e02b2c3d479',
        strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    );

INSERT OR IGNORE INTO sales_pipeline_stages (
    id, name, description, display_order, default_probability, is_active,
    color_code, created_at, updated_at
) VALUES
    (9401, 'qualification', 'Identify and validate the prospect', 1, 25, 1, '#94A3B8',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    (9402, 'needs_analysis', 'Document tier expectations and budget', 2, 40, 1, '#CBD5E1',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    (9403, 'proposal', 'Present the membership proposal', 3, 60, 1, '#F59E0B',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    (9404, 'negotiation', 'Negotiate terms and the close date', 4, 80, 1, '#10B981',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    (9405, 'archived_legacy', 'Retired stage that must stay hidden', 9, 0, 0, '#64748B',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT OR IGNORE INTO sales_team_members (
    id, user_id, role, territory, commission_rate, quota_amount, is_active,
    hire_date, manager_id, created_at, updated_at
) VALUES
    (9301, 'f47ac10b-58cc-4372-a567-0e02b2c3d479', 'sales_rep', 'Taipei', 8.50, 3000000, 1,
     '2024-03-01', '9c858901-8a57-4791-81fe-4c455b099bc9',
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    (9302, '9c858901-8a57-4791-81fe-4c455b099bc9', 'sales_manager', 'Kaohsiung', 12.00, 9000000, 0,
     '2022-06-01', NULL,
     strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
