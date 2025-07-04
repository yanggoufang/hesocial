-- Seed data for HeSocial Platform (DuckDB)
-- This file contains sample data for development and testing

-- Insert sample event categories
INSERT INTO event_categories (name, description, icon) VALUES
('私人晚宴', '高級米其林餐廳的獨家用餐體驗', 'utensils'),
('遊艇派對', '豪華遊艇上的社交聚會', 'anchor'),
('藝術沙龍', '當代藝術展覽與收藏品鑑賞', 'palette'),
('商務社交', '高端商務人士的networking活動', 'briefcase'),
('品酒會', '頂級酒莊的專業品酒體驗', 'wine'),
('高爾夫', '私人高爾夫球場的社交活動', 'golf');

-- Insert sample venues
INSERT INTO venues (name, address, city, latitude, longitude, rating, amenities, images) VALUES
('台北文華東方酒店', '台北市松山區敦化北路166號', '台北', 25.0478, 121.5489, 5, 
 ['米其林餐廳', '頂樓露台', '私人包廂', '代客泊車'], 
 ['/images/venues/mandarin-oriental.jpg']),

('基隆港VIP碼頭', '基隆市中正區中正路1號', '基隆', 25.1276, 121.7392, 5, 
 ['豪華遊艇', '專屬碼頭', '禮賓服務', '海景視野'], 
 ['/images/venues/keelung-marina.jpg']),

('台北當代藝術館', '台北市大同區長安西路39號', '台北', 25.0515, 121.5197, 4, 
 ['當代藝術', '私人展廳', '專業導覽', '典藏室'], 
 ['/images/venues/moca-taipei.jpg']),

('陽明山私人會所', '台北市士林區陽明山建國街1號', '台北', 25.1551, 121.5607, 5, 
 ['山景', '私人會所', '高爾夫練習場', '溫泉'], 
 ['/images/venues/yangmingshan-club.jpg']),

('台中麗思卡爾頓酒店', '台中市西屯區市政北一路77號', '台中', 24.1608, 120.6478, 5, 
 ['米其林餐廳', '行政酒廊', '私人會議室', '豪華套房'], 
 ['/images/venues/ritz-carlton-taichung.jpg']);

-- Insert sample users (for development/testing - passwords are hashed 'password123!')
INSERT INTO users (email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, bio, interests) VALUES
('chen.executive@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '志明', '陳', 52, '科技公司CEO', 25000000, 180000000, 'Diamond', 3, true, 'approved', 
 '專注於科技創新與社會責任的企業家，熱愛藝術收藏與高爾夫運動。', 
 ['科技創新', '藝術收藏', '高爾夫', '紅酒品鑑', '慈善事業']),

('wang.investor@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '美麗', '王', 48, '投資銀行家', 35000000, 250000000, 'Black Card', 4, true, 'approved', 
 '國際投資專家，專精於亞洲市場。享受精緻料理與藝術文化。', 
 ['投資理財', '國際文化', '美食', '音樂', '旅遊']),

('lin.architect@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '建華', '林', 45, '建築師', 15000000, 120000000, 'Platinum', 2, true, 'approved', 
 '知名建築師，設計多項國際獲獎作品。對建築美學與生活品味有獨特見解。', 
 ['建築設計', '藝術', '攝影', '品茶', '古典音樂']),

('zhang.doctor@example.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '雅婷', '張', 50, '整形外科醫師', 18000000, 95000000, 'Diamond', 3, true, 'approved', 
 '專業整形外科醫師，致力於提升客戶自信與美麗。熱愛時尚與美容產業。', 
 ['醫學美容', '時尚', '瑜伽', '健身', '美食']);

-- Insert sample events
INSERT INTO events (name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES
('星空下的法式晚宴',
 '與米其林三星主廚共享精緻法式料理，在台北101頂樓欣賞城市夜景。這是一場結合美食、藝術與社交的頂級體驗，限量20位貴賓參與。',
 '2025-08-27 19:00:00', '2025-08-25 23:59:59', 1, 1, 1,
 '{"vvip": 15000, "vip": 10000, "currency": "TWD", "installmentOptions": [6, 12]}',
 'VVIP', 5, 20,
 ['米其林主廚', '專屬侍酒師', '現場音樂', '城市夜景', '禮賓服務'],
 ['匿名參與選項', '禁止攝影', '機密協議', '專屬入口'],
 ['/images/events/french-dinner-1.jpg', '/images/events/french-dinner-2.jpg'],
 '[{"type": "age", "value": "35-70", "description": "年齡限制35-70歲"}, {"type": "membership", "value": "Diamond", "description": "需要Diamond會員資格"}]'),

('私人遊艇品酒之夜',
 '在豪華遊艇上品嚐世界頂級香檳，與成功企業家建立深度連結。享受夕陽西下的浪漫時光，體驗海上社交的獨特魅力。',
 '2025-08-15 18:30:00', '2025-08-13 23:59:59', 2, 2, 2,
 '{"vip": 8000, "currency": "TWD", "installmentOptions": [6]}',
 'VIP', 4, 16,
 ['頂級香檳', '專業品酒師', '海上夕陽', '精緻小食', '禮賓接送'],
 ['私密社交', '限制攝影', '會員保證'],
 ['/images/events/yacht-party-1.jpg', '/images/events/yacht-party-2.jpg'],
 '[{"type": "age", "value": "30-65", "description": "年齡限制30-65歲"}, {"type": "income", "value": "8000000", "description": "年收入需達800萬以上"}]'),

('當代藝術收藏家沙龍',
 '與知名藝術策展人探討當代藝術趨勢，欣賞私人收藏珍品。這是一場深度的藝術文化交流，適合藝術愛好者與收藏家參與。',
 '2025-09-08 15:00:00', '2025-09-06 23:59:59', 3, 3, 3,
 '{"vvip": 12000, "currency": "TWD"}',
 'Invitation Only', 3, 12,
 ['知名策展人', '私人收藏', '藝術導覽', '下午茶', '限量畫冊'],
 ['專業討論', '學術交流', '收藏家網絡'],
 ['/images/events/art-salon-1.jpg', '/images/events/art-salon-2.jpg'],
 '[{"type": "membership", "value": "Diamond", "description": "僅限Diamond會員"}, {"type": "verification", "value": "required", "description": "需要身份驗證"}]'),

('高爾夫商務社交',
 '在陽明山私人會所享受高爾夫與商務社交的完美結合。與各界菁英建立深度商業關係，在輕鬆的環境中探討合作機會。',
 '2025-07-20 14:00:00', '2025-07-18 23:59:59', 4, 4, 1,
 '{"vip": 6000, "currency": "TWD", "installmentOptions": [3, 6]}',
 'VIP', 3, 24,
 ['18洞高爾夫', '專業教練', '商務午餐', '網絡交流', '禮賓服務'],
 ['商務機密保護', '會員限定', '專業討論'],
 ['/images/events/golf-business-1.jpg', '/images/events/golf-business-2.jpg'],
 '[{"type": "age", "value": "30-65", "description": "年齡限制30-65歲"}, {"type": "membership", "value": "Platinum", "description": "需要Platinum以上會員資格"}]'),

('台中威士忌品鑑晚宴',
 '在台中麗思卡爾頓酒店享受頂級威士忌品鑑體驗。由國際威士忌專家帶領，品嚐來自蘇格蘭與日本的珍稀佳釀。',
 '2025-07-25 19:30:00', '2025-07-23 23:59:59', 5, 5, 2,
 '{"vvip": 9000, "vip": 6500, "currency": "TWD", "installmentOptions": [6]}',
 'VIP', 4, 18,
 ['珍稀威士忌', '國際專家', '精緻晚餐', '品酒證書', '專屬禮品'],
 ['私密品鑑', '專業交流', '收藏建議'],
 ['/images/events/whisky-tasting-1.jpg', '/images/events/whisky-tasting-2.jpg'],
 '[{"type": "age", "value": "25-70", "description": "年齡限制25-70歲"}, {"type": "income", "value": "5000000", "description": "年收入需達500萬以上"}]'),

('音樂沙龍與香檳之夜',
 '在台北文華東方酒店的私人音樂廳，享受古典音樂演奏搭配頂級香檳的優雅夜晚。與音樂愛好者分享藝術心得。',
 '2025-08-05 20:00:00', '2025-08-03 23:59:59', 1, 3, 3,
 '{"vvip": 7500, "currency": "TWD"}',
 'VVIP', 5, 15,
 ['古典音樂', '頂級香檳', '私人音樂廳', '藝術討論', 'VIP包廂'],
 ['優雅氛圍', '藝術交流', '文化深度'],
 ['/images/events/music-salon-1.jpg', '/images/events/music-salon-2.jpg'],
 '[{"type": "membership", "value": "Diamond", "description": "僅限Diamond會員"}, {"type": "verification", "value": "required", "description": "需要身份驗證"}]');

-- Insert sample financial verifications
INSERT INTO financial_verifications (user_id, income_proof_url, asset_documents, verification_date, status, notes) VALUES
(1, '/documents/income-proof-1.pdf', '{"bank_statements": "/documents/bank-1.pdf", "investment_portfolio": "/documents/investment-1.pdf"}', '2024-06-17 14:41:43', 'approved', 'All documents verified successfully'),
(2, '/documents/income-proof-2.pdf', '{"bank_statements": "/documents/bank-2.pdf", "investment_portfolio": "/documents/investment-2.pdf"}', '2024-06-17 14:41:43', 'approved', 'All documents verified successfully'),
(3, '/documents/income-proof-3.pdf', '{"bank_statements": "/documents/bank-3.pdf", "investment_portfolio": "/documents/investment-3.pdf"}', '2024-06-17 14:41:43', 'approved', 'All documents verified successfully'),
(4, '/documents/income-proof-4.pdf', '{"bank_statements": "/documents/bank-4.pdf", "investment_portfolio": "/documents/investment-4.pdf"}', '2024-06-17 14:41:43', 'approved', 'All documents verified successfully');

-- Insert sample audit logs
INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent) VALUES
(1, 'user_registration', 'user', 1, '{"registration_source": "website", "membership_tier": "Diamond"}', '192.168.1.100', 'Mozilla/5.0'),
(2, 'user_registration', 'user', 2, '{"registration_source": "website", "membership_tier": "Black Card"}', '192.168.1.101', 'Mozilla/5.0'),
(3, 'user_registration', 'user', 3, '{"registration_source": "website", "membership_tier": "Platinum"}', '192.168.1.102', 'Mozilla/5.0'),
(4, 'user_registration', 'user', 4, '{"registration_source": "website", "membership_tier": "Diamond"}', '192.168.1.103', 'Mozilla/5.0');