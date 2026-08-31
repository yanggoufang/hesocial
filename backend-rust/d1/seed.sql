-- HeSocial D1 seed - Rust migration Phase 0.5
-- Idempotent: INSERT OR IGNORE everywhere, safe to re-run.
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
        NULL, 'TWD', 'pending',
        0, 0, 0,
        '2026-08-30T03:00:00.000Z', '2026-08-30T03:00:00.000Z'
    );
