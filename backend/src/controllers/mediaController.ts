import { Request, Response } from 'express'
import multer from 'multer'
import { AuthenticatedRequest } from '../types/index.js'
import { duckdb } from '../database/duckdb-connection.js'
import { mediaService } from '../services/MediaService.js'
import logger from '../utils/logger.js'

// Configure multer for memory storage (files will be uploaded to R2)
const upload = multer({
  storage: multer.memoryStorage(),
  limits: {
    fileSize: 10 * 1024 * 1024, // 10MB limit
  },
  fileFilter: (req, file, cb) => {
    // Allow images and documents
    const allowedMimes = [
      'image/jpeg',
      'image/png',
      'image/webp',
      'image/gif',
      'application/pdf',
      'application/msword',
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'application/vnd.ms-excel',
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
    ]
    
    if (allowedMimes.includes(file.mimetype)) {
      cb(null, true)
    } else {
      cb(new Error('Invalid file type. Only images and documents are allowed.'))
    }
  }
})

// Middleware for handling multiple file uploads
export const uploadMiddleware = upload.fields([
  { name: 'eventImages', maxCount: 10 },
  { name: 'eventDocuments', maxCount: 5 },
  { name: 'venueImages', maxCount: 8 },
  { name: 'profilePicture', maxCount: 1 }
])

/**
 * POST /api/media/events/:eventId/images
 * Upload images for an event
 */
export const uploadEventImages = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const { eventId } = req.params
    const files = req.files as { [fieldname: string]: Express.Multer.File[] }

    if (!files?.eventImages || files.eventImages.length === 0) {
      res.status(400).json({
        success: false,
        error: 'No images provided'
      })
      return
    }

    // Check if event exists and user has permission
    const eventCheck = await duckdb.prepare(`
      SELECT id, organizer_id 
      FROM events 
      WHERE id = ? AND is_active = true
    `).get(eventId)

    if (!eventCheck) {
      res.status(404).json({
        success: false,
        error: 'Event not found'
      })
      return
    }

    // Check permissions
    if (req.user.role !== 'super_admin' && 
        req.user.role !== 'admin' && 
        eventCheck.organizer_id !== req.user.id) {
      res.status(403).json({
        success: false,
        error: 'Permission denied'
      })
      return
    }

    // Upload images using MediaService
    const uploadedImages = await mediaService.uploadEventImages(
      eventId,
      files.eventImages,
      req.user.id
    )

    res.json({
      success: true,
      data: {
        eventId,
        uploadedImages,
        count: uploadedImages.length
      }
    })
  } catch (error) {
    logger.error('Upload event images error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to upload images'
    })
  }
}

/**
 * POST /api/media/events/:eventId/documents
 * Upload documents for an event
 */
export const uploadEventDocuments = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const { eventId } = req.params
    const files = req.files as { [fieldname: string]: Express.Multer.File[] }

    if (!files?.eventDocuments || files.eventDocuments.length === 0) {
      res.status(400).json({
        success: false,
        error: 'No documents provided'
      })
      return
    }

    // Check if event exists and user has permission
    const eventCheck = await duckdb.prepare(`
      SELECT id, organizer_id 
      FROM events 
      WHERE id = ? AND is_active = true
    `).get(eventId)

    if (!eventCheck) {
      res.status(404).json({
        success: false,
        error: 'Event not found'
      })
      return
    }

    // Check permissions
    if (req.user.role !== 'super_admin' && 
        req.user.role !== 'admin' && 
        eventCheck.organizer_id !== req.user.id) {
      res.status(403).json({
        success: false,
        error: 'Permission denied'
      })
      return
    }

    // Upload documents using MediaService
    const uploadedDocuments = await mediaService.uploadEventDocuments(
      eventId,
      files.eventDocuments,
      req.user.id
    )

    res.json({
      success: true,
      data: {
        eventId,
        uploadedDocuments,
        count: uploadedDocuments.length
      }
    })
  } catch (error) {
    logger.error('Upload event documents error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to upload documents'
    })
  }
}

/**
 * GET /api/media/events/:eventId
 * Get all media for an event
 */
export const getEventMedia = async (req: Request, res: Response): Promise<void> => {
  try {
    const { eventId } = req.params
    const { type } = req.query

    let whereClause = 'WHERE event_id = ?'
    const params = [eventId]

    if (type && ['image', 'document'].includes(type as string)) {
      whereClause += ' AND type = ?'
      params.push(type as string)
    }

    const mediaQuery = `
      SELECT 
        id, event_id as "eventId", type, file_path as "filePath",
        thumbnail_path as "thumbnailPath", original_filename as "originalFilename",
        file_size as "fileSize", mime_type as "mimeType",
        uploaded_by as "uploadedBy", created_at as "createdAt"
      FROM event_media 
      ${whereClause}
      ORDER BY created_at DESC
    `

    const result = await duckdb.prepare(mediaQuery).all(...params)

    const media = await Promise.all(result.map(async (item: any) => {
      let filePath = item.filePath
      
      // For documents, generate signed URL if user is authenticated
      if (item.type === 'document') {
        const user = (req as any).user
        if (user) {
          try {
            filePath = await mediaService.getSignedUrl(item.filePath, 3600) // 1 hour expiry
          } catch (error) {
            logger.warn('Failed to generate signed URL for document:', error)
          }
        } else {
          // Hide document path for unauthenticated users
          filePath = null
        }
      }

      return {
        ...item,
        filePath,
        thumbnailPath: item.thumbnailPath ? JSON.parse(item.thumbnailPath) : null
      }
    }))

    res.json({
      success: true,
      data: media
    })
  } catch (error) {
    logger.error('Get event media error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get event media'
    })
  }
}

/**
 * DELETE /api/media/:mediaId
 * Delete a media file
 */
export const deleteMedia = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const { mediaId } = req.params

    // Get media record with event/venue info
    const media = await duckdb.prepare(`
      SELECT 
        em.*, e.organizer_id,
        'event' as media_source
      FROM event_media em
      JOIN events e ON em.event_id = e.id
      WHERE em.id = ?
      UNION ALL
      SELECT 
        vm.*, null as organizer_id,
        'venue' as media_source
      FROM venue_media vm
      WHERE vm.id = ?
    `).get(mediaId, mediaId)

    if (!media) {
      res.status(404).json({
        success: false,
        error: 'Media not found'
      })
      return
    }

    // Check permissions
    if (req.user.role !== 'super_admin' && 
        req.user.role !== 'admin' && 
        media.organizer_id !== req.user.id &&
        media.uploaded_by !== req.user.id) {
      res.status(403).json({
        success: false,
        error: 'Permission denied'
      })
      return
    }

    // Delete using MediaService (handles R2 cleanup and database removal)
    await mediaService.deleteMedia(mediaId)

    res.json({
      success: true,
      message: 'Media deleted successfully'
    })
  } catch (error) {
    logger.error('Delete media error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to delete media'
    })
  }
}

/**
 * POST /api/media/venues/:venueId/images
 * Upload images for a venue
 */
export const uploadVenueImages = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const { venueId } = req.params
    const files = req.files as { [fieldname: string]: Express.Multer.File[] }

    if (!files?.venueImages || files.venueImages.length === 0) {
      res.status(400).json({
        success: false,
        error: 'No images provided'
      })
      return
    }

    // Check if venue exists (admin only)
    if (!['admin', 'super_admin'].includes(req.user.role)) {
      res.status(403).json({
        success: false,
        error: 'Admin access required'
      })
      return
    }

    const venueCheck = await duckdb.prepare(`
      SELECT id FROM venues WHERE id = ? AND is_active = true
    `).get(venueId)

    if (!venueCheck) {
      res.status(404).json({
        success: false,
        error: 'Venue not found'
      })
      return
    }

    // Upload images using MediaService
    const uploadedImages = await mediaService.uploadVenueImages(
      venueId,
      files.venueImages,
      req.user.id
    )

    res.json({
      success: true,
      data: {
        venueId,
        uploadedImages,
        count: uploadedImages.length
      }
    })
  } catch (error) {
    logger.error('Upload venue images error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to upload venue images'
    })
  }
}

/**
 * GET /api/media/venues/:venueId
 * Get all media for a venue
 */
export const getVenueMedia = async (req: Request, res: Response): Promise<void> => {
  try {
    const { venueId } = req.params

    const mediaQuery = `
      SELECT 
        id, venue_id as "venueId", type, file_path as "filePath",
        thumbnail_path as "thumbnailPath", original_filename as "originalFilename",
        file_size as "fileSize", mime_type as "mimeType",
        uploaded_by as "uploadedBy", created_at as "createdAt"
      FROM venue_media 
      WHERE venue_id = ?
      ORDER BY created_at DESC
    `

    const result = await duckdb.prepare(mediaQuery).all(venueId)

    const media = result.map((item: any) => ({
      ...item,
      thumbnailPath: item.thumbnailPath ? JSON.parse(item.thumbnailPath) : null
    }))

    res.json({
      success: true,
      data: media
    })
  } catch (error) {
    logger.error('Get venue media error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get venue media'
    })
  }
}

/**
 * GET /api/media/download/:mediaId
 * Download a document (generates signed URL)
 */
export const downloadDocument = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const { mediaId } = req.params

    // Get document record
    const document = await duckdb.prepare(`
      SELECT * FROM event_media WHERE id = ? AND type = 'document'
    `).get(mediaId)

    if (!document) {
      res.status(404).json({
        success: false,
        error: 'Document not found'
      })
      return
    }

    // Generate signed URL for download
    const downloadUrl = await mediaService.getSignedUrl(document.file_path, 300) // 5 minutes

    res.json({
      success: true,
      data: {
        downloadUrl,
        filename: document.original_filename,
        expiresIn: 300
      }
    })
  } catch (error) {
    logger.error('Download document error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to generate download URL'
    })
  }
}