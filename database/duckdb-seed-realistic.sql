-- Enhanced Realistic Seed Data for HeSocial Platform (DuckDB)
-- High-end social event platform targeting affluent individuals aged 45-65

-- Insert enhanced event categories with detailed descriptions
INSERT INTO event_categories (name, description, icon) VALUES
('私人晚宴', '與米其林星級主廚共享精緻美食，專為品味人士設計的獨家用餐體驗', 'utensils'),
('遊艇派對', '在豪華遊艇上享受頂級海上社交體驗，結合美景、美食與優雅社交', 'anchor'),
('藝術沙龍', '當代藝術展覽與收藏品鑑賞，與藝術家和收藏家的深度文化交流', 'palette'),
('商務社交', '高端商務人士的深度networking活動，建立有價值的商業聯繫', 'briefcase'),
('品酒會', '頂級酒莊的專業品酒體驗，包含罕見年份收藏與品酒知識分享', 'wine'),
('高爾夫', '私人高爾夫球場的社交活動，結合運動與商務社交的完美體驗', 'golf'),
('文化講座', '知名學者與文化名人的深度講座，提升文化修養與見識', 'book'),
('私人沙龍', '小型私密聚會，限量邀請的深度社交體驗', 'users');

-- Insert premium venues with detailed information
INSERT INTO venues (name, address, city, latitude, longitude, rating, amenities, images) VALUES
('台北文華東方酒店', '台北市松山區敦化北路166號', '台北', 25.0478, 121.5489, 5, 
 ['米其林餐廳', '頂樓露台', '私人包廂', '代客泊車', '禮賓服務', '專屬電梯', '景觀台'], 
 ['/images/venues/mandarin-oriental-1.jpg', '/images/venues/mandarin-oriental-2.jpg']),

('基隆港VIP碼頭', '基隆市中正區中正路1號', '基隆', 25.1276, 121.7392, 5, 
 ['豪華遊艇', '專屬碼頭', '禮賓服務', '海景視野', '直升機停機坪', '專屬休息室'], 
 ['/images/venues/keelung-marina-1.jpg', '/images/venues/keelung-marina-2.jpg']),

('台北當代藝術館', '台北市大同區長安西路39號', '台北', 25.0515, 121.5197, 5, 
 ['當代藝術', '私人展廳', '專業導覽', '典藏室', '藝術圖書館', '多媒體室'], 
 ['/images/venues/moca-taipei-1.jpg', '/images/venues/moca-taipei-2.jpg']),

('陽明山私人會所', '台北市士林區陽明山建國街1號', '台北', 25.1551, 121.5607, 5, 
 ['山景', '私人會所', '高爾夫練習場', '溫泉', '茶道室', '書法室', '禪修室'], 
 ['/images/venues/yangmingshan-club-1.jpg', '/images/venues/yangmingshan-club-2.jpg']),

('台中麗思卡爾頓酒店', '台中市西屯區市政北一路77號', '台中', 24.1608, 120.6478, 5, 
 ['米其林餐廳', '行政酒廊', '私人會議室', '豪華套房', '頂樓花園', '私人管家'], 
 ['/images/venues/ritz-carlton-taichung-1.jpg', '/images/venues/ritz-carlton-taichung-2.jpg']),

('高雄漢來大飯店', '高雄市前金區成功一路266號', '高雄', 22.6273, 120.3014, 5,
 ['頂樓餐廳', '港景視野', '私人包廂', '專屬停車', '禮賓服務', '行政酒廊'],
 ['/images/venues/grand-hi-lai-kaohsiung-1.jpg', '/images/venues/grand-hi-lai-kaohsiung-2.jpg']),

('新竹國賓大飯店', '新竹市東區中華路二段188號', '新竹', 24.8138, 120.9675, 4,
 ['商務中心', '會議設施', '健身中心', '溫泉', '私人會議室', '高爾夫模擬器'],
 ['/images/venues/ambassador-hsinchu-1.jpg', '/images/venues/ambassador-hsinchu-2.jpg']),

('花蓮理想大地', '花蓮縣壽豐鄉理想路1號', '花蓮', 23.8041, 121.4943, 5,
 ['度假村', '私人海灘', '溫泉', '高爾夫球場', '直升機停機坪', '私人碼頭'],
 ['/images/venues/promisedland-hualien-1.jpg', '/images/venues/promisedland-hualien-2.jpg']);

-- Insert diverse high-profile users with realistic profiles
INSERT INTO users (email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, bio, interests) VALUES
('chen.executive@techcorp.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '志明', '陳', 52, '科技公司CEO', 25000000, 180000000, 'Diamond', 3, true, 'approved', 
 '致力於科技創新與社會責任的企業家，在AI與區塊鏈領域有深度投資。熱愛收集當代藝術品，擁有私人高爾夫球場。', 
 ['科技創新', '藝術收藏', '高爾夫', '紅酒品鑑', '慈善事業', '區塊鏈投資']),

('wang.investor@goldmangroup.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '美麗', '王', 48, '投資銀行家', 35000000, 250000000, 'Black Card', 4, true, 'approved', 
 '國際投資專家，專精於亞洲新興市場與併購業務。曾任職於華爾街頂級投資銀行，現為私人財富管理公司創辦人。', 
 ['國際投資', '併購策略', '美食', '古典音樂', '奢華旅遊', '藝術拍賣']),

('lin.architect@designstudio.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '建華', '林', 45, '建築師', 15000000, 120000000, 'Platinum', 2, true, 'approved', 
 '國際知名建築師，設計多項獲得普立茲克建築獎的作品。專精於可持續建築設計，對東方美學有獨特見解。', 
 ['建築設計', '可持續發展', '攝影', '茶道', '古典音樂', '禪修']),

('zhang.doctor@medicalcenter.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '雅婷', '張', 50, '整形外科醫師', 18000000, 95000000, 'Diamond', 3, true, 'approved', 
 '國際整形外科權威，擁有私人診所與醫學研究基金會。專精於抗衰老醫學，經常受邀至國際醫學會議演講。', 
 ['醫學研究', '抗衰老科學', '瑜伽', '營養學', '時尚設計', '健康管理']),

('liu.realestate@empiregroup.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '國強', '劉', 55, '房地產開發商', 40000000, 320000000, 'Black Card', 4, true, 'approved',
 '大型房地產集團創辦人，專精於豪宅開發與商業地產投資。在台北、上海、新加坡均有大型開發案。',
 ['房地產投資', '城市規劃', '收藏古董', '高爾夫', '私人飛行', '慈善基金會']),

('huang.finance@capitalfund.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '淑芬', '黃', 47, '對沖基金經理', 28000000, 200000000, 'Diamond', 3, true, 'approved',
 '亞洲區頂級對沖基金經理，管理資產超過50億美元。哈佛MBA，曾在倫敦金融城工作十年。',
 ['量化投資', '風險管理', '馬術', '紅酒收藏', '奢華旅遊', '珠寶設計']),

('wu.entrepreneur@innovatetech.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '俊傑', '吳', 49, '創業家', 22000000, 150000000, 'Platinum', 2, true, 'approved',
 '連續創業家，已成功退出三家科技公司。現為天使投資人，專注於AI與生物科技領域投資。',
 ['創業投資', '人工智能', '生物科技', '極限運動', '哲學思辨', '創新思維']),

('lee.lawyer@toplaw.com', '$2b$10$rQZ9uAVQSUg8HSLOzpD3eO9iYg9Yjb1qJ8N0qyc9X1Qjh7Pf1Kg6K', '心怡', '李', 46, '律師', 16000000, 110000000, 'Diamond', 3, true, 'approved',
 '國際商事律師，專精於跨國併購與智慧財產權法。曾任職於紐約頂級律師事務所，現為合夥人。',
 ['國際法', '智慧財產', '藝術法', '古典文學', '品茶', '書法']);

-- Insert premium events with detailed information
INSERT INTO events (name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, amenities, privacy_guarantees, images, requirements) VALUES
('星空下的法式頂級晚宴', 
 '與米其林三星主廚Alain Ducasse共享精緻法式料理，在台北101頂樓獨家包場，俯瞰城市璀璨夜景。晚宴包含八道法式tasting menu，搭配Dom Pérignon香檳與Château Margaux紅酒。現場將有法國國家交響樂團弦樂四重奏伴奏，營造極致浪漫氛圍。', 
 '2024-12-27 19:00:00', '2024-12-25 23:59:59', 1, 1, 1,
 '{"vvip": 25000, "vip": 15000, "currency": "TWD", "installmentOptions": [6, 12], "includes": ["米其林主廚料理", "Dom Pérignon香檳", "專屬侍酒師", "現場音樂", "禮賓接送"]}',
 'VVIP', 5, 16,
 ['米其林三星主廚', 'Dom Pérignon香檳', '法國國家交響樂團', '101頂樓包場', '專屬侍酒師', '禮賓接送服務', '專業攝影師'],
 ['匿名參與選項', '禁止手機攝影', '機密保密協議', '專屬VIP電梯', '身分保護服務'],
 ['/images/events/french-dinner-skyline.jpg', '/images/events/french-dinner-chef.jpg', '/images/events/french-dinner-table.jpg'],
 '[{"type": "age", "value": "35-70", "description": "年齡限制35-70歲"}, {"type": "membership", "value": "Diamond", "description": "需要Diamond或Black Card會員資格"}, {"type": "dress_code", "value": "black_tie", "description": "正式晚宴服裝要求"}]'),

('私人遊艇香檳品鑑之夜', 
 '在150呎豪華遊艇「Ocean Dream」上品嚐世界頂級香檳收藏，包含Krug Clos du Mesnil、Louis Roederer Cristal等珍稀年份。由國際侍酒師大師引導品鑑，同時享受台灣海峽夕陽美景。晚宴包含新鮮海膽、魚子醬等頂級海鮮配搭。', 
 '2024-12-15 18:30:00', '2024-12-13 23:59:59', 2, 2, 2,
 '{"vvip": 18000, "vip": 12000, "currency": "TWD", "installmentOptions": [6], "includes": ["頂級香檳", "專業侍酒師", "海上夕陽", "精緻海鮮", "直升機接送"]}',
 'VVIP', 4, 12,
 ['150呎豪華遊艇', 'Krug香檳收藏', '國際侍酒師大師', '新鮮魚子醬', '直升機接送選項', '專業攝影服務'],
 ['私密海上社交', '限制商業攝影', '會員身分保證', '專屬碼頭通道'],
 ['/images/events/yacht-champagne.jpg', '/images/events/yacht-sunset.jpg', '/images/events/yacht-caviar.jpg'],
 '[{"type": "age", "value": "30-65", "description": "年齡限制30-65歲"}, {"type": "income", "value": "10000000", "description": "年收入需達1000萬以上"}, {"type": "membership", "value": "Platinum", "description": "需要Platinum以上會員資格"}]'),

('當代藝術收藏家私人沙龍', 
 '與國際知名策展人Hans Ulrich Obrist及藝術評論家共同探討當代藝術投資趨勢。現場展示私人收藏的草間彌生、村上隆、奈良美智等亞洲當代藝術珍品。包含專業藝術投資諮詢與拍賣市場分析。', 
 '2024-12-08 15:00:00', '2024-12-06 23:59:59', 3, 3, 3,
 '{"vvip": 20000, "vip": 15000, "currency": "TWD", "includes": ["知名策展人", "私人收藏品", "投資諮詢", "限量畫冊", "下午茶"]}',
 'Invitation Only', 4, 10,
 ['Hans Ulrich Obrist策展', '草間彌生原作', '村上隆收藏', '藝術投資諮詢', '限量簽名畫冊', '私人導覽'],
 ['學術專業討論', '收藏家專屬網絡', '投資機密保護', '作品展示許可'],
 ['/images/events/art-salon-kusama.jpg', '/images/events/art-salon-murakami.jpg', '/images/events/art-salon-discussion.jpg'],
 '[{"type": "membership", "value": "Diamond", "description": "僅限Diamond或Black Card會員"}, {"type": "verification", "value": "required", "description": "需要藝術收藏經驗驗證"}, {"type": "investment", "value": "50000000", "description": "需有藝術投資經驗"}]'),

('高爾夫商務社交錦標賽', 
 '在台北高爾夫俱樂部舉辦的企業家高爾夫比賽，邀請各行業頂級企業家參與。比賽後將在俱樂部會所舉辦晚宴，促進深度商務交流。特別邀請前職業高爾夫球員擔任技術指導。', 
 '2024-12-20 08:00:00', '2024-12-18 23:59:59', 4, 6, 4,
 '{"vvip": 12000, "vip": 8000, "currency": "TWD", "includes": ["18洞比賽", "專業指導", "晚宴", "獎品", "球具租借"]}',
 'VIP', 3, 24,
 ['18洞錦標賽', '前職業球員指導', '企業家晚宴', '商務networking', '豪華獎品', '球具租借服務'],
 ['商務機密保護', '專屬會所通道', '私人儲物櫃', '成績保密選項'],
 ['/images/events/golf-tournament.jpg', '/images/events/golf-networking.jpg', '/images/events/golf-awards.jpg'],
 '[{"type": "membership", "value": "Platinum", "description": "需要Platinum以上會員"}, {"type": "handicap", "value": "30", "description": "高爾夫差點需在30以內"}, {"type": "business", "value": "executive", "description": "需為企業高階主管"}]'),

('威士忌品鑑與雪茄夜', 
 '在台中麗思卡爾頓酒店天台舉辦的威士忌與雪茄品鑑活動。品鑑包含Macallan 25年、Yamazaki 18年等頂級威士忌，搭配古巴Cohiba雪茄。由國際威士忌大師及雪茄專家引導品鑑。', 
 '2024-12-22 20:00:00', '2024-12-20 23:59:59', 5, 5, 5,
 '{"vvip": 15000, "vip": 10000, "currency": "TWD", "includes": ["頂級威士忌", "古巴雪茄", "專家引導", "精緻小食", "專屬雪茄室"]}',
 'VVIP', 4, 16,
 ['Macallan 25年威士忌', 'Cohiba雪茄', '國際威士忌大師', '專屬雪茄室', '城市夜景', '爵士樂現場演奏'],
 ['私密品鑑環境', '商務討論保密', '專屬電梯通道', '雪茄室隱私'],
 ['/images/events/whisky-tasting.jpg', '/images/events/cigar-lounge.jpg', '/images/events/whisky-expert.jpg'],
 '[{"type": "age", "value": "35-65", "description": "年齡限制35-65歲"}, {"type": "membership", "value": "Diamond", "description": "需要Diamond會員資格"}, {"type": "experience", "value": "whisky", "description": "需有威士忌品鑑經驗"}]'),

('私人飛行俱樂部聚會', 
 '在松山機場私人飛機停機坪舉辦的航空愛好者聚會，展示最新的私人飛機型號。包含Gulfstream G650、Bombardier Global 7500等頂級機型參觀。晚宴在機場VIP會所舉行。', 
 '2024-12-25 16:00:00', '2024-12-23 23:59:59', 6, 4, 6,
 '{"vvip": 30000, "vip": 20000, "currency": "TWD", "includes": ["私人飛機參觀", "飛行體驗", "機師交流", "VIP晚宴", "航空雜誌"]}',
 'Invitation Only', 4, 8,
 ['Gulfstream G650參觀', 'Bombardier Global 7500', '飛行體驗', '資深機師交流', '航空投資諮詢', 'VIP機場會所'],
 ['機場安全管制', '參觀許可管理', '商務隱私保護', '專屬通道進出'],
 ['/images/events/private-jet-gulfstream.jpg', '/images/events/private-jet-lounge.jpg', '/images/events/aviation-networking.jpg'],
 '[{"type": "membership", "value": "Black Card", "description": "僅限Black Card會員"}, {"type": "net_worth", "value": "500000000", "description": "淨資產需達5億以上"}, {"type": "aviation", "value": "interest", "description": "需有航空投資興趣"}]');

-- Insert financial verifications for all users
INSERT INTO financial_verifications (user_id, income_proof_url, asset_documents, verification_date, status, notes) VALUES
(1, '/documents/income-proof-chen.pdf', '{"bank_statements": "/documents/bank-chen.pdf", "investment_portfolio": "/documents/investment-chen.pdf", "tax_returns": "/documents/tax-chen.pdf", "business_valuation": "/documents/business-chen.pdf"}', '2024-06-15 10:30:00', 'approved', 'Tech company CEO - All documents verified. Annual income NT$25M confirmed.'),
(2, '/documents/income-proof-wang.pdf', '{"bank_statements": "/documents/bank-wang.pdf", "investment_portfolio": "/documents/investment-wang.pdf", "offshore_accounts": "/documents/offshore-wang.pdf", "fund_statements": "/documents/fund-wang.pdf"}', '2024-06-16 14:20:00', 'approved', 'Investment banker - International portfolio verified. Net worth NT$250M confirmed.'),
(3, '/documents/income-proof-lin.pdf', '{"bank_statements": "/documents/bank-lin.pdf", "investment_portfolio": "/documents/investment-lin.pdf", "real_estate": "/documents/property-lin.pdf", "business_income": "/documents/business-lin.pdf"}', '2024-06-17 09:45:00', 'approved', 'International architect - Project income and property assets verified.'),
(4, '/documents/income-proof-zhang.pdf', '{"bank_statements": "/documents/bank-zhang.pdf", "investment_portfolio": "/documents/investment-zhang.pdf", "medical_practice": "/documents/practice-zhang.pdf", "research_grants": "/documents/grants-zhang.pdf"}', '2024-06-18 11:15:00', 'approved', 'Medical practitioner - Practice income and research funding verified.'),
(5, '/documents/income-proof-liu.pdf', '{"bank_statements": "/documents/bank-liu.pdf", "investment_portfolio": "/documents/investment-liu.pdf", "real_estate_empire": "/documents/empire-liu.pdf", "development_projects": "/documents/projects-liu.pdf"}', '2024-06-19 16:00:00', 'approved', 'Real estate developer - Major development projects and asset portfolio verified.'),
(6, '/documents/income-proof-huang.pdf', '{"bank_statements": "/documents/bank-huang.pdf", "hedge_fund": "/documents/fund-huang.pdf", "investment_returns": "/documents/returns-huang.pdf", "offshore_structures": "/documents/offshore-huang.pdf"}', '2024-06-20 13:30:00', 'approved', 'Hedge fund manager - Fund performance and personal investments verified.'),
(7, '/documents/income-proof-wu.pdf', '{"bank_statements": "/documents/bank-wu.pdf", "startup_exits": "/documents/exits-wu.pdf", "angel_investments": "/documents/angel-wu.pdf", "intellectual_property": "/documents/ip-wu.pdf"}', '2024-06-21 08:45:00', 'approved', 'Serial entrepreneur - Multiple successful exits and angel investment portfolio verified.'),
(8, '/documents/income-proof-lee.pdf', '{"bank_statements": "/documents/bank-lee.pdf", "law_firm_partnership": "/documents/partnership-lee.pdf", "legal_cases": "/documents/cases-lee.pdf", "consulting_income": "/documents/consulting-lee.pdf"}', '2024-06-22 15:20:00', 'approved', 'International lawyer - Law firm partnership and high-value case history verified.');

-- Insert comprehensive audit logs
INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent) VALUES
(1, 'user_registration', 'user', 1, '{"registration_source": "referral", "membership_tier": "Diamond", "referral_code": "TECH2024", "verification_method": "business_documents"}', '203.69.12.45', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'),
(2, 'user_registration', 'user', 2, '{"registration_source": "invitation", "membership_tier": "Black Card", "invited_by": "partner_firm", "verification_method": "financial_statements"}', '118.163.78.32', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'),
(3, 'user_registration', 'user', 3, '{"registration_source": "professional_network", "membership_tier": "Platinum", "profession_verified": true, "awards_validated": true}', '220.135.24.67', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'),
(4, 'user_registration', 'user', 4, '{"registration_source": "medical_association", "membership_tier": "Diamond", "medical_license_verified": true, "practice_validated": true}', '61.216.89.123', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'),
(5, 'user_registration', 'user', 5, '{"registration_source": "business_network", "membership_tier": "Black Card", "company_valuation": "verified", "development_projects": "validated"}', '114.32.145.89', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'),
(6, 'user_registration', 'user', 6, '{"registration_source": "financial_institution", "membership_tier": "Diamond", "fund_aum": "verified", "performance_track_record": "validated"}', '125.227.34.56', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'),
(7, 'user_registration', 'user', 7, '{"registration_source": "startup_ecosystem", "membership_tier": "Platinum", "exit_history": "verified", "angel_portfolio": "validated"}', '203.204.56.78', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'),
(8, 'user_registration', 'user', 8, '{"registration_source": "legal_network", "membership_tier": "Diamond", "bar_admission": "verified", "partnership_status": "validated"}', '210.65.43.21', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'),
(1, 'event_view', 'event', 1, '{"event_name": "星空下的法式頂級晚宴", "interest_level": "high", "pricing_tier_viewed": "vvip"}', '203.69.12.45', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'),
(2, 'event_view', 'event', 2, '{"event_name": "私人遊艇香檳品鑑之夜", "interest_level": "high", "pricing_tier_viewed": "vvip"}', '118.163.78.32', 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'),
(3, 'event_view', 'event', 3, '{"event_name": "當代藝術收藏家私人沙龍", "interest_level": "very_high", "pricing_tier_viewed": "vvip"}', '220.135.24.67', 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36');