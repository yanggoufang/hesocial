import { Request, Response } from 'express'
import { v4 as uuidv4 } from 'uuid'
import { pool } from '@/database/connection.js'
import { AuthenticatedRequest } from '@/types/index.js'
import logger from '@/utils/logger.js'

export const getEvents = async (req: Request, res: Response): Promise<void> => {
  try {
    const {
      page = 1,
      limit = 20,
      sort = 'date_time',
      order = 'asc',
      category,
      exclusivityLevel,
      search,
      upcoming = 'true'
    } = req.query

    const offset = (Number(page) - 1) * Number(limit)
    let whereConditions = ['e.is_active = true']
    const queryParams: any[] = []
    let paramCount = 0

    // Add upcoming filter
    if (upcoming === 'true') {
      whereConditions.push('e.date_time > CURRENT_TIMESTAMP')
    }

    // Add search filter
    if (search) {
      paramCount++
      whereConditions.push(`(e.name ILIKE $${paramCount} OR e.description ILIKE $${paramCount})`)
      queryParams.push(`%${search}%`)
    }

    // Add category filter
    if (category) {
      paramCount++
      whereConditions.push(`ec.name = $${paramCount}`)
      queryParams.push(category)
    }

    // Add exclusivity level filter
    if (exclusivityLevel) {
      paramCount++
      whereConditions.push(`e.exclusivity_level = $${paramCount}`)
      queryParams.push(exclusivityLevel)
    }

    const whereClause = whereConditions.length > 0 ? `WHERE ${whereConditions.join(' AND ')}` : ''
    
    // Validate sort column
    const validSortColumns = ['date_time', 'created_at', 'name', 'capacity', 'current_attendees']
    const sortColumn = validSortColumns.includes(String(sort)) ? sort : 'date_time'
    const sortOrder = order === 'desc' ? 'DESC' : 'ASC'

    const eventsQuery = `
      SELECT 
        e.id, e.name, e.description, e.date_time as "dateTime",
        e.registration_deadline as "registrationDeadline",
        e.pricing, e.exclusivity_level as "exclusivityLevel",
        e.dress_code as "dressCode", e.capacity, e.current_attendees as "currentAttendees",
        e.amenities, e.privacy_guarantees as "privacyGuarantees",
        e.images, e.video_url as "videoUrl", e.requirements,
        e.created_at as "createdAt", e.updated_at as "updatedAt",
        v.name as "venueName", v.address as "venueAddress", v.city as "venueCity",
        v.rating as "venueRating", v.amenities as "venueAmenities",
        ec.name as "categoryName", ec.icon as "categoryIcon",
        u.first_name || ' ' || u.last_name as "organizerName"
      FROM events e
      JOIN venues v ON e.venue_id = v.id
      JOIN event_categories ec ON e.category_id = ec.id
      JOIN users u ON e.organizer_id = u.id
      ${whereClause}
      ORDER BY e.${sortColumn} ${sortOrder}
      LIMIT $${paramCount + 1} OFFSET $${paramCount + 2}
    `

    queryParams.push(Number(limit), offset)

    // Get total count
    const countQuery = `
      SELECT COUNT(*) as total
      FROM events e
      JOIN venues v ON e.venue_id = v.id
      JOIN event_categories ec ON e.category_id = ec.id
      JOIN users u ON e.organizer_id = u.id
      ${whereClause}
    `

    const [eventsResult, countResult] = await Promise.all([
      pool.query(eventsQuery, queryParams),
      pool.query(countQuery, queryParams.slice(0, -2)) // Remove limit and offset for count
    ])

    const events = eventsResult.rows.map(event => ({
      ...event,
      venue: {
        name: event.venueName,
        address: event.venueAddress,
        city: event.venueCity,
        rating: event.venueRating,
        amenities: event.venueAmenities
      },
      category: {
        name: event.categoryName,
        icon: event.categoryIcon
      },
      organizer: event.organizerName,
      // Remove individual venue/category/organizer fields
      venueName: undefined,
      venueAddress: undefined,
      venueCity: undefined,
      venueRating: undefined,
      venueAmenities: undefined,
      categoryName: undefined,
      categoryIcon: undefined,
      organizerName: undefined
    }))

    const total = parseInt(countResult.rows[0].total)
    const totalPages = Math.ceil(total / Number(limit))

    res.json({
      success: true,
      data: events,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages
      }
    })
  } catch (error) {
    logger.error('Get events error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get events'
    })
  }
}

export const getEventById = async (req: Request, res: Response): Promise<void> => {
  try {
    const { id } = req.params

    const eventQuery = `
      SELECT 
        e.id, e.name, e.description, e.date_time as "dateTime",
        e.registration_deadline as "registrationDeadline",
        e.pricing, e.exclusivity_level as "exclusivityLevel",
        e.dress_code as "dressCode", e.capacity, e.current_attendees as "currentAttendees",
        e.amenities, e.privacy_guarantees as "privacyGuarantees",
        e.images, e.video_url as "videoUrl", e.requirements,
        e.created_at as "createdAt", e.updated_at as "updatedAt",
        v.id as "venueId", v.name as "venueName", v.address as "venueAddress", 
        v.city as "venueCity", v.latitude, v.longitude,
        v.rating as "venueRating", v.amenities as "venueAmenities", v.images as "venueImages",
        ec.id as "categoryId", ec.name as "categoryName", ec.description as "categoryDescription",
        ec.icon as "categoryIcon",
        u.first_name || ' ' || u.last_name as "organizerName"
      FROM events e
      JOIN venues v ON e.venue_id = v.id
      JOIN event_categories ec ON e.category_id = ec.id
      JOIN users u ON e.organizer_id = u.id
      WHERE e.id = $1 AND e.is_active = true
    `

    const result = await pool.query(eventQuery, [id])

    if (result.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'Event not found'
      })
      return
    }

    const event = result.rows[0]
    const formattedEvent = {
      ...event,
      venue: {
        id: event.venueId,
        name: event.venueName,
        address: event.venueAddress,
        city: event.venueCity,
        coordinates: {
          lat: event.latitude,
          lng: event.longitude
        },
        rating: event.venueRating,
        amenities: event.venueAmenities,
        images: event.venueImages
      },
      category: {
        id: event.categoryId,
        name: event.categoryName,
        description: event.categoryDescription,
        icon: event.categoryIcon
      },
      organizer: event.organizerName,
      // Remove individual fields
      venueId: undefined,
      venueName: undefined,
      venueAddress: undefined,
      venueCity: undefined,
      latitude: undefined,
      longitude: undefined,
      venueRating: undefined,
      venueAmenities: undefined,
      venueImages: undefined,
      categoryId: undefined,
      categoryName: undefined,
      categoryDescription: undefined,
      categoryIcon: undefined,
      organizerName: undefined
    }

    res.json({
      success: true,
      data: formattedEvent
    })
  } catch (error) {
    logger.error('Get event by ID error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get event'
    })
  }
}

export const createEvent = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const {
      name,
      description,
      dateTime,
      registrationDeadline,
      venueId,
      categoryId,
      pricing,
      exclusivityLevel,
      dressCode,
      capacity,
      amenities = [],
      privacyGuarantees = [],
      requirements = []
    } = req.body

    const eventId = uuidv4()
    const eventQuery = `
      INSERT INTO events (
        id, name, description, date_time, registration_deadline,
        venue_id, category_id, organizer_id, pricing,
        exclusivity_level, dress_code, capacity,
        amenities, privacy_guarantees, requirements
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
      RETURNING 
        id, name, description, date_time as "dateTime",
        registration_deadline as "registrationDeadline",
        pricing, exclusivity_level as "exclusivityLevel",
        dress_code as "dressCode", capacity, current_attendees as "currentAttendees",
        amenities, privacy_guarantees as "privacyGuarantees",
        requirements, created_at as "createdAt"
    `

    const result = await pool.query(eventQuery, [
      eventId, name, description, dateTime, registrationDeadline,
      venueId, categoryId, req.user.id, JSON.stringify(pricing),
      exclusivityLevel, dressCode, capacity,
      amenities, privacyGuarantees, JSON.stringify(requirements)
    ])

    // Log event creation
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'event_created',
        'event',
        eventId,
        JSON.stringify({ eventName: name, exclusivityLevel, capacity }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Event created:', { eventId, userId: req.user.id, eventName: name })

    res.status(201).json({
      success: true,
      message: 'Event created successfully',
      data: result.rows[0]
    })
  } catch (error) {
    logger.error('Create event error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to create event'
    })
  }
}

export const getEventCategories = async (req: Request, res: Response): Promise<void> => {
  try {
    const categoriesQuery = `
      SELECT id, name, description, icon, created_at as "createdAt"
      FROM event_categories
      ORDER BY name
    `

    const result = await pool.query(categoriesQuery)

    res.json({
      success: true,
      data: result.rows
    })
  } catch (error) {
    logger.error('Get event categories error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get event categories'
    })
  }
}

export const getVenues = async (req: Request, res: Response): Promise<void> => {
  try {
    const { city, rating } = req.query
    let whereConditions: string[] = []
    const queryParams: any[] = []
    let paramCount = 0

    if (city) {
      paramCount++
      whereConditions.push(`city ILIKE $${paramCount}`)
      queryParams.push(`%${city}%`)
    }

    if (rating) {
      paramCount++
      whereConditions.push(`rating >= $${paramCount}`)
      queryParams.push(Number(rating))
    }

    const whereClause = whereConditions.length > 0 ? `WHERE ${whereConditions.join(' AND ')}` : ''

    const venuesQuery = `
      SELECT 
        id, name, address, city, latitude, longitude,
        rating, amenities, images, created_at as "createdAt"
      FROM venues
      ${whereClause}
      ORDER BY rating DESC, name
    `

    const result = await pool.query(venuesQuery, queryParams)

    const venues = result.rows.map(venue => ({
      ...venue,
      coordinates: {
        lat: venue.latitude,
        lng: venue.longitude
      },
      latitude: undefined,
      longitude: undefined
    }))

    res.json({
      success: true,
      data: venues
    })
  } catch (error) {
    logger.error('Get venues error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get venues'
    })
  }
}