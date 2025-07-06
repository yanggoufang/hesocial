import { Router } from 'express'
import {
  getEvents,
  getEventById,
  getEventCategories,
  getVenues
} from '../controllers/eventController.js'
import healthRoutes from './health.js'
import adminRoutes from './admin.js'
import authRoutes from './authRoutes.js'
import userManagementRoutes from './userManagement.js'
import eventManagementRoutes from './eventManagement.js'
import venueManagementRoutes from './venueManagement.js'
import categoryManagementRoutes from './categoryManagement.js'
import salesManagementRoutes from './salesManagement.js'
import registrationRoutes from './registrationRoutes.js'

const router = Router()

// Authentication routes
router.use('/auth', authRoutes)

// Health check routes
router.use('/health', healthRoutes)

// Admin routes
router.use('/admin', adminRoutes)

// User management routes (admin only)
router.use('/users', userManagementRoutes)

// Event management routes (comprehensive CRUD system)
router.use('/events', eventManagementRoutes)

// Venue management routes (admin-controlled)
router.use('/venues', venueManagementRoutes)

// Event category management routes (admin-controlled)  
router.use('/categories', categoryManagementRoutes)

// Sales management routes (CRM and sales pipeline)
router.use('/sales', salesManagementRoutes)

// Event registration routes (member registration and payment)
router.use('/registrations', registrationRoutes)

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
      // Health & Monitoring
      health: '/api/health/*',
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