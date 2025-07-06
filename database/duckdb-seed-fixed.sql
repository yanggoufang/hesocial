-- Fixed seed data for HeSocial Platform (DuckDB)
-- All events have future dates and proper syntax

-- Insert event categories
INSERT INTO event_categories (name, description, icon) VALUES
('私人晚宴', '高級米其林餐廳的獨家用餐體驗', 'utensils'),
('遊艇派對', '豪華遊艇上的社交聚會', 'anchor'),
('藝術沙龍', '當代藝術展覽與收藏品鑑賞', 'palette'),
('商務社交', '高端商務人士的networking活動', 'briefcase'),
('品酒會', '頂級酒莊的專業品酒體驗', 'wine'),
('高爾夫', '私人高爾夫球場的社交活動', 'golf-ball-tee'),
('文化藝術', '音樂會、戲劇、文化交流活動', 'music'),
('慈善公益', '高端慈善晚宴與公益活動', 'heart'),
('投資理財', '私人投資論壇與財富管理', 'trending-up'),
('生活品味', '時尚、美容、生活方式體驗', 'sparkles');

-- Insert luxury venues
INSERT INTO venues (name, address, city, latitude, longitude, rating, amenities, images) VALUES
('台北文華東方酒店', '台北市松山區敦化北路166號', '台北', 25.0478, 121.5489, 5, 
 ['米其林餐廳', '頂樓露台', '私人包廂', '代客泊車'], 
 ['/images/venues/mandarin-oriental.jpg']),
('淡水漁人碼頭VIP遊艇俱樂部', '新北市淡水區觀海路83號', '新北', 25.1755, 121.4085, 5, 
 ['豪華遊艇', '專屬碼頭', '禮賓服務', '海景視野'], 
 ['/images/venues/tamsui-marina.jpg']),
('台北當代藝術館', '台北市大同區長安西路39號', '台北', 25.0515, 121.5197, 5, 
 ['當代藝術', '私人展廳', '專業導覽', '典藏室'], 
 ['/images/venues/moca-taipei.jpg']),
('陽明山天籟渡假酒店', '新北市金山區重和里名流路1-7號', '新北', 25.2082, 121.5727, 5, 
 ['溫泉', '高爾夫球場', '私人會所', '山景'], 
 ['/images/venues/yangmingshan-resort.jpg']),
('台中麗思卡爾頓酒店', '台中市西屯區市政北一路77號', '台中', 24.1608, 120.6478, 5, 
 ['米其林餐廳', '行政酒廊', '私人會議室', '豪華套房'], 
 ['/images/venues/ritz-carlton-taichung.jpg']);

-- Insert users
INSERT INTO users (email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, bio, interests) VALUES
('richard.chen@techcorp.tw', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '志明', '陳', 52, '科技公司董事長', 45000000, 280000000, 'Black Card', 4, true, 'approved', 
 '台灣知名科技集團創辦人，專注於半導體與AI技術發展。', 
 ['科技創新', '現代藝術', '高爾夫', '威士忌收藏']),
('isabella.wang@capitalfund.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '美麗', '王', 48, '私募基金合夥人', 38000000, 320000000, 'Black Card', 5, true, 'approved', 
 '國際私募股權基金亞太區合夥人，專精於跨境併購與投資。', 
 ['投資併購', '古典音樂', '法式料理', '當代藝術']),
('david.lin@archstudio.tw', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '建華', '林', 45, '國際建築師', 25000000, 180000000, 'Diamond', 3, true, 'approved', 
 '普立茲克建築獎得主，設計多項國際地標建築。', 
 ['建築設計', '現代藝術', '攝影', '茶文化']);

-- Insert future events (all dates from August 2025 onwards)
INSERT INTO events (name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES

('米其林三星主廚私人晚宴',
 '與法國米其林三星主廚共享獨家法式料理，結合台灣在地食材的創新詮釋。',
 '2025-08-15 19:30:00', '2025-08-13 23:59:59', 1, 1, 1,
 '{"vvip": 18000, "currency": "TWD"}',
 'VVIP', 5, 16,
 ['米其林主廚', '專屬侍酒師', '主廚互動', '限量簽名食譜'],
 ['禁止攝影', '機密菜單', '私人用餐'],
 ['/images/events/michelin-dinner.jpg'],
 '[{"type": "membership", "value": "Diamond", "description": "需要Diamond以上會員資格"}]'),

('豪華遊艇夕陽香檳派對',
 '在120呎豪華遊艇上欣賞淡水夕陽，品嚐Dom Pérignon香檳與精緻開胃菜。',
 '2025-08-20 17:00:00', '2025-08-18 23:59:59', 2, 2, 2,
 '{"vip": 12000, "currency": "TWD"}',
 'VIP', 4, 20,
 ['120呎遊艇', 'Dom Pérignon香檳', '現場DJ', '精緻餐點'],
 ['私密社交', '限制攝影', '會員保證'],
 ['/images/events/yacht-sunset.jpg'],
 '[{"type": "age", "value": "35-65", "description": "年齡限制35-65歲"}]'),

('當代藝術私人收藏展',
 '獨家參觀台灣重要藏家的私人收藏，包括草間彌生、奈良美智等國際大師作品。',
 '2025-08-25 15:00:00', '2025-08-23 23:59:59', 3, 3, 3,
 '{"vvip": 15000, "currency": "TWD"}',
 'Invitation Only', 3, 12,
 ['私人收藏', '專業導覽', '藝術家簽名畫冊', '下午茶'],
 ['收藏家交流', '私密展示', '學術討論'],
 ['/images/events/private-collection.jpg'],
 '[{"type": "membership", "value": "Diamond", "description": "僅限Diamond會員"}]'),

('陽明山高爾夫商務錦標賽',
 '在陽明山私人高爾夫球場舉辦18洞錦標賽，與各界CEO建立商業關係。',
 '2025-09-01 08:00:00', '2025-08-30 23:59:59', 4, 6, 1,
 '{"vip": 8500, "currency": "TWD"}',
 'VIP', 3, 32,
 ['18洞錦標賽', '專業教練', '慶功宴', '獎盃獎品'],
 ['商務機密保護', '會員限定', '競技公平'],
 ['/images/events/golf-tournament.jpg'],
 '[{"type": "membership", "value": "Platinum", "description": "需要Platinum以上會員"}]'),

('日本威士忌大師品鑑會',
 '邀請日本山崎蒸餾廠首席調酒師親臨台中，品鑑稀有年份威士忌。',
 '2025-09-05 19:00:00', '2025-09-03 23:59:59', 5, 5, 2,
 '{"vvip": 16000, "vip": 12000, "currency": "TWD"}',
 'VIP', 4, 24,
 ['山崎大師', '稀有年份', '專業品評', '威士忌歷史'],
 ['品鑑專業', '收藏建議', '私密交流'],
 ['/images/events/whisky-master.jpg'],
 '[{"type": "age", "value": "30-70", "description": "年齡限制30-70歲"}]'),

('維也納愛樂四重奏音樂會',
 '在台北文華東方酒店私人音樂廳，聆聽維也納愛樂樂團四重奏演出。',
 '2025-09-10 20:00:00', '2025-09-08 23:59:59', 1, 7, 3,
 '{"vvip": 22000, "currency": "TWD"}',
 'VVIP', 5, 18,
 ['維也納愛樂', '室內樂', '音樂家交流', '香檳招待'],
 ['優雅氛圍', '文化深度', '音樂鑑賞'],
 ['/images/events/vienna-quartet.jpg'],
 '[{"type": "membership", "value": "Black Card", "description": "僅限Black Card會員"}]'),

('兒童癌症基金慈善晚宴',
 '在台北舉辦慈善晚宴，為兒童癌症基金募集善款。',
 '2025-09-15 18:30:00', '2025-09-13 23:59:59', 1, 8, 1,
 '{"ticket": 15000, "currency": "TWD"}',
 'VIP', 4, 150,
 ['慈善拍賣', '名人表演', '晚宴', '善款捐贈'],
 ['慈善透明', '社會責任', '愛心見證'],
 ['/images/events/charity-gala.jpg'],
 '[{"type": "verification", "value": "required", "description": "需要身份驗證"}]'),

('私募股權投資論壇',
 '討論亞太地區投資機會，與頂級基金經理人分享投資策略。',
 '2025-09-20 14:00:00', '2025-09-18 23:59:59', 5, 9, 2,
 '{"vip": 25000, "currency": "TWD"}',
 'Invitation Only', 4, 50,
 ['投資大師', '市場分析', '投資機會', '私人諮詢'],
 ['商業機密', '投資策略', '私人諮詢'],
 ['/images/events/investment-forum.jpg'],
 '[{"type": "net_worth", "value": "100000000", "description": "淨資產需達1億以上"}]'),

('頂級SPA美容養生體驗',
 '享受全天候SPA療程，包括瑞士抗衰老護理、日式溫泉與營養師諮詢。',
 '2025-09-25 10:00:00', '2025-09-23 23:59:59', 4, 10, 3,
 '{"vip": 8000, "currency": "TWD"}',
 'VIP', 2, 16,
 ['瑞士護理', '溫泉療程', '營養諮詢', '瑜伽課程'],
 ['私人空間', '個人隱私', '專業服務'],
 ['/images/events/luxury-spa.jpg'],
 '[{"type": "gender", "value": "female", "description": "限女性參與"}]'),

('法拉利俱樂部賽道體驗',
 '駕駛法拉利488 GTB，由專業教練指導賽道駕駛技巧。',
 '2025-10-01 09:00:00', '2025-09-29 23:59:59', 5, 10, 1,
 '{"vip": 35000, "currency": "TWD"}',
 'VIP', 3, 12,
 ['法拉利488', '專業教練', '賽道體驗', '駕駛證書'],
 ['安全保障', '專業指導', '保險覆蓋'],
 ['/images/events/ferrari-track.jpg'],
 '[{"type": "license", "value": "required", "description": "需有效駕照"}]'),

('台灣茶文化雅集',
 '品嚐頂級台灣高山茶，學習茶道精神與沖泡技藝。',
 '2025-10-05 14:30:00', '2025-10-03 23:59:59', 4, 7, 3,
 '{"vip": 4500, "currency": "TWD"}',
 'VIP', 2, 20,
 ['茶藝大師', '高山茶品', '茶道體驗', '茶具禮品'],
 ['文化傳承', '寧靜氛圍', '心靈交流'],
 ['/images/events/tea-ceremony.jpg'],
 '[{"type": "interest", "value": "tea_culture", "description": "對茶文化有興趣者"}]'),

('普立茲克建築獎得主講座',
 '邀請知名建築大師分享建築哲學與創作理念。',
 '2025-10-10 16:00:00', '2025-10-08 23:59:59', 3, 3, 2,
 '{"vvip": 12000, "currency": "TWD"}',
 'VVIP', 3, 30,
 ['建築大師', '建築哲學', '創作分享', '私人問答'],
 ['學術交流', '文化深度', '建築美學'],
 ['/images/events/architecture-lecture.jpg'],
 '[{"type": "profession", "value": "architecture_related", "description": "建築相關專業優先"}]');

-- Insert financial verifications
INSERT INTO financial_verifications (user_id, income_proof_url, asset_documents, verification_date, status, notes) VALUES
(1, '/documents/income-proof-1.pdf', '{"bank_statements": "/documents/bank-1.pdf"}', '2024-06-15 10:30:00', 'approved', '科技業高收入驗證完成'),
(2, '/documents/income-proof-2.pdf', '{"bank_statements": "/documents/bank-2.pdf"}', '2024-06-15 11:15:00', 'approved', '私募基金合夥人資產驗證'),
(3, '/documents/income-proof-3.pdf', '{"bank_statements": "/documents/bank-3.pdf"}', '2024-06-15 14:20:00', 'approved', '建築師收入與藝術收藏驗證');

-- Insert sample registrations
INSERT INTO registrations (user_id, event_id, status, payment_status, special_requests) VALUES
(1, 1, 'approved', 'paid', '素食選項'),
(2, 2, 'approved', 'paid', '私人碼頭接送'),
(3, 3, 'approved', 'paid', '英文導覽');

-- Insert audit logs
INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent) VALUES
(1, 'user_registration', 'user', 1, '{"registration_source": "website", "membership_tier": "Black Card"}', '192.168.1.100', 'Mozilla/5.0 Chrome'),
(2, 'user_registration', 'user', 2, '{"registration_source": "website", "membership_tier": "Black Card"}', '192.168.1.101', 'Mozilla/5.0 Safari'),
(3, 'user_registration', 'user', 3, '{"registration_source": "invitation", "membership_tier": "Diamond"}', '192.168.1.102', 'Mozilla/5.0 Chrome');