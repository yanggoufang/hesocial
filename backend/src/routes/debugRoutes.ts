import { Router } from 'express'

const router = Router()

router.get('/events-all', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    const result = await pool.query(`
      SELECT
        e.id, e.name, e.date_time, e.is_active,
        v.name as venue_name,
        ec.name as category_name
      FROM events e
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      ORDER BY e.date_time
    `)

    res.json({
      success: true,
      total: result.rows.length,
      data: result.rows
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    })
  }
})

router.get('/tables', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    const result = await pool.query(`
      SELECT table_name
      FROM information_schema.tables
      WHERE table_schema = 'main'
      ORDER BY table_name
    `)

    res.json({
      success: true,
      tables: result.rows.map(row => row.table_name)
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    })
  }
})

router.post('/seed', async (req, res) => {
  try {
    const { duckdb } = await import('../database/duckdb-connection.js')
    await duckdb.seedData()

    res.json({
      success: true,
      message: 'Database seeded successfully'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Seeding failed'
    })
  }
})

router.post('/manual-seed', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')

    await pool.query(`INSERT OR IGNORE INTO event_categories (id, name, description, icon) VALUES
      (1, '私人晚宴', '頂級餐廳私人用餐體驗', 'utensils'),
      (2, '遊艇派對', '豪華遊艇社交聚會', 'anchor'),
      (3, '藝術鑑賞', '高端藝術品展覽與收藏', 'palette')`)

    await pool.query(`INSERT OR IGNORE INTO venues (id, name, address, city, latitude, longitude, rating, amenities, images) VALUES
      (1, '台北君悅酒店', '台北市信義區松壽路2號', '台北', 25.0330, 121.5654, 5, '["停車場","無線網路"]', '[]'),
      (2, '大倉久和大飯店', '台北市中山區南京東路一段9號', '台北', 25.0518, 121.5228, 5, '["日式庭園","高級餐廳"]', '[]')`)

    await pool.query(`INSERT OR IGNORE INTO users (id, email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, is_verified, verification_status) VALUES
      (1, 'admin@hesocial.com', '$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6', 'Admin', 'User', 35, 'Administrator', 5000000, 30000000, 'Black Card', true, 'approved')`)

    await pool.query(`DELETE FROM events WHERE id IN (1, 2, 3)`)
    await pool.query(`INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, current_attendees, is_active) VALUES
      (1, '頂級威士忌品鑑晚宴', '邀請威士忌專家分享珍稀威士忌，搭配精緻法式料理', '2025-08-15 19:00:00', '2025-08-10 18:00:00', 1, 1, 1, '{"platinum": 15000, "diamond": 12000, "black_card": 8000}', 'VIP', 4, 20, 8, true),
      (2, '私人遊艇星空派對', '在豪華遊艇上享受星空下的奢華體驗', '2025-08-20 20:00:00', '2025-08-18 12:00:00', 2, 2, 1, '{"platinum": 25000, "diamond": 20000, "black_card": 15000}', 'VVIP', 5, 30, 15, true),
      (3, '藝術收藏家私享會', '與知名藝術收藏家交流，欣賞珍貴藝術品', '2025-08-25 15:00:00', '2025-08-22 17:00:00', 2, 3, 1, '{"platinum": 18000, "diamond": 15000, "black_card": 12000}', 'Invitation Only', 3, 25, 12, true)`)

    res.json({
      success: true,
      message: 'Manual seeding completed successfully'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Manual seeding failed'
    })
  }
})

router.post('/update-dates', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')

    await pool.query(`UPDATE events SET
      date_time = '2025-08-15 19:00:00',
      registration_deadline = '2025-08-10 18:00:00'
      WHERE id = 1`)

    await pool.query(`UPDATE events SET
      date_time = '2025-08-20 20:00:00',
      registration_deadline = '2025-08-18 12:00:00'
      WHERE id = 2`)

    await pool.query(`UPDATE events SET
      date_time = '2025-08-25 15:00:00',
      registration_deadline = '2025-08-22 17:00:00'
      WHERE id = 3`)

    res.json({
      success: true,
      message: 'Event dates updated to future'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Update failed'
    })
  }
})

router.post('/expand-seed', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')

    await pool.query(`INSERT OR IGNORE INTO event_categories (id, name, description, icon) VALUES
      (4, '品酒會', '頂級葡萄酒與烈酒品鑑', 'wine'),
      (5, '高爾夫聚會', '私人高爾夫球場社交', 'target'),
      (6, '慈善晚宴', '高端慈善籌款活動', 'heart'),
      (7, '商務論壇', '精英商業交流會議', 'briefcase'),
      (8, '時尚秀', '頂級時裝展示會', 'shirt')`)

    await pool.query(`INSERT OR IGNORE INTO venues (id, name, address, city, latitude, longitude, rating, amenities, images) VALUES
      (3, '台北101觀景台', '台北市信義區信義路五段7號', '台北', 25.0340, 121.5645, 5, '[\"360度景觀\",\"高級餐廳\"]', '[]'),
      (4, '美麗華百樂園', '台北市中山區敬業三路20號', '台北', 25.0833, 121.5500, 4, '[\"摩天輪\",\"空中花園\"]', '[]'),
      (5, '圓山大飯店', '台北市中山區中山北路四段1號', '台北', 25.0792, 121.5263, 5, '[\"古典建築\",\"中式庭園\"]', '[]'),
      (6, '台北萬豪酒店', '台北市中山區樂群二路199號', '台北', 25.0839, 121.5464, 5, '[\"行政酒廊\",\"室外泳池\"]', '[]'),
      (7, '陽明山中國麗緻大飯店', '台北市北投區格致路237號', '台北', 25.1364, 121.5471, 4, '[\"溫泉\",\"山景\"]', '[]'),
      (8, '台北寒舍艾美酒店', '台北市信義區松仁路38號', '台北', 25.0368, 121.5645, 5, '[\"藝術收藏\",\"米其林餐廳\"]', '[]')`)

    await pool.query(`DELETE FROM events`)
    await pool.query(`INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, current_attendees, is_active) VALUES
      (1, '頂級威士忌品鑑晚宴', '邀請威士忌專家分享珍稀威士忌，搭配精緻法式料理', '2025-08-15 19:00:00', '2025-08-10 18:00:00', 1, 1, 1, '{"platinum": 15000, "diamond": 12000, "black_card": 8000}', 'VIP', 4, 20, 8, true),
      (2, '私人遊艇星空派對', '在豪華遊艇上享受星空下的奢華體驗', '2025-08-20 20:00:00', '2025-08-18 12:00:00', 2, 2, 1, '{"platinum": 25000, "diamond": 20000, "black_card": 15000}', 'VVIP', 5, 30, 15, true),
      (3, '藝術收藏家私享會', '與知名藝術收藏家交流，欣賞珍貴藝術品', '2025-08-25 15:00:00', '2025-08-22 17:00:00', 8, 3, 1, '{"platinum": 18000, "diamond": 15000, "black_card": 12000}', 'Invitation Only', 3, 25, 12, true),
      (4, '法國香檳品鑑會', 'Dom Pérignon與Krug香檳大師班', '2025-09-05 18:30:00', '2025-09-01 15:00:00', 3, 4, 1, '{"platinum": 12000, "diamond": 10000, "black_card": 7000}', 'VIP', 4, 18, 6, true),
      (5, '義大利超級托斯卡納之夜', '品鑑Sassicaia、Ornellaia等頂級紅酒', '2025-09-10 19:30:00', '2025-09-07 18:00:00', 5, 4, 1, '{"platinum": 20000, "diamond": 17000, "black_card": 14000}', 'VVIP', 4, 16, 9, true),
      (6, '私人高爾夫錦標賽', '林口國際高爾夫俱樂部專屬比賽', '2025-09-15 08:00:00', '2025-09-10 17:00:00', 4, 5, 1, '{"platinum": 8000, "diamond": 6500, "black_card": 5000}', 'VIP', 2, 40, 22, true),
      (7, '慈善拍賣晚宴', '支持兒童教育基金會，頂級藝術品拍賣', '2025-09-20 18:00:00', '2025-09-17 20:00:00', 6, 6, 1, '{"platinum": 30000, "diamond": 25000, "black_card": 20000}', 'Invitation Only', 5, 50, 31, true),
      (8, '企業家論壇峰會', '亞洲頂尖企業家分享商業洞察', '2025-09-25 14:00:00', '2025-09-22 12:00:00', 7, 7, 1, '{"platinum": 15000, "diamond": 12000, "black_card": 10000}', 'VIP', 3, 80, 47, true),
      (9, '巴黎時裝週預覽', 'Hermès與Chanel最新系列私人展示', '2025-10-01 16:00:00', '2025-09-28 14:00:00', 1, 8, 1, '{"platinum": 35000, "diamond": 30000, "black_card": 25000}', 'VVIP', 5, 35, 18, true),
      (10, '米其林主廚聯合晚宴', '三位米其林三星主廚聯手創作', '2025-10-05 19:00:00', '2025-10-02 16:00:00', 2, 1, 1, '{"platinum": 28000, "diamond": 24000, "black_card": 20000}', 'Invitation Only', 5, 24, 14, true),
      (11, '茶藝文化體驗會', '台灣高山茶與日本抹茶道文化', '2025-10-10 14:30:00', '2025-10-07 12:00:00', 3, 3, 1, '{"platinum": 8000, "diamond": 6500, "black_card": 5000}', 'VIP', 2, 30, 11, true),
      (12, '限量超跑鑑賞會', 'Ferrari、Lamborghini最新車款私人預覽', '2025-10-15 10:00:00', '2025-10-12 18:00:00', 4, 7, 1, '{"platinum": 22000, "diamond": 18000, "black_card": 15000}', 'VVIP', 3, 45, 28, true)`)

    res.json({
      success: true,
      message: '12 luxury events created successfully',
      events: 12,
      categories: 8,
      venues: 8
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Expanded seeding failed'
    })
  }
})

router.get('/counts', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    const tables = ['users', 'venues', 'event_categories', 'events']
    const counts: Record<string, number | string> = {}

    for (const table of tables) {
      try {
        const result = await pool.query('SELECT COUNT(*) as count FROM ' + table)
        counts[table] = Number(result.rows[0]?.count) || 0
      } catch (error) {
        counts[table] = 'Error: ' + (error instanceof Error ? error.message : 'Unknown')
      }
    }

    res.json({
      success: true,
      counts
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    })
  }
})

export default router
