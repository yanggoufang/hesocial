-- HeSocial D1 (SQLite) canonical schema - Rust migration Phase 0.5
-- Source of truth: docs/rust-migration/ROADMAP.md
--
-- Locked decisions applied here:
--   #1  users.password_algo routes bcrypt -> PBKDF2 lazy rehash on login
--   #2  events uses the unified event-management shape
--   #3  visitor_sessions / visitor_page_views / visitor_events are NOT here
--       (analytics -> Cloudflare Analytics Engine / KV in Phase 6)
--   --  dead tables omitted (0 code references): user_sessions, audit_logs,
--       oauth_providers, financial_verifications, event_feedback
--   --  `registrations` (never `event_registrations`)
--
-- Type mapping from the DuckDB sources:
--   VARCHAR(n) -> VARCHAR(n)   |  BIGINT -> BIGINT  |  DECIMAL(p,s) -> DECIMAL(p,s)
--   VARCHAR[]  -> TEXT (JSON array)             JSON -> TEXT
--   TIMESTAMP  -> TEXT (ISO-8601 UTC)           BOOLEAN -> INTEGER (0/1)
--   users.id   -> TEXT (UUID, matches the runtime); every FK into users is TEXT
--   every other PK -> INTEGER
--
-- SQLite accepts all of the type names above; nothing here is DuckDB-only.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS server_state (
    id INTEGER PRIMARY KEY,
    start_count INTEGER NOT NULL DEFAULT 0,
    first_start_time TEXT,
    last_start_time TEXT,
    last_stop_time TEXT,
    last_session_duration INTEGER DEFAULT 0,
    total_lifetime INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    -- password_hash and the financial/profile columns are nullable so the
    -- Google OAuth insert (Phase 2b) can mirror Express exactly: passport's
    -- find-or-create writes NULL password_hash/age/profession/annual_income/
    -- net_worth and the user fills them in via /complete-profile. The CHECKs
    -- still apply to non-NULL values (SQLite treats NULL as passing).
    password_hash VARCHAR(255),
    password_algo TEXT NOT NULL DEFAULT 'bcrypt',
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    age INTEGER CHECK (age >= 18 AND age <= 100),
    profession VARCHAR(200),
    annual_income BIGINT CHECK (annual_income >= 5000000),
    net_worth BIGINT CHECK (net_worth >= 30000000),
    membership_tier VARCHAR(20) NOT NULL CHECK (membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
    privacy_level INTEGER NOT NULL DEFAULT 3 CHECK (privacy_level >= 1 AND privacy_level <= 5),
    is_verified INTEGER DEFAULT 0,
    verification_status VARCHAR(20) DEFAULT 'pending' CHECK (verification_status IN ('pending', 'approved', 'rejected')),
    role VARCHAR(20) NOT NULL DEFAULT 'user' CHECK (role IN ('user', 'admin', 'super_admin')),
    profile_picture TEXT,
    bio TEXT,
    interests TEXT,
    stripe_customer_id VARCHAR(255),
    phone_number VARCHAR(20),
    date_of_birth TEXT,
    linkedin_profile VARCHAR(255),
    company VARCHAR(255),
    job_title VARCHAR(255),
    preferred_language VARCHAR(10) DEFAULT 'en',
    timezone VARCHAR(50) DEFAULT 'UTC',
    profile_visibility VARCHAR(20) DEFAULT 'members' CHECK (profile_visibility IN ('public', 'members', 'private')),
    email_notifications INTEGER DEFAULT 1,
    push_notifications INTEGER DEFAULT 1,
    marketing_emails INTEGER DEFAULT 0,
    last_login_at TEXT,
    login_count INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS venues (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    district VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100) DEFAULT 'Taiwan',
    venue_type VARCHAR(50) NOT NULL,
    capacity_min INTEGER DEFAULT 1,
    capacity_max INTEGER NOT NULL,
    price_tier VARCHAR(20) DEFAULT 'premium',
    amenities TEXT,
    contact_name VARCHAR(100),
    contact_phone VARCHAR(50),
    contact_email VARCHAR(255),
    booking_requirements TEXT,
    cancellation_policy TEXT,
    is_active INTEGER DEFAULT 1,
    images TEXT,
    location_coordinates TEXT,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS event_categories (
    id INTEGER PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    icon VARCHAR(50),
    color VARCHAR(7),
    target_membership_tiers TEXT,
    typical_duration_hours INTEGER DEFAULT 3,
    typical_capacity TEXT,
    is_active INTEGER DEFAULT 1,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    detailed_description TEXT,
    category_id INTEGER NOT NULL REFERENCES event_categories(id) ON DELETE RESTRICT,
    venue_id INTEGER NOT NULL REFERENCES venues(id) ON DELETE RESTRICT,
    organizer_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    start_datetime TEXT NOT NULL,
    end_datetime TEXT NOT NULL,
    timezone VARCHAR(50) DEFAULT 'Asia/Taipei',
    capacity_min INTEGER DEFAULT 1,
    capacity_max INTEGER NOT NULL CHECK (capacity_max > 0),
    current_registrations INTEGER NOT NULL DEFAULT 0 CHECK (current_registrations >= 0),
    price_platinum DECIMAL(10, 2),
    price_diamond DECIMAL(10, 2),
    price_black_card DECIMAL(10, 2),
    currency VARCHAR(3) NOT NULL DEFAULT 'TWD',
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'pending_review', 'approved', 'published', 'full', 'completed', 'cancelled', 'archived')),
    approval_status VARCHAR(20) DEFAULT 'pending' CHECK (approval_status IN ('pending', 'approved', 'rejected')),
    approved_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    approved_at TEXT,
    required_membership_tiers TEXT,
    required_verification INTEGER DEFAULT 1,
    age_restriction TEXT,
    dress_code VARCHAR(100),
    language VARCHAR(50) DEFAULT 'Traditional Chinese',
    special_requirements TEXT,
    inclusions TEXT,
    exclusions TEXT,
    registration_opens_at TEXT,
    registration_closes_at TEXT,
    cancellation_deadline TEXT,
    waitlist_enabled INTEGER DEFAULT 1,
    auto_approval INTEGER DEFAULT 0,
    meta_title VARCHAR(255),
    meta_description TEXT,
    featured_image VARCHAR(500),
    gallery_images TEXT,
    internal_notes TEXT,
    cost_breakdown TEXT,
    profit_margin DECIMAL(5, 2),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    published_at TEXT,
    archived_at TEXT,
    CHECK (end_datetime > start_datetime)
);

CREATE TABLE IF NOT EXISTS registrations (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'waitlisted', 'cancelled', 'attended', 'no_show')),
    registration_type VARCHAR(20) DEFAULT 'member' CHECK (registration_type IN ('member', 'guest', 'vip')),
    guest_count INTEGER DEFAULT 0 CHECK (guest_count >= 0),
    guest_details TEXT,
    price_paid DECIMAL(10, 2),
    currency VARCHAR(3) DEFAULT 'TWD',
    payment_status VARCHAR(20) DEFAULT 'pending' CHECK (payment_status IN ('pending', 'paid', 'partial', 'refunded', 'failed')),
    payment_method VARCHAR(50),
    payment_reference VARCHAR(255),
    payment_intent_id VARCHAR(255),
    payment_date TEXT,
    dietary_restrictions TEXT,
    accessibility_needs TEXT,
    special_requests TEXT,
    emergency_contact TEXT,
    confirmed_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    confirmed_at TEXT,
    check_in_time TEXT,
    confirmation_sent INTEGER DEFAULT 0,
    reminder_sent INTEGER DEFAULT 0,
    feedback_collected INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    cancelled_at TEXT,
    UNIQUE (event_id, user_id)
);

CREATE TABLE IF NOT EXISTS event_waitlist (
    id INTEGER PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position > 0),
    status VARCHAR(20) DEFAULT 'waiting' CHECK (status IN ('waiting', 'offered', 'accepted', 'declined', 'expired')),
    offered_at TEXT,
    offer_expires_at TEXT,
    response_deadline TEXT,
    notify_when_available INTEGER DEFAULT 1,
    notification_sent INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    removed_at TEXT,
    UNIQUE (event_id, user_id)
);

CREATE TABLE IF NOT EXISTS event_privacy_overrides (
    id INTEGER PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    privacy_level INTEGER NOT NULL DEFAULT 3 CHECK (privacy_level >= 1 AND privacy_level <= 5),
    allow_contact INTEGER DEFAULT 1,
    show_in_list INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (user_id, event_id)
);

CREATE TABLE IF NOT EXISTS event_participant_access (
    id INTEGER PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    registration_id INTEGER REFERENCES registrations(id) ON DELETE CASCADE,
    has_access INTEGER DEFAULT 0,
    access_granted_at TEXT,
    payment_status VARCHAR(20) DEFAULT 'pending',
    access_level VARCHAR(20) DEFAULT 'basic',
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE (user_id, event_id)
);

CREATE TABLE IF NOT EXISTS participant_view_logs (
    id INTEGER PRIMARY KEY,
    viewer_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    participant_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    access_level INTEGER NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    viewed_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sales_leads (
    id INTEGER PRIMARY KEY,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20),
    company VARCHAR(200),
    job_title VARCHAR(200),
    annual_income BIGINT,
    net_worth BIGINT,
    source VARCHAR(50) NOT NULL DEFAULT 'manual',
    referral_code VARCHAR(50),
    lead_score INTEGER DEFAULT 0 CHECK (lead_score >= 0 AND lead_score <= 100),
    status VARCHAR(20) DEFAULT 'new' CHECK (status IN ('new', 'qualified', 'contacted', 'nurturing', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
    interested_membership_tier VARCHAR(20) CHECK (interested_membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
    budget_range VARCHAR(50),
    timeline VARCHAR(50),
    pain_points TEXT,
    interests TEXT,
    notes TEXT,
    last_contact_date TEXT,
    next_follow_up_date TEXT,
    assigned_to TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sales_opportunities (
    id INTEGER PRIMARY KEY,
    -- No ON DELETE action, mirroring Express where the FK is declared but
    -- never enforced: lead deletion orphans children explicitly in the
    -- delete handler batch (a CASCADE would silently destroy them).
    lead_id INTEGER REFERENCES sales_leads(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    stage VARCHAR(30) NOT NULL DEFAULT 'qualification' CHECK (stage IN ('qualification', 'needs_analysis', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
    probability INTEGER DEFAULT 25 CHECK (probability >= 0 AND probability <= 100),
    value DECIMAL(12, 2) NOT NULL DEFAULT 0,
    expected_close_date TEXT,
    actual_close_date TEXT,
    membership_tier VARCHAR(20) NOT NULL CHECK (membership_tier IN ('Platinum', 'Diamond', 'Black Card')),
    payment_terms VARCHAR(50),
    close_reason TEXT,
    assigned_to TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sales_activities (
    id INTEGER PRIMARY KEY,
    lead_id INTEGER REFERENCES sales_leads(id),
    opportunity_id INTEGER REFERENCES sales_opportunities(id) ON DELETE CASCADE,
    activity_type VARCHAR(30) NOT NULL CHECK (activity_type IN ('call', 'email', 'meeting', 'demo', 'proposal', 'follow_up', 'note')),
    subject VARCHAR(255) NOT NULL,
    description TEXT,
    outcome VARCHAR(50),
    duration_minutes INTEGER,
    scheduled_at TEXT,
    completed_at TEXT,
    created_by TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sales_pipeline_stages (
    id INTEGER PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    display_order INTEGER NOT NULL,
    default_probability INTEGER DEFAULT 0 CHECK (default_probability >= 0 AND default_probability <= 100),
    is_active INTEGER DEFAULT 1,
    color_code VARCHAR(7),
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS sales_team_members (
    id INTEGER PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(30) NOT NULL CHECK (role IN ('sales_rep', 'senior_sales_rep', 'sales_manager', 'sales_director')),
    territory VARCHAR(100),
    commission_rate DECIMAL(5, 2) DEFAULT 0.00,
    quota_amount BIGINT DEFAULT 0,
    is_active INTEGER DEFAULT 1,
    hire_date TEXT,
    manager_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS event_media (
    id VARCHAR(36) PRIMARY KEY,
    event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    type VARCHAR(20) NOT NULL CHECK (type IN ('image', 'document')),
    file_path VARCHAR(500) NOT NULL,
    thumbnail_path TEXT,
    original_filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS venue_media (
    id VARCHAR(36) PRIMARY KEY,
    venue_id INTEGER NOT NULL REFERENCES venues(id) ON DELETE CASCADE,
    type VARCHAR(20) NOT NULL CHECK (type IN ('image', 'document')),
    file_path VARCHAR(500) NOT NULL,
    thumbnail_path TEXT,
    original_filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_membership_tier ON users(membership_tier);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login_at);
CREATE INDEX IF NOT EXISTS idx_users_company ON users(company);

CREATE INDEX IF NOT EXISTS idx_venues_venue_type ON venues(venue_type);
CREATE INDEX IF NOT EXISTS idx_venues_is_active ON venues(is_active);
CREATE INDEX IF NOT EXISTS idx_venues_city ON venues(city);

CREATE INDEX IF NOT EXISTS idx_event_categories_is_active ON event_categories(is_active);

CREATE INDEX IF NOT EXISTS idx_events_category_id ON events(category_id);
CREATE INDEX IF NOT EXISTS idx_events_venue_id ON events(venue_id);
CREATE INDEX IF NOT EXISTS idx_events_organizer_id ON events(organizer_id);
CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
CREATE INDEX IF NOT EXISTS idx_events_approval_status ON events(approval_status);
CREATE INDEX IF NOT EXISTS idx_events_start_datetime ON events(start_datetime);
CREATE INDEX IF NOT EXISTS idx_events_published_at ON events(published_at);

CREATE INDEX IF NOT EXISTS idx_registrations_event_id ON registrations(event_id);
CREATE INDEX IF NOT EXISTS idx_registrations_user_id ON registrations(user_id);
CREATE INDEX IF NOT EXISTS idx_registrations_status ON registrations(status);
CREATE INDEX IF NOT EXISTS idx_registrations_payment_status ON registrations(payment_status);

CREATE INDEX IF NOT EXISTS idx_event_waitlist_event_id ON event_waitlist(event_id);
CREATE INDEX IF NOT EXISTS idx_event_waitlist_user_id ON event_waitlist(user_id);
CREATE INDEX IF NOT EXISTS idx_event_waitlist_position ON event_waitlist(position);

CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_user_id ON event_privacy_overrides(user_id);
CREATE INDEX IF NOT EXISTS idx_event_privacy_overrides_event_id ON event_privacy_overrides(event_id);

CREATE INDEX IF NOT EXISTS idx_event_participant_access_event_id ON event_participant_access(event_id);
CREATE INDEX IF NOT EXISTS idx_event_participant_access_registration_id ON event_participant_access(registration_id);
CREATE INDEX IF NOT EXISTS idx_event_participant_access_payment_status ON event_participant_access(payment_status);

CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewer ON participant_view_logs(viewer_id);
CREATE INDEX IF NOT EXISTS idx_participant_view_logs_participant ON participant_view_logs(participant_id);
CREATE INDEX IF NOT EXISTS idx_participant_view_logs_event ON participant_view_logs(event_id);
CREATE INDEX IF NOT EXISTS idx_participant_view_logs_viewed_at ON participant_view_logs(viewed_at);

CREATE INDEX IF NOT EXISTS idx_sales_leads_assigned_to ON sales_leads(assigned_to);
CREATE INDEX IF NOT EXISTS idx_sales_leads_status ON sales_leads(status);

CREATE INDEX IF NOT EXISTS idx_sales_opportunities_lead_id ON sales_opportunities(lead_id);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_assigned_to ON sales_opportunities(assigned_to);
CREATE INDEX IF NOT EXISTS idx_sales_opportunities_stage ON sales_opportunities(stage);

CREATE INDEX IF NOT EXISTS idx_sales_activities_lead_id ON sales_activities(lead_id);
CREATE INDEX IF NOT EXISTS idx_sales_activities_opportunity_id ON sales_activities(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_sales_activities_created_by ON sales_activities(created_by);

CREATE INDEX IF NOT EXISTS idx_sales_team_members_user_id ON sales_team_members(user_id);

CREATE INDEX IF NOT EXISTS idx_event_media_event_id ON event_media(event_id);
CREATE INDEX IF NOT EXISTS idx_event_media_type ON event_media(type);
CREATE INDEX IF NOT EXISTS idx_event_media_uploaded_by ON event_media(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_event_media_created_at ON event_media(created_at);

CREATE INDEX IF NOT EXISTS idx_venue_media_venue_id ON venue_media(venue_id);
CREATE INDEX IF NOT EXISTS idx_venue_media_type ON venue_media(type);
CREATE INDEX IF NOT EXISTS idx_venue_media_uploaded_by ON venue_media(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_venue_media_created_at ON venue_media(created_at);

-- Visitor tracking. Ported from the DuckDB tables Express writes
-- (`database/migrations/005_visitor_tracking.sql`) rather than kept in
-- Analytics Engine: AE was only chosen to dodge D1's write bottleneck, and
-- with D1 gone the tracking belongs next to the data it reports on.
-- `time_spent` has no DuckDB counterpart; it carries what the AE port tracked
-- as `double3` and what `events/engagement` averages.
CREATE TABLE IF NOT EXISTS visitor_sessions (
    id INTEGER PRIMARY KEY,
    visitor_id VARCHAR(50) NOT NULL UNIQUE,
    user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NOT NULL,
    referer TEXT,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    page_views INTEGER NOT NULL DEFAULT 1,
    session_count INTEGER NOT NULL DEFAULT 1,
    converted_at TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS visitor_page_views (
    id INTEGER PRIMARY KEY,
    visitor_id VARCHAR(50) NOT NULL,
    path VARCHAR(500) NOT NULL,
    method VARCHAR(10) NOT NULL DEFAULT 'GET',
    query_params TEXT,
    referer TEXT,
    timestamp TEXT NOT NULL,
    time_spent REAL NOT NULL DEFAULT 0,
    ip_address VARCHAR(45) NOT NULL,
    user_agent TEXT NOT NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS visitor_events (
    id INTEGER PRIMARY KEY,
    visitor_id VARCHAR(50) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    event_data TEXT,
    timestamp TEXT NOT NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_visitor_sessions_visitor_id ON visitor_sessions(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_sessions_user_id ON visitor_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_visitor_sessions_last_seen ON visitor_sessions(last_seen);

CREATE INDEX IF NOT EXISTS idx_visitor_page_views_visitor_id ON visitor_page_views(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_page_views_timestamp ON visitor_page_views(timestamp);
CREATE INDEX IF NOT EXISTS idx_visitor_page_views_path ON visitor_page_views(path);

CREATE INDEX IF NOT EXISTS idx_visitor_events_visitor_id ON visitor_events(visitor_id);
CREATE INDEX IF NOT EXISTS idx_visitor_events_type ON visitor_events(event_type);
CREATE INDEX IF NOT EXISTS idx_visitor_events_timestamp ON visitor_events(timestamp);
