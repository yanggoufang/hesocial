import { Response } from 'express'
import { v4 as uuidv4 } from 'uuid'
import { pool } from '@/database/connection.js'
import { AuthenticatedRequest } from '@/types/index.js'
import logger from '@/utils/logger.js'

export const registerForEvent = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { eventId, specialRequests } = req.body

    // Check if event exists and is available for registration
    const eventQuery = `
      SELECT 
        id, name, capacity, current_attendees, 
        registration_deadline, exclusivity_level,
        requirements, pricing
      FROM events 
      WHERE id = $1 AND is_active = true
    `

    const eventResult = await pool.query(eventQuery, [eventId])

    if (eventResult.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'Event not found'
      })
      return
    }

    const event = eventResult.rows[0]

    // Check registration deadline
    if (new Date() > new Date(event.registration_deadline)) {
      res.status(400).json({
        success: false,
        error: 'Registration deadline has passed'
      })
      return
    }

    // Check capacity
    if (event.current_attendees >= event.capacity) {
      res.status(400).json({
        success: false,
        error: 'Event is at full capacity'
      })
      return
    }

    // Check if user is already registered
    const existingRegistration = await pool.query(
      'SELECT id FROM registrations WHERE user_id = $1 AND event_id = $2',
      [req.user.id, eventId]
    )

    if (existingRegistration.rows.length > 0) {
      res.status(409).json({
        success: false,
        error: 'Already registered for this event'
      })
      return
    }

    // Check event requirements
    const requirements = event.requirements || []
    for (const requirement of requirements) {
      switch (requirement.type) {
        case 'age':
          const [minAge, maxAge] = requirement.value.split('-').map(Number)
          if (req.user.age < minAge || req.user.age > maxAge) {
            res.status(403).json({
              success: false,
              error: `Age requirement not met: ${requirement.description}`
            })
            return
          }
          break

        case 'income':
          if (req.user.annualIncome < Number(requirement.value)) {
            res.status(403).json({
              success: false,
              error: `Income requirement not met: ${requirement.description}`
            })
            return
          }
          break

        case 'membership':
          const requiredTiers = ['Platinum', 'Diamond', 'Black Card']
          const userTierIndex = requiredTiers.indexOf(req.user.membershipTier)
          const requiredTierIndex = requiredTiers.indexOf(requirement.value)
          
          if (userTierIndex < requiredTierIndex) {
            res.status(403).json({
              success: false,
              error: `Membership requirement not met: ${requirement.description}`
            })
            return
          }
          break

        case 'verification':
          if (!req.user.isVerified || req.user.verificationStatus !== 'approved') {
            res.status(403).json({
              success: false,
              error: `Verification requirement not met: ${requirement.description}`
            })
            return
          }
          break
      }
    }

    // Create registration
    const registrationId = uuidv4()
    const registrationQuery = `
      INSERT INTO registrations (
        id, user_id, event_id, special_requests, status
      ) VALUES ($1, $2, $3, $4, $5)
      RETURNING 
        id, user_id as "userId", event_id as "eventId",
        status, payment_status as "paymentStatus",
        special_requests as "specialRequests",
        created_at as "createdAt"
    `

    // For VVIP and Invitation Only events, set status to pending for manual approval
    const initialStatus = event.exclusivity_level === 'VIP' ? 'approved' : 'pending'

    const result = await pool.query(registrationQuery, [
      registrationId, req.user.id, eventId, specialRequests, initialStatus
    ])

    // Log registration
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'event_registration',
        'registration',
        registrationId,
        JSON.stringify({ 
          eventId, 
          eventName: event.name, 
          exclusivityLevel: event.exclusivity_level,
          status: initialStatus
        }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Event registration created:', { 
      registrationId, 
      userId: req.user.id, 
      eventId, 
      status: initialStatus 
    })

    const message = initialStatus === 'pending' 
      ? 'Registration submitted and pending approval'
      : 'Registration successful'

    res.status(201).json({
      success: true,
      message,
      data: result.rows[0]
    })
  } catch (error) {
    logger.error('Event registration error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to register for event'
    })
  }
}

export const getUserRegistrations = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { page = 1, limit = 20, status } = req.query
    const offset = (Number(page) - 1) * Number(limit)

    let whereConditions = ['r.user_id = $1']
    const queryParams: any[] = [req.user.id]
    let paramCount = 1

    if (status) {
      paramCount++
      whereConditions.push(`r.status = $${paramCount}`)
      queryParams.push(status)
    }

    const whereClause = whereConditions.join(' AND ')

    const registrationsQuery = `
      SELECT 
        r.id, r.status, r.payment_status as "paymentStatus",
        r.special_requests as "specialRequests",
        r.created_at as "createdAt", r.updated_at as "updatedAt",
        e.id as "eventId", e.name as "eventName", e.description as "eventDescription",
        e.date_time as "eventDateTime", e.pricing as "eventPricing",
        e.exclusivity_level as "eventExclusivityLevel",
        e.images as "eventImages",
        v.name as "venueName", v.address as "venueAddress",
        ec.name as "categoryName"
      FROM registrations r
      JOIN events e ON r.event_id = e.id
      JOIN venues v ON e.venue_id = v.id
      JOIN event_categories ec ON e.category_id = ec.id
      WHERE ${whereClause}
      ORDER BY r.created_at DESC
      LIMIT $${paramCount + 1} OFFSET $${paramCount + 2}
    `

    queryParams.push(Number(limit), offset)

    // Get total count
    const countQuery = `
      SELECT COUNT(*) as total
      FROM registrations r
      JOIN events e ON r.event_id = e.id
      WHERE ${whereClause}
    `

    const [registrationsResult, countResult] = await Promise.all([
      pool.query(registrationsQuery, queryParams),
      pool.query(countQuery, queryParams.slice(0, -2))
    ])

    const registrations = registrationsResult.rows.map(reg => ({
      id: reg.id,
      status: reg.status,
      paymentStatus: reg.paymentStatus,
      specialRequests: reg.specialRequests,
      createdAt: reg.createdAt,
      updatedAt: reg.updatedAt,
      event: {
        id: reg.eventId,
        name: reg.eventName,
        description: reg.eventDescription,
        dateTime: reg.eventDateTime,
        pricing: reg.eventPricing,
        exclusivityLevel: reg.eventExclusivityLevel,
        images: reg.eventImages,
        venue: {
          name: reg.venueName,
          address: reg.venueAddress
        },
        category: {
          name: reg.categoryName
        }
      }
    }))

    const total = parseInt(countResult.rows[0].total)
    const totalPages = Math.ceil(total / Number(limit))

    res.json({
      success: true,
      data: registrations,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages
      }
    })
  } catch (error) {
    logger.error('Get user registrations error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get registrations'
    })
  }
}

export const cancelRegistration = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { registrationId } = req.params

    // Check if registration exists and belongs to user
    const registrationQuery = `
      SELECT r.id, r.status, r.payment_status, e.name as event_name, e.date_time
      FROM registrations r
      JOIN events e ON r.event_id = e.id
      WHERE r.id = $1 AND r.user_id = $2
    `

    const result = await pool.query(registrationQuery, [registrationId, req.user.id])

    if (result.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
      return
    }

    const registration = result.rows[0]

    // Check if event is in the past
    if (new Date(registration.date_time) < new Date()) {
      res.status(400).json({
        success: false,
        error: 'Cannot cancel registration for past events'
      })
      return
    }

    // Check if already cancelled
    if (registration.status === 'cancelled') {
      res.status(400).json({
        success: false,
        error: 'Registration already cancelled'
      })
      return
    }

    // Update registration status
    await pool.query(
      'UPDATE registrations SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2',
      ['cancelled', registrationId]
    )

    // Log cancellation
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'registration_cancelled',
        'registration',
        registrationId,
        JSON.stringify({ 
          eventName: registration.event_name,
          paymentStatus: registration.payment_status
        }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Registration cancelled:', { 
      registrationId, 
      userId: req.user.id,
      eventName: registration.event_name
    })

    res.json({
      success: true,
      message: 'Registration cancelled successfully'
    })
  } catch (error) {
    logger.error('Cancel registration error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to cancel registration'
    })
  }
}