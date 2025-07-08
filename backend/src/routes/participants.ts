import { Router } from 'express'
import { AuthenticatedRequest, ApiResponse } from '../types/index.js'
import { authenticateToken as auth } from '../middleware/auth.js'
import ParticipantAccessService from '../services/participantAccessService.js'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'

const router = Router()
const participantService = new ParticipantAccessService(duckdb)

/**
 * GET /api/events/:eventId/participants
 * Get filtered list of event participants based on viewer's access level
 */
router.get('/events/:eventId/participants', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId } = req.params
    const userId = req.user!.userId
    
    // Query parameters for filtering and pagination
    const page = parseInt(req.query.page as string) || 1
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100) // Max 100 per page
    
    const filters = {
      membershipTier: req.query.membershipTier as string,
      profession: req.query.profession as string,
      minPrivacyLevel: req.query.minPrivacyLevel ? parseInt(req.query.minPrivacyLevel as string) : undefined
    }

    // Get client IP and user agent for logging
    const ipAddress = req.ip || req.connection.remoteAddress
    const userAgent = req.get('User-Agent')

    const result = participantService.getEventParticipants(
      eventId, 
      userId, 
      page, 
      limit, 
      filters
    )

    const response: ApiResponse = {
      success: true,
      data: result,
      pagination: {
        page,
        limit,
        total: result.totalCount,
        totalPages: Math.ceil(result.totalCount / limit)
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Error getting event participants:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to get event participants'
    }
    res.status(500).json(response)
  }
})

/**
 * GET /api/events/:eventId/participant-access
 * Check viewer's access level for viewing participants
 */
router.get('/events/:eventId/participant-access', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId } = req.params
    const userId = req.user!.userId

    const accessCheck = participantService.checkParticipantAccess(userId, eventId)

    const response: ApiResponse = {
      success: true,
      data: accessCheck
    }

    res.json(response)
  } catch (error) {
    logger.error('Error checking participant access:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to check participant access'
    }
    res.status(500).json(response)
  }
})

/**
 * GET /api/events/:eventId/participants/:participantId
 * Get detailed information about a specific participant (if access allows)
 */
router.get('/events/:eventId/participants/:participantId', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId, participantId } = req.params
    const userId = req.user!.userId

    // Check if viewer has access
    const accessCheck = participantService.checkParticipantAccess(userId, eventId)
    
    if (!accessCheck.hasAccess) {
      const response: ApiResponse = {
        success: false,
        error: 'Access denied - payment required to view participants'
      }
      return res.status(403).json(response)
    }

    // Get the specific participant with privacy filtering
    const result = participantService.getEventParticipants(
      eventId,
      userId,
      1,
      1,
      {} // No additional filters, we want this specific participant
    )

    const participant = result.participants.find(p => p.id === participantId)
    
    if (!participant) {
      const response: ApiResponse = {
        success: false,
        error: 'Participant not found or not visible'
      }
      return res.status(404).json(response)
    }

    // Log the profile view
    const ipAddress = req.ip || req.connection.remoteAddress
    const userAgent = req.get('User-Agent')
    
    const response: ApiResponse = {
      success: true,
      data: {
        participant,
        viewerAccess: accessCheck.accessLevel
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Error getting participant details:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to get participant details'
    }
    res.status(500).json(response)
  }
})

/**
 * POST /api/events/:eventId/participants/:participantId/contact
 * Initiate contact with a participant (if allowed)
 */
router.post('/events/:eventId/participants/:participantId/contact', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId, participantId } = req.params
    const userId = req.user!.userId
    const { message } = req.body

    // Check if viewer has access and can initiate contact
    const accessCheck = participantService.checkParticipantAccess(userId, eventId)
    
    if (!accessCheck.hasAccess || !accessCheck.accessLevel.canInitiateContact) {
      const response: ApiResponse = {
        success: false,
        error: 'Access denied - cannot initiate contact'
      }
      return res.status(403).json(response)
    }

    // Validate message
    if (!message || message.trim().length === 0) {
      const response: ApiResponse = {
        success: false,
        error: 'Message is required'
      }
      return res.status(400).json(response)
    }

    // TODO: Implement contact request system
    // This would typically create a contact request record, send notification, etc.
    // For now, we'll just log the attempt

    logger.info(`Contact initiated: ${userId} -> ${participantId} for event ${eventId}`)

    const response: ApiResponse = {
      success: true,
      message: 'Contact request sent successfully'
    }

    res.json(response)
  } catch (error) {
    logger.error('Error initiating contact:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to initiate contact'
    }
    res.status(500).json(response)
  }
})

/**
 * PUT /api/events/:eventId/privacy-settings
 * Update user's privacy settings for a specific event
 */
router.put('/events/:eventId/privacy-settings', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId } = req.params
    const userId = req.user!.userId
    const { privacyLevel, allowContact, showInList } = req.body

    // Validate privacy level
    if (privacyLevel !== undefined && (privacyLevel < 1 || privacyLevel > 5)) {
      const response: ApiResponse = {
        success: false,
        error: 'Privacy level must be between 1 and 5'
      }
      return res.status(400).json(response)
    }

    const db = duckdb
    
    // Update or insert privacy override
    const stmt = db.prepare(`
      INSERT INTO event_privacy_overrides (user_id, event_id, privacy_level, allow_contact, show_in_list)
      VALUES (?, ?, ?, ?, ?)
      ON CONFLICT(user_id, event_id) DO UPDATE SET
      privacy_level = COALESCE(excluded.privacy_level, privacy_level),
      allow_contact = COALESCE(excluded.allow_contact, allow_contact),
      show_in_list = COALESCE(excluded.show_in_list, show_in_list),
      updated_at = CURRENT_TIMESTAMP
    `)

    stmt.run(userId, eventId, privacyLevel, allowContact, showInList)

    const response: ApiResponse = {
      success: true,
      message: 'Privacy settings updated successfully'
    }

    res.json(response)
  } catch (error) {
    logger.error('Error updating privacy settings:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to update privacy settings'
    }
    res.status(500).json(response)
  }
})

/**
 * GET /api/events/:eventId/privacy-settings
 * Get user's current privacy settings for an event
 */
router.get('/events/:eventId/privacy-settings', auth, async (req: AuthenticatedRequest, res) => {
  try {
    const { eventId } = req.params
    const userId = req.user!.userId

    const db = duckdb
    
    const settings = db.prepare(`
      SELECT 
        COALESCE(epo.privacy_level, u.default_privacy_level, 2) as privacy_level,
        COALESCE(epo.allow_contact, u.allow_contact_requests, true) as allow_contact,
        COALESCE(epo.show_in_list, u.show_in_participant_lists, true) as show_in_list
      FROM users u
      LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ?
      WHERE u.id = ?
    `).get(eventId, userId)

    const response: ApiResponse = {
      success: true,
      data: settings
    }

    res.json(response)
  } catch (error) {
    logger.error('Error getting privacy settings:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to get privacy settings'
    }
    res.status(500).json(response)
  }
})

export default router