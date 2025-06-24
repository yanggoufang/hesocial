-- Seed data for HeSocial Platform
-- This file contains sample data for development and testing

-- Insert sample event categories
INSERT INTO event_categories (id, name, description, icon) VALUES
(uuid_generate_v4(), '私人晚宴', '高級米其林餐廳的獨家用餐體驗', 'utensils'),
(uuid_generate_v4(), '遊艇派對', '豪華遊艇上的社交聚會', 'anchor'),
(uuid_generate_v4(), '藝術沙龍', '當代藝術展覽與收藏品鑑賞', 'palette'),
(uuid_generate_v4(), '商務社交', '高端商務人士的networking活動', 'briefcase'),
(uuid_generate_v4(), '品酒會', '頂級酒莊的專業品酒體驗', 'wine'),
(uuid_generate_v4(), '高爾夫', '私人高爾夫球場的社交活動', 'golf');

-- Insert sample venues
INSERT INTO venues (id, name, address, city, latitude, longitude, rating, amenities, images) VALUES
(uuid_generate_v4(), '台北文華東方酒店', '台北市松山區敦化北路166號', '台北', 25.0478, 121.5489, 5, 
 ARRAY['米其林餐廳', '頂樓露台', '私人包廂', '代客泊車'], 
 ARRAY['/images/venues/mandarin-oriental.jpg']),

(uuid_generate_v4(), '基隆港VIP碼頭', '基隆市中正區中正路1號', '基隆', 25.1276, 121.7392, 5, 
 ARRAY['豪華遊艇', '專屬碼頭', '禮賓服務', '海景視野'], 
 ARRAY['/images/venues/keelung-marina.jpg']),

(uuid_generate_v4(), '台北當代藝術館', '台北市大同區長安西路39號', '台北', 25.0515, 121.5197, 4, 
 ARRAY['當代藝術', '私人展廳', '專業導覽', '典藏室'], 
 ARRAY['/images/venues/moca-taipei.jpg']),

(uuid_generate_v4(), '陽明山私人會所', '台北市士林區陽明山建國街1號', '台北', 25.1551, 121.5607, 5, 
 ARRAY['山景', '私人會所', '高爾夫練習場', '溫泉'], 
 ARRAY['/images/venues/yangmingshan-club.jpg']),

(uuid_generate_v4(), '台中麗思卡爾頓酒店', '台中市西屯區市政北一路77號', '台中', 24.1608, 120.6478, 5, 
 ARRAY['米其林餐廳', '行政酒廊', '私人會議室', '豪華套房'], 
 ARRAY['/images/venues/ritz-carlton-taichung.jpg']);

-- Insert sample users (for development/testing - passwords are hashed 'password123!')
INSERT INTO users (id, email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, bio, interests) VALUES
(uuid_generate_v4(), 'chen.executive@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '志明', '陳', 52, '科技公司CEO', 25000000, 180000000, 'Diamond', 3, true, 'approved', 
 '專注於科技創新與社會責任的企業家，熱愛藝術收藏與高爾夫運動。', 
 ARRAY['科技創新', '藝術收藏', '高爾夫', '紅酒品鑑', '慈善事業']),

(uuid_generate_v4(), 'wang.investor@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '美麗', '王', 48, '投資銀行家', 35000000, 250000000, 'Black Card', 4, true, 'approved', 
 '國際投資專家，專精於亞洲市場。享受精緻料理與藝術文化。', 
 ARRAY['投資理財', '國際文化', '美食', '音樂', '旅遊']),

(uuid_generate_v4(), 'lin.architect@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '建華', '林', 45, '建築師', 15000000, 120000000, 'Platinum', 2, true, 'approved', 
 '知名建築師，設計多項國際獲獎作品。對建築美學與生活品味有獨特見解。', 
 ARRAY['建築設計', '藝術', '攝影', '品茶', '古典音樂']),

(uuid_generate_v4(), 'zhang.doctor@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '雅婷', '張', 50, '整形外科醫師', 18000000, 95000000, 'Diamond', 3, true, 'approved', 
 '專業整形外科醫師，致力於提升客戶自信與美麗。熱愛時尚與美容產業。', 
 ARRAY['醫學美容', '時尚', '瑜伽', '健身', '美食']);

-- Get some UUIDs for events (using the first venue and category)
DO $$
DECLARE
    venue_id UUID;
    category_id UUID;
    organizer_id UUID;
BEGIN
    -- Get first venue
    SELECT id INTO venue_id FROM venues LIMIT 1;
    -- Get first category
    SELECT id INTO category_id FROM event_categories WHERE name = '私人晚宴' LIMIT 1;
    -- Get first user as organizer
    SELECT id INTO organizer_id FROM users LIMIT 1;

    -- Insert sample events
    INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES
    (uuid_generate_v4(), '星空下的法式晚宴', 
     '與米其林三星主廚共享精緻法式料理，在台北101頂樓欣賞城市夜景。這是一場結合美食、藝術與社交的頂級體驗，限量20位貴賓參與。', 
     '2024-12-27 19:00:00+08', '2024-12-25 23:59:59+08', venue_id, category_id, organizer_id,
     '{"vvip": 15000, "vip": 10000, "currency": "TWD", "installmentOptions": [6, 12]}',
     'VVIP', 5, 20,
     ARRAY['米其林主廚', '專屬侍酒師', '現場音樂', '城市夜景', '禮賓服務'],
     ARRAY['匿名參與選項', '禁止攝影', '機密協議', '專屬入口'],
     ARRAY['/images/events/french-dinner-1.jpg', '/images/events/french-dinner-2.jpg'],
     '[{"type": "age", "value": "35-70", "description": "年齡限制35-70歲"}, {"type": "membership", "value": "Diamond", "description": "需要Diamond會員資格"}]');

    -- Get yacht venue and category
    SELECT id INTO venue_id FROM venues WHERE name LIKE '%碼頭%' LIMIT 1;
    SELECT id INTO category_id FROM event_categories WHERE name = '遊艇派對' LIMIT 1;

    INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES
    (uuid_generate_v4(), '私人遊艇品酒之夜', 
     '在豪華遊艇上品嚐世界頂級香檳，與成功企業家建立深度連結。享受夕陽西下的浪漫時光，體驗海上社交的獨特魅力。', 
     '2024-12-15 18:30:00+08', '2024-12-13 23:59:59+08', venue_id, category_id, organizer_id,
     '{"vip": 8000, "currency": "TWD", "installmentOptions": [6]}',
     'VIP', 4, 16,
     ARRAY['頂級香檳', '專業品酒師', '海上夕陽', '精緻小食', '禮賓接送'],
     ARRAY['私密社交', '限制攝影', '會員保證'],
     ARRAY['/images/events/yacht-party-1.jpg', '/images/events/yacht-party-2.jpg'],
     '[{"type": "age", "value": "30-65", "description": "年齡限制30-65歲"}, {"type": "income", "value": "8000000", "description": "年收入需達800萬以上"}]');

    -- Get art venue and category
    SELECT id INTO venue_id FROM venues WHERE name LIKE '%藝術館%' LIMIT 1;
    SELECT id INTO category_id FROM event_categories WHERE name = '藝術沙龍' LIMIT 1;

    INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES
    (uuid_generate_v4(), '當代藝術收藏家沙龍', 
     '與知名藝術策展人探討當代藝術趨勢，欣賞私人收藏珍品。這是一場深度的藝術文化交流，適合藝術愛好者與收藏家參與。', 
     '2024-12-08 15:00:00+08', '2024-12-06 23:59:59+08', venue_id, category_id, organizer_id,
     '{"vvip": 12000, "currency": "TWD"}',
     'Invitation Only', 3, 12,
     ARRAY['知名策展人', '私人收藏', '藝術導覽', '下午茶', '限量畫冊'],
     ARRAY['專業討論', '學術交流', '收藏家網絡'],
     ARRAY['/images/events/art-salon-1.jpg', '/images/events/art-salon-2.jpg'],
     '[{"type": "membership", "value": "Diamond", "description": "僅限Diamond會員"}, {"type": "verification", "value": "required", "description": "需要身份驗證"}]');

END $$;

-- Insert sample financial verifications
INSERT INTO financial_verifications (user_id, income_proof_url, asset_documents, verification_date, status, notes) 
SELECT 
    id,
    '/documents/income-proof-' || id || '.pdf',
    '{"bank_statements": "/documents/bank-' || id || '.pdf", "investment_portfolio": "/documents/investment-' || id || '.pdf"}',
    CURRENT_TIMESTAMP - INTERVAL '7 days',
    'approved',
    'All documents verified successfully'
FROM users
WHERE verification_status = 'approved';

-- Insert sample audit logs
INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
SELECT 
    id,
    'user_registration',
    'user',
    id,
    '{"registration_source": "website", "membership_tier": "' || membership_tier || '"}',
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
FROM users;

-- Refresh materialized views if any exist
-- REFRESH MATERIALIZED VIEW IF EXISTS user_activity_summary;

COMMIT;