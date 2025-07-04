import { Router } from 'express'
import {
  getEvents,
  getEventById,
  getEventCategories,
  getVenues
} from '../controllers/eventController.js'
import { R2BackupService } from '../services/r2-backup.js'

const router = Router()

// Lazy initialization of backup service
let backupService: R2BackupService | null = null
const getBackupService = () => {
  if (!backupService) {
    backupService = new R2BackupService()
  }
  return backupService
}

// Event routes
router.get('/events', getEvents)
router.get('/events/categories', getEventCategories)
router.get('/events/venues', getVenues)
router.get('/events/:id', getEventById)

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

// Admin backup routes
router.post('/admin/backup', async (req, res) => {
  try {
    const backupId = await getBackupService().createManualBackup()
    const timestamp = new Date().toISOString()
    
    res.json({
      success: true,
      backupId,
      timestamp,
      message: 'Manual backup created successfully',
      tags: { type: 'manual' }
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Backup failed',
      message: 'Failed to create manual backup'
    })
  }
})

router.get('/admin/backups', async (req, res) => {
  try {
    const limit = parseInt(req.query.limit as string) || 20
    const backups = await getBackupService().listBackups(limit)
    
    res.json({
      success: true,
      backups,
      count: backups.length,
      message: 'Backups retrieved successfully'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to list backups',
      message: 'Failed to retrieve backup list'
    })
  }
})

router.put('/admin/backup/:id/status', async (req, res) => {
  try {
    const { id } = req.params
    const { status } = req.body
    
    if (status !== 'latest_restored') {
      return res.status(400).json({
        success: false,
        error: 'Invalid status',
        message: 'Status must be "latest_restored"'
      })
    }
    
    await getBackupService().updateBackupStatus(id, status)
    
    return res.json({
      success: true,
      message: 'Backup status updated successfully',
      backupId: id,
      status
    })
  } catch (error) {
    return res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Status update failed',
      message: 'Failed to update backup status'
    })
  }
})

router.delete('/admin/backups/cleanup', async (req, res) => {
  try {
    await getBackupService().cleanupOldBackups()
    
    res.json({
      success: true,
      message: 'Old backups cleaned up successfully'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Cleanup failed',
      message: 'Failed to cleanup old backups'
    })
  }
})

router.get('/admin/backup/test-connection', async (req, res) => {
  try {
    const connected = await getBackupService().testConnection()
    
    res.json({
      success: true,
      connected,
      message: connected ? 'R2 connection successful' : 'R2 connection failed'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Connection test failed',
      message: 'Failed to test R2 connection'
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
      events: 'Full functionality',
      authentication: 'Coming soon',
      payments: 'Coming soon'
    },
    endpoints: {
      events: '/api/events',
      categories: '/api/events/categories',
      venues: '/api/events/venues',
      health: '/api/health',
      admin: {
        backup: '/api/admin/backup',
        backups: '/api/admin/backups',
        testConnection: '/api/admin/backup/test-connection'
      }
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

// Auth placeholder routes
router.get('/auth/profile', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

router.post('/auth/login', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

router.post('/auth/register', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

// Debug route to insert minimal test data
router.post('/debug/insert-test-data', async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    
    // Just insert test events since other tables already have data
    await pool.query(`
      INSERT INTO events (id, name, description, date_time, registration_deadline, venue_id, category_id, organizer_id, pricing, exclusivity_level, dress_code, capacity, current_attendees, amenities, privacy_guarantees, images, requirements)
      VALUES
      (1, '測試活動1', '第一個測試活動', '2025-08-15 19:00:00', '2025-08-13 23:59:59', 1, 1, 1, '{"vip": 5000}', 'VIP', 3, 20, 5, '["測試"]', '["測試"]', '["test1.jpg"]', '[]'),
      (2, '測試活動2', '第二個測試活動', '2025-09-15 19:00:00', '2025-09-13 23:59:59', 1, 1, 1, '{"vip": 6000}', 'VIP', 3, 25, 8, '["測試"]', '["測試"]', '["test2.jpg"]', '[]'),
      (3, '測試活動3', '第三個測試活動', '2025-10-15 19:00:00', '2025-10-13 23:59:59', 1, 1, 1, '{"vip": 7000}', 'VVIP', 4, 15, 3, '["測試"]', '["測試"]', '["test3.jpg"]', '[]')
    `)
    
    res.json({
      success: true,
      message: 'Test events inserted successfully'
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to insert test data'
    })
  }
})

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