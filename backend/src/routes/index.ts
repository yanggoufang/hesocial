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