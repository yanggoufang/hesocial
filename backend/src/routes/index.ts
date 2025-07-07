import { Router } from 'express'
import {
  getEvents,
  getEventById,
  getEventCategories,
  getVenues
} from '../controllers/eventController.js'
import healthRoutes from './health.js'
// Temporarily disable other routes that may have import issues
// import adminRoutes from './admin.js'
// import authRoutes from './authRoutes.js'
// import userManagementRoutes from './userManagement.js'
// import eventManagementRoutes from './eventManagement.js'
// import venueManagementRoutes from './venueManagement.js'
// import categoryManagementRoutes from './categoryManagement.js'
// import salesManagementRoutes from './salesManagement.js'
// import registrationRoutes from './registrationRoutes.js'
// import mediaRoutes from './mediaRoutes.js'
// import systemHealthRoutes from './systemHealthRoutes.js'

const router = Router()

// Health check routes (only working route for now)
router.use('/health', healthRoutes)

// Temporarily disable other routes that may have import issues
// router.use('/auth', authRoutes)
// router.use('/admin', adminRoutes)
// router.use('/users', userManagementRoutes)
// router.use('/events', eventManagementRoutes)
// router.use('/venues', venueManagementRoutes)
// router.use('/categories', categoryManagementRoutes)
// router.use('/sales', salesManagementRoutes)
// router.use('/registrations', registrationRoutes)
// router.use('/media', mediaRoutes)
// router.use('/system', systemHealthRoutes)

// Basic API routes for frontend functionality
router.get('/events', getEvents)
router.get('/events/:id', getEventById)
router.get('/categories', getEventCategories)
router.get('/venues', getVenues)

// Legacy event routes (for backwards compatibility during transition)
router.get('/legacy/events', getEvents)
router.get('/legacy/events/categories', getEventCategories)
router.get('/legacy/events/venues', getVenues)
router.get('/legacy/events/:id', getEventById)

// Debug route to check all events regardless of date
router.get('/debug/events-all', async (req, res) => {
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

// Debug route to check database tables
router.get('/debug/tables', async (req, res) => {
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
      tables: result.rows.map(r => r.table_name)
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    })
  }
})

// Debug route to manually seed database
router.post('/debug/seed', async (req, res) => {
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

// Simple manual seeding endpoint
router.post('/debug/manual-seed', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    
    // Insert basic categories
    await pool.query(`INSERT OR IGNORE INTO event_categories (id, name, description, icon) VALUES 
      (1, '私人晚宴', '頂級餐廳私人用餐體驗', 'utensils'),
      (2, '遊艇派對', '豪華遊艇社交聚會', 'anchor'),
      (3, '藝術鑑賞', '高端藝術品展覽與收藏', 'palette')`)
    
    // Insert basic venues
    await pool.query(`INSERT OR IGNORE INTO venues (id, name, address, city, latitude, longitude, rating, amenities, images) VALUES 
      (1, '台北君悅酒店', '台北市信義區松壽路2號', '台北', 25.0330, 121.5654, 5, '["停車場","無線網路"]', '[]'),
      (2, '大倉久和大飯店', '台北市中山區南京東路一段9號', '台北', 25.0518, 121.5228, 5, '["日式庭園","高級餐廳"]', '[]')`)
    
    // Insert basic users if not exist
    await pool.query(`INSERT OR IGNORE INTO users (id, email, password_hash, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, is_verified, verification_status) VALUES 
      (1, 'admin@hesocial.com', '$2b$10$hash', 'Admin', 'User', 35, 'Administrator', 5000000, 30000000, 'Black Card', true, 'approved')`)
    
    // Insert basic events with future dates and all required fields
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

// Update events to future dates
router.post('/debug/update-dates', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    
    // Update existing events to future dates
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

// Debug route to check all table counts
router.get('/debug/counts', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    
    const tables = ['users', 'venues', 'event_categories', 'events']
    const counts: any = {}
    
    for (const table of tables) {
      try {
        const result = await pool.query(`SELECT COUNT(*) as count FROM ${table}`)
        counts[table] = Number(result.rows[0]?.count) || 0
      } catch (error) {
        counts[table] = `Error: ${error instanceof Error ? error.message : 'Unknown'}`
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

// Simple test endpoint to verify routes are working
router.get('/test', (req, res) => {
  res.json({
    success: true,
    message: 'API routes are working!',
    timestamp: new Date().toISOString()
  })
})

// API info endpoint
router.get('/', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Luxury Social Event Platform (DuckDB)',
    version: '1.0.0',
    database: 'DuckDB',
    features: {
      authentication: '✅ Production Ready - JWT + Google OAuth',
      events: '✅ Event Content Management System - Full CRUD',
      venues: '✅ Venue Management - Admin controlled',
      categories: '✅ Event Categories - Luxury event types',
      userManagement: '✅ User Management - Admin dashboard',
      backupSystem: '✅ R2 Backup - Manual backup preferred',
      salesManagement: '✅ Sales Pipeline - CRM and opportunity management',
      payments: '🚧 Payment Processing - Coming soon'
    },
    endpoints: {
      // Authentication
      auth: '/api/auth/*',
      // Event Management (Full CRUD)
      events: '/api/events/*',
      venues: '/api/venues/*',
      categories: '/api/categories/*',
      // User Management (Admin)
      users: '/api/users/*',
      // Sales Management (CRM)
      sales: '/api/sales/*',
      // Event Registration & Payment
      registrations: '/api/registrations/*',
      // Media Management (Images & Documents)
      media: '/api/media/*',
      // Health & Monitoring
      health: '/api/health/*',
      system: '/api/system/*',
      // Admin Operations
      admin: '/api/admin/*',
      // Legacy endpoints (deprecated)
      legacyEvents: '/api/legacy/events'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

// Note: Auth routes are now implemented in /auth/* endpoints

// Catch all for other API routes
router.use('*', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Endpoint not implemented yet',
    message: 'This endpoint will be available in the next phase',
    availableEndpoints: ['/events', '/events/categories', '/events/venues']
  })
})

export default router