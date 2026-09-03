import { Router, Request, Response } from 'express'
import { authenticateToken, requireAdmin, requireSuperAdmin } from '../middleware/auth.js'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'

const router = Router()

/**
 * GET /api/events
 * Get all events with pagination, filtering, and search
 * Public endpoint with role-based data visibility
 */
router.get('/', async (req, res) => {
  try {
    const page = parseInt(req.query.page as string) || 1
    const limit = parseInt(req.query.limit as string) || 20
    const search = req.query.search as string || ''
    const category = req.query.category as string || ''
    const status = req.query.status as string || 'published'
    const venue = req.query.venue as string || ''
    const startDate = req.query.startDate as string || ''
    const endDate = req.query.endDate as string || ''
    const membershipTier = req.query.membershipTier as string || ''

    const offset = (page - 1) * limit

    let whereConditions = []
    let params: any[] = []

    // Base visibility - only published events for public, all for admins
    const user = (req as any).user
    if (!user || !['admin', 'super_admin'].includes(user.role)) {
      whereConditions.push(`e.status = 'published'`)
      whereConditions.push(`e.start_datetime > CURRENT_TIMESTAMP`)
    } else if (status) {
      whereConditions.push(`e.status = ?`)
      params.push(status)
    }

    if (search) {
      whereConditions.push(`(e.title ILIKE ? OR e.description ILIKE ?)`)
      params.push(`%${search}%`, `%${search}%`)
    }

    if (category) {
      whereConditions.push(`ec.slug = ?`)
      params.push(category)
    }

    if (venue) {
      whereConditions.push(`v.id = ?`)
      params.push(venue)
    }

    if (startDate) {
      whereConditions.push(`e.start_datetime >= ?`)
      params.push(startDate)
    }

    if (endDate) {
      whereConditions.push(`e.start_datetime <= ?`)
      params.push(endDate)
    }

    if (membershipTier) {
      whereConditions.push(`JSON_EXTRACT(e.required_membership_tiers, '$') LIKE ?`)
      params.push(`%"${membershipTier}"%`)
    }

    const whereClause = whereConditions.length > 0 ? `WHERE ${whereConditions.join(' AND ')}` : ''

    // Get total count
    const countQuery = `
      SELECT COUNT(*) as total 
      FROM events e
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      LEFT JOIN venues v ON e.venue_id = v.id
      ${whereClause}
    `
    const countResult = await duckdb.query(countQuery, params)
    const total = countResult.rows[0]?.total || 0

    // Get events with full details
    const eventsQuery = `
      SELECT 
        e.id, e.title, e.slug, e.description, e.detailed_description,
        e.start_datetime, e.end_datetime, e.timezone,
        e.capacity_min, e.capacity_max, e.current_registrations,
        e.price_platinum, e.price_diamond, e.price_black_card, e.currency,
        e.status, e.approval_status,
        e.required_membership_tiers, e.required_verification,
        e.dress_code, e.language, e.inclusions, e.exclusions,
        e.registration_opens_at, e.registration_closes_at,
        e.cancellation_deadline, e.waitlist_enabled,
        e.featured_image, e.gallery_images,
        e.created_at, e.updated_at, e.published_at,
        
        -- Category details
        ec.name as category_name, ec.slug as category_slug,
        ec.icon as category_icon, ec.color as category_color,
        
        -- Venue details
        v.name as venue_name, v.address as venue_address,
        v.city as venue_city, v.venue_type,
        v.capacity_max as venue_capacity_max,
        
        -- Organizer details (limited for non-admins)
        CASE 
          WHEN ? THEN u.first_name || ' ' || u.last_name
          ELSE 'HeSocial Events Team'
        END as organizer_name
        
      FROM events e
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN users u ON e.organizer_id = u.id
      ${whereClause}
      ORDER BY e.start_datetime ASC, e.created_at DESC
      LIMIT ? OFFSET ?
    `

    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)
    const eventsResult = await duckdb.query(eventsQuery, [isAdmin, ...params, limit, offset])
    const events = eventsResult.rows

    return res.json({
      success: true,
      data: events,
      pagination: {
        page,
        limit,
        total,
        totalPages: Math.ceil(total / limit)
      }
    })
  } catch (error) {
    logger.error('Failed to get events:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve events'
    })
  }
})

/**
 * GET /api/events/:id
 * Get event by ID with full details
 */
router.get('/:id', async (req, res) => {
  try {
    const { id } = req.params
    const user = (req as any).user

    const eventQuery = `
      SELECT 
        e.*,
        ec.name as category_name, ec.slug as category_slug,
        ec.icon as category_icon, ec.color as category_color,
        ec.description as category_description,
        
        v.name as venue_name, v.description as venue_description,
        v.address as venue_address, v.city as venue_city,
        v.venue_type, v.capacity_min as venue_capacity_min,
        v.capacity_max as venue_capacity_max, v.amenities as venue_amenities,
        v.contact_name as venue_contact_name, v.contact_phone as venue_contact_phone,
        
        organizer.first_name || ' ' || organizer.last_name as organizer_name,
        organizer.email as organizer_email,
        
        approver.first_name || ' ' || approver.last_name as approved_by_name
        
      FROM events e
      LEFT JOIN event_categories ec ON e.category_id = ec.id
      LEFT JOIN venues v ON e.venue_id = v.id
      LEFT JOIN users organizer ON e.organizer_id = organizer.id
      LEFT JOIN users approver ON e.approved_by = approver.id
      WHERE e.id = ?
    `

    const eventResult = await duckdb.query(eventQuery, [id])
    const event = eventResult.rows[0]

    if (!event) {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }

    // Check visibility permissions
    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)
    if (!isAdmin && event.status !== 'published') {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }

    // Get registration statistics
    const statsQuery = `
      SELECT 
        COUNT(*) as total_registrations,
        COUNT(CASE WHEN status = 'confirmed' THEN 1 END) as confirmed_registrations,
        COUNT(CASE WHEN status = 'waitlisted' THEN 1 END) as waitlisted_registrations,
        COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_registrations
      FROM event_registrations 
      WHERE event_id = ?
    `
    const statsResult = await duckdb.query(statsQuery, [id])
    const stats = statsResult.rows[0]

    // Get waitlist count
    const waitlistQuery = `SELECT COUNT(*) as waitlist_count FROM event_waitlist WHERE event_id = ? AND status = 'waiting'`
    const waitlistResult = await duckdb.query(waitlistQuery, [id])
    const waitlistCount = waitlistResult.rows[0]?.waitlist_count || 0

    // Remove sensitive admin data for non-admins
    if (!isAdmin) {
      delete event.organizer_email
      delete event.internal_notes
      delete event.cost_breakdown
      delete event.profit_margin
    }

    return res.json({
      success: true,
      data: {
        ...event,
        registration_stats: stats,
        waitlist_count: waitlistCount
      }
    })
  } catch (error) {
    logger.error('Failed to get event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve event'
    })
  }
})

/**
 * POST /api/events
 * Create new event (Admin only)
 */
router.post('/', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const {
      title,
      description,
      detailedDescription,
      categoryId,
      venueId,
      startDatetime,
      endDatetime,
      timezone,
      capacityMin,
      capacityMax,
      pricePlatinum,
      priceDiamond,
      priceBlackCard,
      currency,
      requiredMembershipTiers,
      requiredVerification,
      ageRestriction,
      dressCode,
      language,
      specialRequirements,
      inclusions,
      exclusions,
      registrationOpensAt,
      registrationClosesAt,
      cancellationDeadline,
      waitlistEnabled,
      autoApproval,
      metaTitle,
      metaDescription,
      featuredImage,
      internalNotes,
      costBreakdown,
      profitMargin
    } = req.body

    const user = (req as any).user
    const organizerId = user.id

    // Generate slug from title
    const slug = title.toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-')
      .trim('-') + '-' + Date.now()

    // Generate new event ID
    const eventId = Date.now()

    const insertQuery = `
      INSERT INTO events (
        id, title, slug, description, detailed_description,
        category_id, venue_id, organizer_id,
        start_datetime, end_datetime, timezone,
        capacity_min, capacity_max,
        price_platinum, price_diamond, price_black_card, currency,
        required_membership_tiers, required_verification, age_restriction,
        dress_code, language, special_requirements,
        inclusions, exclusions,
        registration_opens_at, registration_closes_at, cancellation_deadline,
        waitlist_enabled, auto_approval,
        meta_title, meta_description, featured_image,
        internal_notes, cost_breakdown, profit_margin,
        created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `

    await duckdb.query(insertQuery, [
      eventId, title, slug, description, detailedDescription,
      categoryId, venueId, organizerId,
      startDatetime, endDatetime, timezone || 'Asia/Taipei',
      capacityMin || 1, capacityMax,
      pricePlatinum, priceDiamond, priceBlackCard, currency || 'TWD',
      JSON.stringify(requiredMembershipTiers || []), requiredVerification !== false,
      JSON.stringify(ageRestriction || {}),
      dressCode, language || 'Traditional Chinese', specialRequirements,
      JSON.stringify(inclusions || []), JSON.stringify(exclusions || []),
      registrationOpensAt, registrationClosesAt, cancellationDeadline,
      waitlistEnabled !== false, autoApproval === true,
      metaTitle || title, metaDescription || description, featuredImage,
      internalNotes, JSON.stringify(costBreakdown || {}), profitMargin,
      new Date().toISOString(), new Date().toISOString()
    ])

    logger.info(`Event created: ${title} (ID: ${eventId}) by admin: ${user.email}`)

    return res.status(201).json({
      success: true,
      message: 'Event created successfully',
      data: { eventId, slug }
    })
  } catch (error) {
    logger.error('Failed to create event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to create event'
    })
  }
})

/**
 * PUT /api/events/:id
 * Update event (Admin only)
 */
router.put('/:id', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const updates = req.body
    const user = (req as any).user

    // Check if event exists
    const existingEventResult = await duckdb.query('SELECT id, status FROM events WHERE id = ?', [id])
    const existingEvent = existingEventResult.rows[0]
    
    if (!existingEvent) {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }

    // Build update query dynamically
    const updateFields = []
    const params = []

    const allowedFields = [
      'title', 'description', 'detailed_description', 'category_id', 'venue_id',
      'start_datetime', 'end_datetime', 'timezone',
      'capacity_min', 'capacity_max',
      'price_platinum', 'price_diamond', 'price_black_card', 'currency',
      'required_membership_tiers', 'required_verification', 'age_restriction',
      'dress_code', 'language', 'special_requirements',
      'inclusions', 'exclusions',
      'registration_opens_at', 'registration_closes_at', 'cancellation_deadline',
      'waitlist_enabled', 'auto_approval',
      'meta_title', 'meta_description', 'featured_image', 'gallery_images',
      'internal_notes', 'cost_breakdown', 'profit_margin'
    ]

    for (const field of allowedFields) {
      if (updates[field] !== undefined) {
        updateFields.push(`${field} = ?`)
        
        // Handle JSON fields
        if (['required_membership_tiers', 'age_restriction', 'inclusions', 'exclusions', 'gallery_images', 'cost_breakdown'].includes(field)) {
          params.push(JSON.stringify(updates[field]))
        } else {
          params.push(updates[field])
        }
      }
    }

    if (updateFields.length === 0) {
      return res.status(400).json({
        success: false,
        error: 'No valid fields to update'
      })
    }

    updateFields.push('updated_at = ?')
    params.push(new Date().toISOString())
    params.push(id)

    await duckdb.query(`
      UPDATE events 
      SET ${updateFields.join(', ')} 
      WHERE id = ?
    `, params)

    logger.info(`Event updated: ${id} by admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Event updated successfully'
    })
  } catch (error) {
    logger.error('Failed to update event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update event'
    })
  }
})

/**
 * DELETE /api/events/:id
 * Delete event (Super Admin only)
 */
router.delete('/:id', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const user = (req as any).user

    // Check if event exists and get details
    const existingEventResult = await duckdb.query('SELECT id, title, status FROM events WHERE id = ?', [id])
    const existingEvent = existingEventResult.rows[0]
    
    if (!existingEvent) {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }

    // Check if event has registrations
    const registrationsResult = await duckdb.query('SELECT COUNT(*) as count FROM event_registrations WHERE event_id = ?', [id])
    const registrationCount = registrationsResult.rows[0]?.count || 0

    if (registrationCount > 0) {
      return res.status(400).json({
        success: false,
        error: 'Cannot delete event with existing registrations. Archive the event instead.'
      })
    }

    // Delete event (cascade will handle related records)
    await duckdb.query('DELETE FROM events WHERE id = ?', [id])

    logger.warn(`Event deleted: ${existingEvent.title} (ID: ${id}) by super admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Event deleted successfully'
    })
  } catch (error) {
    logger.error('Failed to delete event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to delete event'
    })
  }
})

/**
 * POST /api/events/:id/publish
 * Publish event (Admin only)
 */
router.post('/:id/publish', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const user = (req as any).user

    const result = await duckdb.query(`
      UPDATE events 
      SET status = 'published', published_at = ?, updated_at = ?
      WHERE id = ? AND approval_status = 'approved'
    `, [new Date().toISOString(), new Date().toISOString(), id])

    if (result.changes === 0) {
      return res.status(400).json({
        success: false,
        error: 'Event not found or not approved for publishing'
      })
    }

    logger.info(`Event published: ${id} by admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Event published successfully'
    })
  } catch (error) {
    logger.error('Failed to publish event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to publish event'
    })
  }
})

/**
 * POST /api/events/:id/approve
 * Approve event for publishing (Admin only)
 */
router.post('/:id/approve', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const { approved } = req.body // true for approve, false for reject
    const user = (req as any).user

    const approvalStatus = approved ? 'approved' : 'rejected'
    
    const result = await duckdb.query(`
      UPDATE events 
      SET approval_status = ?, approved_by = ?, approved_at = ?, updated_at = ?
      WHERE id = ?
    `, [approvalStatus, user.id, new Date().toISOString(), new Date().toISOString(), id])

    if (result.changes === 0) {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }

    logger.info(`Event ${approvalStatus}: ${id} by admin: ${user.email}`)

    return res.json({
      success: true,
      message: `Event ${approved ? 'approved' : 'rejected'} successfully`
    })
  } catch (error) {
    logger.error('Failed to approve/reject event:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to process event approval'
    })
  }
})

export default router