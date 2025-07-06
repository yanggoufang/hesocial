import { Request, Response } from 'express'
import { pool } from '../database/duckdb-pool.js'
import { Registration, ApiResponse, AuthenticatedRequest } from '../types/index.js'
import { logger } from '../utils/logger.js'

// Event Registration Management
export const registerForEvent = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { eventId } = req.params
    const { specialRequests } = req.body
    const userId = req.user?.id

    if (!userId) {
      return res.status(401).json({
        success: false,
        error: 'User authentication required'
      })
    }

    // Check if event exists and is available for registration
    const eventQuery = `
      SELECT 
        e.*,
        v.name as venue_name,
        ec.name as category_name
      FROM events e
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      WHERE e.id = ? AND e.is_active = true
    `
    const eventResult = await pool.query(eventQuery, [eventId])
    const event = eventResult.rows[0]

    if (!event) {
      return res.status(404).json({
        success: false,
        error: 'Event not found or not available for registration'
      })
    }

    // Check if registration deadline has passed
    const now = new Date()
    const registrationDeadline = new Date(event.registration_deadline)
    if (now > registrationDeadline) {
      return res.status(400).json({
        success: false,
        error: 'Registration deadline has passed'
      })
    }

    // Check if event is full
    if (event.current_attendees >= event.capacity) {
      return res.status(400).json({
        success: false,
        error: 'Event is at full capacity'
      })
    }

    // Check if user is already registered
    const existingRegistrationQuery = `
      SELECT id FROM registrations 
      WHERE user_id = ? AND event_id = ?
    `
    const existingResult = await pool.query(existingRegistrationQuery, [userId, eventId])
    if (existingResult.rows.length > 0) {
      return res.status(400).json({
        success: false,
        error: 'You are already registered for this event'
      })
    }

    // Get user information for membership validation
    const userQuery = `
      SELECT membership_tier, is_verified, verification_status
      FROM users 
      WHERE id = ?
    `
    const userResult = await pool.query(userQuery, [userId])
    const user = userResult.rows[0]

    if (!user) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    // Check membership tier requirements
    const eventRequirements = JSON.parse(event.requirements || '[]')
    const membershipRequirement = eventRequirements.find((req: any) => req.type === 'membership')
    
    if (membershipRequirement) {
      const allowedTiers = membershipRequirement.value
      if (!allowedTiers.includes(user.membership_tier)) {
        return res.status(403).json({
          success: false,
          error: `This event requires ${allowedTiers.join(' or ')} membership`
        })
      }
    }

    // Check verification requirement
    const verificationRequirement = eventRequirements.find((req: any) => req.type === 'verification')
    if (verificationRequirement && verificationRequirement.value === true) {
      if (!user.is_verified || user.verification_status !== 'approved') {
        return res.status(403).json({
          success: false,
          error: 'This event requires verified membership status'
        })
      }
    }

    // Create registration
    const registrationId = Date.now()
    const insertQuery = `
      INSERT INTO registrations (
        id, user_id, event_id, status, payment_status, special_requests,
        created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `
    
    await pool.query(insertQuery, [
      registrationId,
      userId,
      eventId,
      'pending',
      'pending',
      specialRequests || null,
      new Date().toISOString(),
      new Date().toISOString()
    ])

    // Update event current attendees count
    await pool.query(`
      UPDATE events 
      SET current_attendees = current_attendees + 1,
          updated_at = ?
      WHERE id = ?
    `, [new Date().toISOString(), eventId])

    logger.info(`User ${userId} registered for event ${eventId}`)

    res.status(201).json({
      success: true,
      data: {
        registrationId,
        status: 'pending',
        message: 'Registration submitted successfully. Pending approval.'
      }
    })
  } catch (error) {
    logger.error('Error registering for event:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to register for event'
    })
  }
}

export const getUserRegistrations = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const userId = req.user?.id
    const { page = 1, limit = 20, status } = req.query

    if (!userId) {
      return res.status(401).json({
        success: false,
        error: 'User authentication required'
      })
    }

    let whereClause = 'WHERE r.user_id = ?'
    const params: any[] = [userId]

    if (status) {
      whereClause += ' AND r.status = ?'
      params.push(status)
    }

    const offset = (Number(page) - 1) * Number(limit)

    // Get total count
    const countQuery = `SELECT COUNT(*) as total FROM registrations r ${whereClause}`
    const countResult = await pool.query(countQuery, params)
    const total = Number(countResult.rows[0]?.total) || 0

    // Get registrations with event details
    const registrationsQuery = `
      SELECT 
        r.*,
        e.name as event_name,
        e.description as event_description,
        e.date_time as event_date_time,
        e.exclusivity_level,
        e.dress_code,
        e.pricing,
        v.name as venue_name,
        v.address as venue_address,
        ec.name as category_name
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      ${whereClause}
      ORDER BY r.created_at DESC
      LIMIT ? OFFSET ?
    `

    const result = await pool.query(registrationsQuery, [...params, limit, offset])

    const response: ApiResponse<Registration[]> = {
      success: true,
      data: result.rows,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages: Math.ceil(total / Number(limit))
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Error fetching user registrations:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch registrations'
    })
  }
}

export const getRegistrationById = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { id } = req.params
    const userId = req.user?.id

    if (!userId) {
      return res.status(401).json({
        success: false,
        error: 'User authentication required'
      })
    }

    const query = `
      SELECT 
        r.*,
        e.name as event_name,
        e.description as event_description,
        e.date_time as event_date_time,
        e.registration_deadline,
        e.exclusivity_level,
        e.dress_code,
        e.capacity,
        e.current_attendees,
        e.pricing,
        e.amenities,
        e.privacy_guarantees,
        e.requirements,
        e.images as event_images,
        v.name as venue_name,
        v.address as venue_address,
        v.latitude,
        v.longitude,
        v.amenities as venue_amenities,
        v.images as venue_images,
        ec.name as category_name,
        ec.description as category_description
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      WHERE r.id = ? AND r.user_id = ?
    `

    const result = await pool.query(query, [id, userId])

    if (result.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
    }

    res.json({
      success: true,
      data: result.rows[0]
    })
  } catch (error) {
    logger.error('Error fetching registration details:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch registration details'
    })
  }
}

export const updateRegistration = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { id } = req.params
    const { specialRequests } = req.body
    const userId = req.user?.id

    if (!userId) {
      return res.status(401).json({
        success: false,
        error: 'User authentication required'
      })
    }

    // Check if registration exists and belongs to user
    const existingQuery = `
      SELECT r.*, e.registration_deadline
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      WHERE r.id = ? AND r.user_id = ?
    `
    const existingResult = await pool.query(existingQuery, [id, userId])

    if (existingResult.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
    }

    const registration = existingResult.rows[0]

    // Check if registration can still be modified
    if (registration.status === 'approved' || registration.status === 'rejected') {
      return res.status(400).json({
        success: false,
        error: 'Cannot modify registration after approval/rejection'
      })
    }

    // Check if registration deadline has passed
    const now = new Date()
    const registrationDeadline = new Date(registration.registration_deadline)
    if (now > registrationDeadline) {
      return res.status(400).json({
        success: false,
        error: 'Cannot modify registration after deadline'
      })
    }

    // Update registration
    const updateQuery = `
      UPDATE registrations 
      SET special_requests = ?, updated_at = ?
      WHERE id = ? AND user_id = ?
    `

    await pool.query(updateQuery, [
      specialRequests || null,
      new Date().toISOString(),
      id,
      userId
    ])

    logger.info(`Registration ${id} updated by user ${userId}`)

    res.json({
      success: true,
      message: 'Registration updated successfully'
    })
  } catch (error) {
    logger.error('Error updating registration:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to update registration'
    })
  }
}

export const cancelRegistration = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const { id } = req.params
    const userId = req.user?.id

    if (!userId) {
      return res.status(401).json({
        success: false,
        error: 'User authentication required'
      })
    }

    // Check if registration exists and belongs to user
    const existingQuery = `
      SELECT r.*, e.date_time, e.id as event_id
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      WHERE r.id = ? AND r.user_id = ?
    `
    const existingResult = await pool.query(existingQuery, [id, userId])

    if (existingResult.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
    }

    const registration = existingResult.rows[0]

    // Check if event has already occurred
    const now = new Date()
    const eventDate = new Date(registration.date_time)
    if (now > eventDate) {
      return res.status(400).json({
        success: false,
        error: 'Cannot cancel registration for past events'
      })
    }

    // Check cancellation policy (24 hours before event)
    const timeDifference = eventDate.getTime() - now.getTime()
    const hoursUntilEvent = timeDifference / (1000 * 3600)
    
    if (hoursUntilEvent < 24) {
      return res.status(400).json({
        success: false,
        error: 'Cannot cancel registration within 24 hours of event'
      })
    }

    // Update registration status
    const updateQuery = `
      UPDATE registrations 
      SET status = 'cancelled', updated_at = ?
      WHERE id = ? AND user_id = ?
    `

    await pool.query(updateQuery, [
      new Date().toISOString(),
      id,
      userId
    ])

    // Update event current attendees count
    await pool.query(`
      UPDATE events 
      SET current_attendees = GREATEST(current_attendees - 1, 0),
          updated_at = ?
      WHERE id = ?
    `, [new Date().toISOString(), registration.event_id])

    logger.info(`Registration ${id} cancelled by user ${userId}`)

    res.json({
      success: true,
      message: 'Registration cancelled successfully'
    })
  } catch (error) {
    logger.error('Error cancelling registration:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to cancel registration'
    })
  }
}

// Admin functions for registration management
export const getAllRegistrations = async (req: Request, res: Response) => {
  try {
    const { 
      page = 1, 
      limit = 20, 
      status, 
      eventId, 
      userId,
      paymentStatus,
      search 
    } = req.query

    let whereClause = '1=1'
    const params: any[] = []

    if (status) {
      whereClause += ' AND r.status = ?'
      params.push(status)
    }

    if (eventId) {
      whereClause += ' AND r.event_id = ?'
      params.push(eventId)
    }

    if (userId) {
      whereClause += ' AND r.user_id = ?'
      params.push(userId)
    }

    if (paymentStatus) {
      whereClause += ' AND r.payment_status = ?'
      params.push(paymentStatus)
    }

    if (search) {
      whereClause += ' AND (e.name ILIKE ? OR u.first_name ILIKE ? OR u.last_name ILIKE ? OR u.email ILIKE ?)'
      const searchTerm = `%${search}%`
      params.push(searchTerm, searchTerm, searchTerm, searchTerm)
    }

    const offset = (Number(page) - 1) * Number(limit)

    // Get total count
    const countQuery = `
      SELECT COUNT(*) as total 
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      LEFT JOIN users u ON r.user_id = u.id
      WHERE ${whereClause}
    `
    const countResult = await pool.query(countQuery, params)
    const total = Number(countResult.rows[0]?.total) || 0

    // Get registrations with detailed information
    const registrationsQuery = `
      SELECT 
        r.*,
        e.name as event_name,
        e.date_time as event_date_time,
        e.pricing,
        u.first_name,
        u.last_name,
        u.email,
        u.membership_tier,
        v.name as venue_name,
        ec.name as category_name
      FROM registrations r
      LEFT JOIN events e ON r.event_id = e.id
      LEFT JOIN users u ON r.user_id = u.id
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      WHERE ${whereClause}
      ORDER BY r.created_at DESC
      LIMIT ? OFFSET ?
    `

    const result = await pool.query(registrationsQuery, [...params, limit, offset])

    const response: ApiResponse<Registration[]> = {
      success: true,
      data: result.rows,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages: Math.ceil(total / Number(limit))
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Error fetching all registrations:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch registrations'
    })
  }
}

export const approveRegistration = async (req: Request, res: Response) => {
  try {
    const { id } = req.params
    const { approved, notes } = req.body

    const status = approved ? 'approved' : 'rejected'

    // Update registration status
    const updateQuery = `
      UPDATE registrations 
      SET status = ?, updated_at = ?
      WHERE id = ?
    `

    const result = await pool.query(updateQuery, [
      status,
      new Date().toISOString(),
      id
    ])

    if (result.rowCount === 0) {
      return res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
    }

    logger.info(`Registration ${id} ${status} by admin`)

    res.json({
      success: true,
      message: `Registration ${status} successfully`
    })
  } catch (error) {
    logger.error('Error approving registration:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to update registration status'
    })
  }
}

export const updatePaymentStatus = async (req: Request, res: Response) => {
  try {
    const { id } = req.params
    const { paymentStatus, paymentIntentId } = req.body

    const updateQuery = `
      UPDATE registrations 
      SET payment_status = ?, payment_intent_id = ?, updated_at = ?
      WHERE id = ?
    `

    const result = await pool.query(updateQuery, [
      paymentStatus,
      paymentIntentId || null,
      new Date().toISOString(),
      id
    ])

    if (result.rowCount === 0) {
      return res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
    }

    logger.info(`Registration ${id} payment status updated to ${paymentStatus}`)

    res.json({
      success: true,
      message: 'Payment status updated successfully'
    })
  } catch (error) {
    logger.error('Error updating payment status:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to update payment status'
    })
  }
}