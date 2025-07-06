import { Router, Request, Response } from 'express'
import { authenticateToken, requireAdmin, requireSuperAdmin } from '../middleware/auth.js'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'

const router = Router()

/**
 * GET /api/venues
 * Get all venues with pagination and filtering
 */
router.get('/', async (req, res) => {
  try {
    const page = parseInt(req.query.page as string) || 1
    const limit = parseInt(req.query.limit as string) || 20
    const search = req.query.search as string || ''
    const venueType = req.query.venueType as string || ''
    const city = req.query.city as string || ''
    const isActive = req.query.isActive as string || ''

    const offset = (page - 1) * limit

    let whereConditions = []
    let params: any[] = []

    // Only show active venues for non-admin users
    const user = (req as any).user
    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)
    
    if (!isAdmin) {
      whereConditions.push('is_active = true')
    } else if (isActive) {
      whereConditions.push('is_active = ?')
      params.push(isActive === 'true')
    }

    if (search) {
      whereConditions.push('(name ILIKE ? OR description ILIKE ? OR address ILIKE ?)')
      params.push(`%${search}%`, `%${search}%`, `%${search}%`)
    }

    if (venueType) {
      whereConditions.push('venue_type = ?')
      params.push(venueType)
    }

    if (city) {
      whereConditions.push('city ILIKE ?')
      params.push(`%${city}%`)
    }

    const whereClause = whereConditions.length > 0 ? `WHERE ${whereConditions.join(' AND ')}` : ''

    // Get total count
    const countQuery = `SELECT COUNT(*) as total FROM venues ${whereClause}`
    const countResult = await duckdb.query(countQuery, params)
    const total = countResult.rows[0]?.total || 0

    // Get venues
    const venuesQuery = `
      SELECT 
        id, name, description, address, city, district, postal_code, country,
        venue_type, capacity_min, capacity_max, price_tier, amenities,
        contact_name, contact_phone, contact_email, booking_requirements,
        cancellation_policy, is_active, images, location_coordinates,
        created_at, updated_at
      FROM venues
      ${whereClause}
      ORDER BY is_active DESC, name ASC
      LIMIT ? OFFSET ?
    `
    const venuesResult = await duckdb.query(venuesQuery, [...params, limit, offset])

    return res.json({
      success: true,
      data: venuesResult.rows,
      pagination: {
        page,
        limit,
        total,
        totalPages: Math.ceil(total / limit)
      }
    })
  } catch (error) {
    logger.error('Failed to get venues:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve venues'
    })
  }
})

/**
 * GET /api/venues/:id
 * Get venue by ID
 */
router.get('/:id', async (req, res) => {
  try {
    const { id } = req.params
    
    const venueQuery = `
      SELECT *
      FROM venues
      WHERE id = ?
    `
    const venueResult = await duckdb.query(venueQuery, [id])
    const venue = venueResult.rows[0]

    if (!venue) {
      return res.status(404).json({
        success: false,
        error: 'Venue not found'
      })
    }

    // Check if venue is active for non-admin users
    const user = (req as any).user
    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)
    
    if (!isAdmin && !venue.is_active) {
      return res.status(404).json({
        success: false,
        error: 'Venue not found'
      })
    }

    return res.json({
      success: true,
      data: venue
    })
  } catch (error) {
    logger.error('Failed to get venue:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve venue'
    })
  }
})

/**
 * POST /api/venues
 * Create new venue (Admin only)
 */
router.post('/', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const {
      name,
      description,
      address,
      city,
      district,
      postalCode,
      country,
      venueType,
      capacityMin,
      capacityMax,
      priceTier,
      amenities,
      contactName,
      contactPhone,
      contactEmail,
      bookingRequirements,
      cancellationPolicy,
      images,
      locationCoordinates
    } = req.body

    const user = (req as any).user
    const venueId = Date.now()

    const insertQuery = `
      INSERT INTO venues (
        id, name, description, address, city, district, postal_code, country,
        venue_type, capacity_min, capacity_max, price_tier, amenities,
        contact_name, contact_phone, contact_email, booking_requirements,
        cancellation_policy, is_active, images, location_coordinates,
        created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `

    await duckdb.query(insertQuery, [
      venueId, name, description, address, city, district, postalCode, country || 'Taiwan',
      venueType, capacityMin || 1, capacityMax,
      priceTier || 'premium', JSON.stringify(amenities || []),
      contactName, contactPhone, contactEmail, bookingRequirements, cancellationPolicy,
      true, JSON.stringify(images || []), JSON.stringify(locationCoordinates || {}),
      new Date().toISOString(), new Date().toISOString()
    ])

    logger.info(`Venue created: ${name} (ID: ${venueId}) by admin: ${user.email}`)

    return res.status(201).json({
      success: true,
      message: 'Venue created successfully',
      data: { venueId }
    })
  } catch (error) {
    logger.error('Failed to create venue:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to create venue'
    })
  }
})

/**
 * PUT /api/venues/:id
 * Update venue (Admin only)
 */
router.put('/:id', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const updates = req.body
    const user = (req as any).user

    // Check if venue exists
    const existingVenueResult = await duckdb.query('SELECT id FROM venues WHERE id = ?', [id])
    if (!existingVenueResult.rows[0]) {
      return res.status(404).json({
        success: false,
        error: 'Venue not found'
      })
    }

    // Build update query dynamically
    const updateFields = []
    const params = []

    const allowedFields = [
      'name', 'description', 'address', 'city', 'district', 'postal_code', 'country',
      'venue_type', 'capacity_min', 'capacity_max', 'price_tier', 'amenities',
      'contact_name', 'contact_phone', 'contact_email', 'booking_requirements',
      'cancellation_policy', 'is_active', 'images', 'location_coordinates'
    ]

    for (const field of allowedFields) {
      if (updates[field] !== undefined) {
        updateFields.push(`${field} = ?`)
        
        // Handle JSON fields
        if (['amenities', 'images', 'location_coordinates'].includes(field)) {
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
      UPDATE venues 
      SET ${updateFields.join(', ')} 
      WHERE id = ?
    `, params)

    logger.info(`Venue updated: ${id} by admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Venue updated successfully'
    })
  } catch (error) {
    logger.error('Failed to update venue:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update venue'
    })
  }
})

/**
 * DELETE /api/venues/:id
 * Delete venue (Super Admin only)
 */
router.delete('/:id', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const user = (req as any).user

    // Check if venue exists
    const existingVenueResult = await duckdb.query('SELECT id, name FROM venues WHERE id = ?', [id])
    const existingVenue = existingVenueResult.rows[0]
    
    if (!existingVenue) {
      return res.status(404).json({
        success: false,
        error: 'Venue not found'
      })
    }

    // Check if venue has events
    const eventsResult = await duckdb.query('SELECT COUNT(*) as count FROM events WHERE venue_id = ?', [id])
    const eventCount = eventsResult.rows[0]?.count || 0

    if (eventCount > 0) {
      return res.status(400).json({
        success: false,
        error: 'Cannot delete venue with associated events. Deactivate the venue instead.'
      })
    }

    // Delete venue
    await duckdb.query('DELETE FROM venues WHERE id = ?', [id])

    logger.warn(`Venue deleted: ${existingVenue.name} (ID: ${id}) by super admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Venue deleted successfully'
    })
  } catch (error) {
    logger.error('Failed to delete venue:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to delete venue'
    })
  }
})

export default router