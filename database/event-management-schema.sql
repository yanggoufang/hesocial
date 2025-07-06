-- Event Content Management System Database Schema
-- This schema extends the existing HeSocial database with event management capabilities

-- Venues table for luxury event locations
CREATE TABLE IF NOT EXISTS venues (
  id BIGINT PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  address TEXT NOT NULL,
  city VARCHAR(100) NOT NULL,
  district VARCHAR(100),
  postal_code VARCHAR(20),
  country VARCHAR(100) DEFAULT 'Taiwan',
  venue_type VARCHAR(50) NOT NULL, -- 'restaurant', 'yacht', 'gallery', 'private_residence', 'hotel', 'outdoor'
  capacity_min INTEGER DEFAULT 1,
  capacity_max INTEGER NOT NULL,
  price_tier VARCHAR(20) DEFAULT 'premium', -- 'premium', 'luxury', 'ultra_luxury'
  amenities JSON, -- ['parking', 'valet', 'wine_cellar', 'private_chef', 'security']
  contact_name VARCHAR(100),
  contact_phone VARCHAR(50),
  contact_email VARCHAR(255),
  booking_requirements TEXT,
  cancellation_policy TEXT,
  is_active BOOLEAN DEFAULT true,
  images JSON, -- Array of image URLs
  location_coordinates JSON, -- {'lat': 25.0330, 'lng': 121.5654}
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Event categories for luxury social experiences
CREATE TABLE IF NOT EXISTS event_categories (
  id BIGINT PRIMARY KEY,
  name VARCHAR(100) NOT NULL UNIQUE,
  slug VARCHAR(100) NOT NULL UNIQUE,
  description TEXT,
  icon VARCHAR(50), -- Icon identifier for UI
  color VARCHAR(7), -- Hex color code
  target_membership_tiers JSON, -- ['Platinum', 'Diamond', 'Black Card']
  typical_duration_hours INTEGER DEFAULT 3,
  typical_capacity JSON, -- {'min': 8, 'max': 20}
  is_active BOOLEAN DEFAULT true,
  sort_order INTEGER DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert default event categories
INSERT OR IGNORE INTO event_categories (id, name, slug, description, icon, color, target_membership_tiers, typical_duration_hours, typical_capacity) VALUES
(1, '私人晚宴', 'private-dining', '獨家私人晚宴體驗，由米其林主廚精心設計', 'chef-hat', '#D4AF37', '["Platinum", "Diamond", "Black Card"]', 4, '{"min": 6, "max": 12}'),
(2, '遊艇派對', 'yacht-parties', '豪華遊艇上的頂級社交聚會', 'anchor', '#1E40AF', '["Diamond", "Black Card"]', 6, '{"min": 10, "max": 30}'),
(3, '藝術鑑賞', 'art-appreciation', '私人畫廊與藝術收藏品鑑會', 'palette', '#7C2D12', '["Platinum", "Diamond", "Black Card"]', 3, '{"min": 8, "max": 15}'),
(4, '商務人脈', 'business-networking', '高端商務交流與人脈建立活動', 'briefcase', '#059669', '["Diamond", "Black Card"]', 3, '{"min": 12, "max": 25}'),
(5, '生活品味', 'lifestyle-wellness', '品酒、茶道、養生等生活美學體驗', 'leaf', '#7C3AED', '["Platinum", "Diamond", "Black Card"]', 2, '{"min": 6, "max": 16}'),
(6, '投資理財', 'investment-seminars', '私人財富管理與投資策略研討', 'trending-up', '#DC2626', '["Black Card"]', 2, '{"min": 5, "max": 10}');

-- Events table for luxury social events
CREATE TABLE IF NOT EXISTS events (
  id BIGINT PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  slug VARCHAR(255) NOT NULL UNIQUE,
  description TEXT,
  detailed_description TEXT,
  category_id BIGINT NOT NULL REFERENCES event_categories(id),
  venue_id BIGINT NOT NULL REFERENCES venues(id),
  organizer_id BIGINT NOT NULL REFERENCES users(id), -- Admin who created the event
  
  -- Event timing
  start_datetime TIMESTAMP NOT NULL,
  end_datetime TIMESTAMP NOT NULL,
  timezone VARCHAR(50) DEFAULT 'Asia/Taipei',
  
  -- Capacity and pricing
  capacity_min INTEGER DEFAULT 1,
  capacity_max INTEGER NOT NULL,
  current_registrations INTEGER DEFAULT 0,
  price_platinum DECIMAL(10,2),
  price_diamond DECIMAL(10,2),
  price_black_card DECIMAL(10,2),
  currency VARCHAR(3) DEFAULT 'TWD',
  
  -- Event status and workflow
  status VARCHAR(20) DEFAULT 'draft', -- 'draft', 'pending_review', 'approved', 'published', 'full', 'completed', 'cancelled', 'archived'
  approval_status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
  approved_by BIGINT REFERENCES users(id),
  approved_at TIMESTAMP,
  
  -- Member requirements
  required_membership_tiers JSON, -- ['Diamond', 'Black Card']
  required_verification BOOLEAN DEFAULT true,
  age_restriction JSON, -- {'min': 25, 'max': null}
  
  -- Event details
  dress_code VARCHAR(100),
  language VARCHAR(50) DEFAULT 'Traditional Chinese',
  special_requirements TEXT,
  inclusions JSON, -- ['transportation', 'meals', 'beverages', 'gifts']
  exclusions JSON,
  
  -- Registration settings
  registration_opens_at TIMESTAMP,
  registration_closes_at TIMESTAMP,
  cancellation_deadline TIMESTAMP,
  waitlist_enabled BOOLEAN DEFAULT true,
  auto_approval BOOLEAN DEFAULT false,
  
  -- SEO and metadata
  meta_title VARCHAR(255),
  meta_description TEXT,
  featured_image VARCHAR(500),
  gallery_images JSON,
  
  -- Admin fields
  internal_notes TEXT,
  cost_breakdown JSON,
  profit_margin DECIMAL(5,2),
  
  -- Timestamps
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  published_at TIMESTAMP,
  archived_at TIMESTAMP
);

-- Event registrations for member participation
CREATE TABLE IF NOT EXISTS event_registrations (
  id BIGINT PRIMARY KEY,
  event_id BIGINT NOT NULL REFERENCES events(id),
  user_id BIGINT NOT NULL REFERENCES users(id),
  
  -- Registration details
  status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'confirmed', 'waitlisted', 'cancelled', 'attended', 'no_show'
  registration_type VARCHAR(20) DEFAULT 'member', -- 'member', 'guest', 'vip'
  guest_count INTEGER DEFAULT 0,
  guest_details JSON,
  
  -- Payment information
  price_paid DECIMAL(10,2),
  currency VARCHAR(3) DEFAULT 'TWD',
  payment_status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'paid', 'partial', 'refunded', 'failed'
  payment_method VARCHAR(50),
  payment_reference VARCHAR(255),
  payment_date TIMESTAMP,
  
  -- Special requests
  dietary_restrictions TEXT,
  accessibility_needs TEXT,
  special_requests TEXT,
  emergency_contact JSON,
  
  -- Confirmation details
  confirmed_by BIGINT REFERENCES users(id),
  confirmed_at TIMESTAMP,
  check_in_time TIMESTAMP,
  
  -- Communication
  confirmation_sent BOOLEAN DEFAULT false,
  reminder_sent BOOLEAN DEFAULT false,
  feedback_collected BOOLEAN DEFAULT false,
  
  -- Timestamps
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  cancelled_at TIMESTAMP
);

-- Event feedback and reviews
CREATE TABLE IF NOT EXISTS event_feedback (
  id BIGINT PRIMARY KEY,
  event_id BIGINT NOT NULL REFERENCES events(id),
  user_id BIGINT NOT NULL REFERENCES users(id),
  registration_id BIGINT REFERENCES event_registrations(id),
  
  -- Ratings (1-5 scale)
  overall_rating INTEGER CHECK (overall_rating >= 1 AND overall_rating <= 5),
  venue_rating INTEGER CHECK (venue_rating >= 1 AND venue_rating <= 5),
  service_rating INTEGER CHECK (service_rating >= 1 AND service_rating <= 5),
  value_rating INTEGER CHECK (value_rating >= 1 AND value_rating <= 5),
  
  -- Written feedback
  title VARCHAR(255),
  review_text TEXT,
  improvements_suggested TEXT,
  would_recommend BOOLEAN,
  would_attend_similar BOOLEAN,
  
  -- Admin response
  admin_response TEXT,
  responded_by BIGINT REFERENCES users(id),
  responded_at TIMESTAMP,
  
  -- Moderation
  is_approved BOOLEAN DEFAULT false,
  is_featured BOOLEAN DEFAULT false,
  moderated_by BIGINT REFERENCES users(id),
  moderated_at TIMESTAMP,
  
  -- Timestamps
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Event waitlist for when events are full
CREATE TABLE IF NOT EXISTS event_waitlist (
  id BIGINT PRIMARY KEY,
  event_id BIGINT NOT NULL REFERENCES events(id),
  user_id BIGINT NOT NULL REFERENCES users(id),
  
  position INTEGER NOT NULL,
  status VARCHAR(20) DEFAULT 'waiting', -- 'waiting', 'offered', 'accepted', 'declined', 'expired'
  
  -- Offer details
  offered_at TIMESTAMP,
  offer_expires_at TIMESTAMP,
  response_deadline TIMESTAMP,
  
  -- Notification preferences
  notify_when_available BOOLEAN DEFAULT true,
  notification_sent BOOLEAN DEFAULT false,
  
  -- Timestamps
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  removed_at TIMESTAMP
);

-- Indexes for performance optimization
CREATE INDEX IF NOT EXISTS idx_events_category_id ON events(category_id);
CREATE INDEX IF NOT EXISTS idx_events_venue_id ON events(venue_id);
CREATE INDEX IF NOT EXISTS idx_events_status ON events(status);
CREATE INDEX IF NOT EXISTS idx_events_start_datetime ON events(start_datetime);
CREATE INDEX IF NOT EXISTS idx_events_published_at ON events(published_at);

CREATE INDEX IF NOT EXISTS idx_event_registrations_event_id ON event_registrations(event_id);
CREATE INDEX IF NOT EXISTS idx_event_registrations_user_id ON event_registrations(user_id);
CREATE INDEX IF NOT EXISTS idx_event_registrations_status ON event_registrations(status);

CREATE INDEX IF NOT EXISTS idx_event_feedback_event_id ON event_feedback(event_id);
CREATE INDEX IF NOT EXISTS idx_event_feedback_user_id ON event_feedback(user_id);

CREATE INDEX IF NOT EXISTS idx_event_waitlist_event_id ON event_waitlist(event_id);
CREATE INDEX IF NOT EXISTS idx_event_waitlist_user_id ON event_waitlist(user_id);
CREATE INDEX IF NOT EXISTS idx_event_waitlist_position ON event_waitlist(position);

CREATE INDEX IF NOT EXISTS idx_venues_venue_type ON venues(venue_type);
CREATE INDEX IF NOT EXISTS idx_venues_is_active ON venues(is_active);