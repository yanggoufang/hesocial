import { Router } from 'express'
import { authenticateToken, requireAdmin } from '../middleware/auth.js'
import {
  uploadMiddleware,
  uploadEventImages,
  uploadEventDocuments,
  getEventMedia,
  deleteMedia,
  uploadVenueImages,
  getVenueMedia,
  downloadDocument
} from '../controllers/mediaController.js'

const router = Router()

// Event Media Routes
/**
 * POST /api/media/events/:eventId/images
 * Upload images for an event
 * Requires authentication and proper permissions
 */
router.post(
  '/events/:eventId/images',
  authenticateToken,
  uploadMiddleware,
  uploadEventImages as (req: Request, res: Response) => Promise<void>
)

/**
 * POST /api/media/events/:eventId/documents
 * Upload documents for an event
 * Requires authentication and proper permissions
 */
router.post(
  '/events/:eventId/documents',
  authenticateToken,
  uploadMiddleware,
  uploadEventDocuments as (req: Request, res: Response) => Promise<void>
)

/**
 * GET /api/media/events/:eventId
 * Get all media for an event
 * Public endpoint but respects event visibility
 */
router.get('/events/:eventId', getEventMedia)

// Venue Media Routes
/**
 * POST /api/media/venues/:venueId/images
 * Upload images for a venue
 * Requires admin authentication
 */
router.post(
  '/venues/:venueId/images',
  authenticateToken,
  requireAdmin,
  uploadMiddleware,
  uploadVenueImages as (req: Request, res: Response) => Promise<void>
)

/**
 * GET /api/media/venues/:venueId
 * Get all media for a venue
 * Public endpoint
 */
router.get('/venues/:venueId', getVenueMedia)

// General Media Routes
/**
 * DELETE /api/media/:mediaId
 * Delete a media file
 * Requires authentication and proper permissions
 */
router.delete('/:mediaId', authenticateToken, deleteMedia as (req: Request, res: Response) => Promise<void>)

/**
 * GET /api/media/download/:mediaId
 * Download a document (generates signed URL)
 * Requires authentication
 */
router.get('/download/:mediaId', authenticateToken, downloadDocument as (req: Request, res: Response) => Promise<void>)

/**
 * GET /api/media/stats
 * Get media statistics for admin dashboard
 * Requires admin authentication
 */
router.get('/stats', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { duckdb } = await import('../database/duckdb-connection.js')
    
    // Get media statistics
    const stats = await Promise.all([
      // Total event media
      duckdb.prepare(`
        SELECT 
          COUNT(*) as total,
          COUNT(CASE WHEN type = 'image' THEN 1 END) as images,
          COUNT(CASE WHEN type = 'document' THEN 1 END) as documents,
          COALESCE(SUM(file_size), 0) as totalSize
        FROM event_media
      `).get(),
      
      // Total venue media
      duckdb.prepare(`
        SELECT 
          COUNT(*) as total,
          COALESCE(SUM(file_size), 0) as totalSize
        FROM venue_media
      `).get(),
      
      // Recent uploads (last 30 days)
      duckdb.prepare(`
        SELECT COUNT(*) as recentUploads
        FROM event_media 
        WHERE created_at > datetime('now', '-30 days')
      `).get(),
      
      // Top uploaders
      duckdb.prepare(`
        SELECT 
          u.first_name || ' ' || u.last_name as name,
          COUNT(*) as uploadCount
        FROM event_media em
        JOIN users u ON em.uploaded_by = u.id
        GROUP BY em.uploaded_by, u.first_name, u.last_name
        ORDER BY uploadCount DESC
        LIMIT 5
      `).all()
    ])

    const [eventStats, venueStats, recentStats, topUploaders] = stats

    res.json({
      success: true,
      data: {
        eventMedia: {
          total: eventStats.total,
          images: eventStats.images,
          documents: eventStats.documents,
          totalSize: eventStats.totalSize
        },
        venueMedia: {
          total: venueStats.total,
          totalSize: venueStats.totalSize
        },
        recentUploads: recentStats.recentUploads,
        topUploaders: topUploaders,
        totalFiles: eventStats.total + venueStats.total,
        totalStorageUsed: eventStats.totalSize + venueStats.totalSize
      }
    })
  } catch (error) {
    console.error('Get media stats error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get media statistics'
    })
  }
})

/**
 * POST /api/media/cleanup
 * Clean up orphaned media files
 * Requires super admin authentication
 */
router.post('/cleanup', authenticateToken, async (req, res) => {
  try {
    const user = (req as any).user
    if (user.role !== 'super_admin') {
      res.status(403).json({
        success: false,
        error: 'Super admin access required'
      })
      return
    }

    const { duckdb } = await import('../database/duckdb-connection.js')
    const fs = await import('fs/promises')
    const path = await import('path')

    // Find orphaned event media (events that no longer exist)
    const orphanedEventMedia = await duckdb.prepare(`
      SELECT em.id, em.file_path, em.thumbnail_path
      FROM event_media em
      LEFT JOIN events e ON em.event_id = e.id
      WHERE e.id IS NULL OR e.is_active = false
    `).all()

    // Find orphaned venue media (venues that no longer exist)
    const orphanedVenueMedia = await duckdb.prepare(`
      SELECT vm.id, vm.file_path, vm.thumbnail_path
      FROM venue_media vm
      LEFT JOIN venues v ON vm.venue_id = v.id
      WHERE v.id IS NULL OR v.is_active = false
    `).all()

    let deletedFiles = 0
    let deletedRecords = 0

    // Clean up orphaned event media
    for (const media of orphanedEventMedia) {
      try {
        // Delete files
        const fullPath = path.join(process.cwd(), media.file_path.substring(1))
        await fs.unlink(fullPath)
        deletedFiles++

        // Delete thumbnails
        if (media.thumbnail_path) {
          const thumbnails = JSON.parse(media.thumbnail_path)
          for (const thumbnailPath of Object.values(thumbnails) as string[]) {
            try {
              const thumbFullPath = path.join(process.cwd(), thumbnailPath.substring(1))
              await fs.unlink(thumbFullPath)
              deletedFiles++
            } catch (error) {
              // Ignore thumbnail deletion errors
            }
          }
        }

        // Delete database record
        await duckdb.prepare('DELETE FROM event_media WHERE id = ?').run(media.id)
        deletedRecords++
      } catch (error) {
        console.warn(`Failed to delete media ${media.id}:`, error)
      }
    }

    // Clean up orphaned venue media
    for (const media of orphanedVenueMedia) {
      try {
        // Delete files
        const fullPath = path.join(process.cwd(), media.file_path.substring(1))
        await fs.unlink(fullPath)
        deletedFiles++

        // Delete thumbnails
        if (media.thumbnail_path) {
          const thumbnails = JSON.parse(media.thumbnail_path)
          for (const thumbnailPath of Object.values(thumbnails) as string[]) {
            try {
              const thumbFullPath = path.join(process.cwd(), thumbnailPath.substring(1))
              await fs.unlink(thumbFullPath)
              deletedFiles++
            } catch (error) {
              // Ignore thumbnail deletion errors
            }
          }
        }

        // Delete database record
        await duckdb.prepare('DELETE FROM venue_media WHERE id = ?').run(media.id)
        deletedRecords++
      } catch (error) {
        console.warn(`Failed to delete venue media ${media.id}:`, error)
      }
    }

    res.json({
      success: true,
      data: {
        deletedFiles,
        deletedRecords,
        orphanedEventMedia: orphanedEventMedia.length,
        orphanedVenueMedia: orphanedVenueMedia.length
      }
    })
  } catch (error) {
    console.error('Media cleanup error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to clean up media'
    })
  }
})

export default router