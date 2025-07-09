import { Router } from 'express'
import { authenticateToken, requireAdmin } from '../middleware/auth.js'
import { pool } from '../database/duckdb-pool.js'
import { getVisitorAnalytics } from '../middleware/visitorTracking.js'
import logger from '../utils/logger.js'

const router = Router()

// All analytics routes require admin access
router.use(authenticateToken)
router.use(requireAdmin)

/**
 * GET /api/analytics/visitors
 * Get visitor analytics overview
 */
router.get('/visitors', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    const analytics = await getVisitorAnalytics(days)
    
    if (!analytics) {
      return res.status(500).json({
        success: false,
        error: 'Failed to retrieve visitor analytics'
      })
    }

    res.json({
      success: true,
      data: {
        period_days: days,
        ...analytics
      }
    })
  } catch (error) {
    logger.error('Visitor analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve visitor analytics'
    })
  }
})

/**
 * GET /api/analytics/visitors/daily
 * Get daily visitor analytics
 */
router.get('/visitors/daily', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    const result = await pool.query(`
      SELECT * FROM visitor_analytics_daily 
      WHERE date >= DATE('now', '-${days} days')
      ORDER BY date DESC
      LIMIT 100
    `)

    res.json({
      success: true,
      data: result.rows
    })
  } catch (error) {
    logger.error('Daily visitor analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve daily analytics'
    })
  }
})

/**
 * GET /api/analytics/pages/popular
 * Get popular pages analytics
 */
router.get('/pages/popular', async (req, res) => {
  try {
    const limit = parseInt(req.query.limit as string) || 20
    const result = await pool.query(`
      SELECT * FROM popular_pages 
      ORDER BY views DESC 
      LIMIT $1
    `, [limit])

    res.json({
      success: true,
      data: result.rows
    })
  } catch (error) {
    logger.error('Popular pages analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve page analytics'
    })
  }
})

/**
 * GET /api/analytics/conversion
 * Get conversion funnel analytics
 */
router.get('/conversion', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    const result = await pool.query(`
      SELECT 
        COUNT(DISTINCT vs.visitor_id) as total_visitors,
        COUNT(DISTINCT CASE WHEN vpv.path LIKE '/events/%' THEN vs.visitor_id END) as event_viewers,
        COUNT(DISTINCT CASE WHEN vs.user_id IS NOT NULL THEN vs.visitor_id END) as registered_users,
        ROUND(
          COUNT(DISTINCT CASE WHEN vs.user_id IS NOT NULL THEN vs.visitor_id END) * 100.0 / 
          COUNT(DISTINCT vs.visitor_id), 2
        ) as conversion_rate
      FROM visitor_sessions vs
      LEFT JOIN visitor_page_views vpv ON vs.visitor_id = vpv.visitor_id
      WHERE vs.last_seen >= DATE('now', '-${days} days')
    `)

    res.json({
      success: true,
      data: {
        period_days: days,
        ...result.rows[0]
      }
    })
  } catch (error) {
    logger.error('Conversion analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve conversion analytics'
    })
  }
})

/**
 * GET /api/analytics/visitors/:visitorId
 * Get detailed visitor journey
 */
router.get('/visitors/:visitorId', async (req, res) => {
  try {
    const { visitorId } = req.params

    // Get visitor session info
    const sessionResult = await pool.query(`
      SELECT * FROM visitor_sessions WHERE visitor_id = $1
    `, [visitorId])

    if (sessionResult.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Visitor not found'
      })
    }

    // Get visitor page views
    const pageViewsResult = await pool.query(`
      SELECT * FROM visitor_page_views 
      WHERE visitor_id = $1 
      ORDER BY timestamp DESC
      LIMIT 100
    `, [visitorId])

    res.json({
      success: true,
      data: {
        session: sessionResult.rows[0],
        page_views: pageViewsResult.rows
      }
    })
  } catch (error) {
    logger.error('Visitor detail error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve visitor details'
    })
  }
})

/**
 * GET /api/analytics/events/overview
 * Get event analytics overview
 */
router.get('/events/overview', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    
    // Get event statistics
    const eventStats = await pool.query(`
      SELECT 
        COUNT(*) as total_events,
        COUNT(CASE WHEN date_time >= DATE('now', '-${days} days') THEN 1 END) as recent_events,
        COUNT(CASE WHEN date_time >= DATE('now') THEN 1 END) as upcoming_events,
        COUNT(CASE WHEN date_time < DATE('now') THEN 1 END) as past_events,
        AVG(CASE WHEN capacity > 0 THEN (current_attendees * 100.0 / capacity) END) as avg_occupancy_rate
      FROM events
    `)

    // Get registration statistics
    const registrationStats = await pool.query(`
      SELECT 
        COUNT(*) as total_registrations,
        COUNT(CASE WHEN created_at >= DATE('now', '-${days} days') THEN 1 END) as recent_registrations,
        COUNT(DISTINCT user_id) as unique_attendees
      FROM registrations
      WHERE status = 'confirmed'
    `)

    // Get popular events
    const popularEvents = await pool.query(`
      SELECT 
        e.id,
        e.name,
        e.date_time,
        e.capacity,
        e.current_attendees,
        ROUND((e.current_attendees * 100.0 / e.capacity), 2) as occupancy_rate
      FROM events e
      WHERE e.date_time >= DATE('now', '-${days} days')
      ORDER BY e.current_attendees DESC
      LIMIT 10
    `)

    res.json({
      success: true,
      data: {
        period_days: days,
        event_stats: eventStats.rows[0],
        registration_stats: registrationStats.rows[0],
        popular_events: popularEvents.rows
      }
    })
  } catch (error) {
    logger.error('Event analytics overview error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve event analytics overview'
    })
  }
})

/**
 * GET /api/analytics/events/performance
 * Get event performance metrics
 */
router.get('/events/performance', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    
    // Get event performance metrics
    const performance = await pool.query(`
      SELECT 
        e.id,
        e.name,
        e.date_time,
        e.capacity,
        e.current_attendees,
        e.pricing_vip,
        e.pricing_vvip,
        ROUND((e.current_attendees * 100.0 / e.capacity), 2) as occupancy_rate,
        COUNT(r.id) as total_registrations,
        COUNT(CASE WHEN r.status = 'confirmed' THEN 1 END) as confirmed_registrations,
        COUNT(CASE WHEN r.status = 'pending' THEN 1 END) as pending_registrations,
        COUNT(CASE WHEN r.status = 'cancelled' THEN 1 END) as cancelled_registrations,
        AVG(CASE WHEN r.tier = 'vip' THEN e.pricing_vip WHEN r.tier = 'vvip' THEN e.pricing_vvip END) as avg_revenue_per_attendee
      FROM events e
      LEFT JOIN registrations r ON e.id = r.event_id
      WHERE e.date_time >= DATE('now', '-${days} days')
      GROUP BY e.id, e.name, e.date_time, e.capacity, e.current_attendees, e.pricing_vip, e.pricing_vvip
      ORDER BY occupancy_rate DESC
    `)

    res.json({
      success: true,
      data: {
        period_days: days,
        events: performance.rows
      }
    })
  } catch (error) {
    logger.error('Event performance analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve event performance analytics'
    })
  }
})

/**
 * GET /api/analytics/events/engagement
 * Get event engagement metrics
 */
router.get('/events/engagement', async (req, res) => {
  try {
    const days = parseInt(req.query.days as string) || 30
    
    // Get engagement metrics
    const engagement = await pool.query(`
      SELECT 
        DATE(vpv.timestamp) as date,
        COUNT(DISTINCT vpv.visitor_id) as unique_visitors,
        COUNT(vpv.id) as total_page_views,
        COUNT(CASE WHEN vpv.path LIKE '/events/%' THEN 1 END) as event_page_views,
        COUNT(CASE WHEN vpv.path LIKE '/events/%/register' THEN 1 END) as registration_page_views,
        AVG(vpv.time_spent) as avg_time_spent
      FROM visitor_page_views vpv
      WHERE vpv.timestamp >= DATE('now', '-${days} days')
        AND vpv.path LIKE '/events%'
      GROUP BY DATE(vpv.timestamp)
      ORDER BY date DESC
    `)

    res.json({
      success: true,
      data: {
        period_days: days,
        engagement: engagement.rows
      }
    })
  } catch (error) {
    logger.error('Event engagement analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve event engagement analytics'
    })
  }
})

/**
 * POST /api/analytics/events/track
 * Track custom visitor events (for future Google Analytics integration)
 */
router.post('/events/track', async (req, res) => {
  try {
    const { visitor_id, event_type, event_data } = req.body

    if (!visitor_id || !event_type) {
      return res.status(400).json({
        success: false,
        error: 'visitor_id and event_type are required'
      })
    }

    await pool.query(`
      INSERT INTO visitor_events (visitor_id, event_type, event_data, timestamp)
      VALUES ($1, $2, $3, $4)
    `, [
      visitor_id,
      event_type,
      JSON.stringify(event_data || {}),
      new Date().toISOString()
    ])

    res.json({
      success: true,
      message: 'Event tracked successfully'
    })
  } catch (error) {
    logger.error('Event tracking error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to track event'
    })
  }
})

/**
 * GET /api/analytics/revenue/events
 * Get revenue analytics for events
 */
router.get('/revenue/events', async (req, res) => {
  try {
    // Get revenue by month - using actual column names
    const monthlyRevenueQuery = `
      SELECT 
        strftime(e.dateTime, '%Y-%m') as month,
        COUNT(e.id) as event_count,
        SUM(e.currentAttendees) as total_registrations,
        SUM(e.currentAttendees * CAST(json_extract(e.pricing, '$.platinum') AS INTEGER)) as revenue
      FROM events e
      WHERE e.currentAttendees > 0
        AND e.dateTime >= CURRENT_TIMESTAMP - INTERVAL '12 months'
      GROUP BY strftime(e.dateTime, '%Y-%m')
      ORDER BY month DESC
    `
    
    const monthlyRevenue = await pool.query(monthlyRevenueQuery)
    
    // Get revenue by category - using actual column names
    const categoryRevenueQuery = `
      SELECT 
        ec.name as category,
        SUM(e.currentAttendees * CAST(json_extract(e.pricing, '$.platinum') AS INTEGER)) as revenue,
        COUNT(e.id) as event_count,
        AVG(e.currentAttendees * CAST(json_extract(e.pricing, '$.platinum') AS INTEGER)) as avg_revenue_per_event
      FROM events e
      JOIN event_categories ec ON e.category_id = ec.id
      WHERE e.currentAttendees > 0
        AND e.dateTime >= CURRENT_TIMESTAMP - INTERVAL '12 months'
      GROUP BY ec.id, ec.name
      ORDER BY revenue DESC
    `
    
    const categoryRevenue = await pool.query(categoryRevenueQuery)
    
    // Get revenue by membership tier
    const tierRevenueQuery = `
      SELECT 
        u.membership_tier,
        COUNT(r.id) as registration_count,
        SUM(
          CASE 
            WHEN u.membership_tier = 'Platinum' THEN e.price_platinum
            WHEN u.membership_tier = 'Diamond' THEN e.price_diamond
            WHEN u.membership_tier = 'Black Card' THEN e.price_black_card
            ELSE e.price_platinum
          END
        ) as total_revenue
      FROM registrations r
      JOIN users u ON r.user_id = u.id
      JOIN events e ON r.event_id = e.id
      WHERE r.status = 'confirmed'
        AND e.start_datetime >= datetime('now', '-12 months')
      GROUP BY u.membership_tier
      ORDER BY total_revenue DESC
    `
    
    const tierRevenue = await pool.query(tierRevenueQuery)

    res.json({
      success: true,
      data: {
        monthlyRevenue: monthlyRevenue.rows || [],
        categoryRevenue: categoryRevenue.rows || [],
        tierRevenue: tierRevenue.rows || []
      }
    })
  } catch (error) {
    logger.error('Revenue analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch revenue analytics'
    })
  }
})

/**
 * GET /api/analytics/engagement/members
 * Get member engagement analytics
 */
router.get('/engagement/members', async (req, res) => {
  try {
    // Get member engagement metrics
    const engagementQuery = `
      SELECT 
        u.membership_tier,
        COUNT(DISTINCT u.id) as total_members,
        COUNT(DISTINCT r.user_id) as active_members,
        (COUNT(DISTINCT r.user_id)::float / COUNT(DISTINCT u.id) * 100) as engagement_rate,
        AVG(user_stats.event_count) as avg_events_per_member
      FROM users u
      LEFT JOIN registrations r ON u.id = r.user_id 
        AND r.created_at >= datetime('now', '-30 days')
        AND r.status != 'cancelled'
      LEFT JOIN (
        SELECT 
          user_id,
          COUNT(*) as event_count
        FROM registrations
        WHERE created_at >= datetime('now', '-30 days')
          AND status != 'cancelled'
        GROUP BY user_id
      ) user_stats ON u.id = user_stats.user_id
      WHERE u.created_at <= datetime('now', '-7 days')
      GROUP BY u.membership_tier
      ORDER BY engagement_rate DESC
    `
    
    const engagement = await pool.query(engagementQuery)
    
    // Get most active members
    const activeMembers = `
      SELECT 
        u.first_name,
        u.last_name,
        u.membership_tier,
        COUNT(r.id) as events_attended,
        SUM(
          CASE 
            WHEN u.membership_tier = 'Platinum' THEN e.price_platinum
            WHEN u.membership_tier = 'Diamond' THEN e.price_diamond
            WHEN u.membership_tier = 'Black Card' THEN e.price_black_card
            ELSE e.price_platinum
          END
        ) as total_spent
      FROM users u
      JOIN registrations r ON u.id = r.user_id
      JOIN events e ON r.event_id = e.id
      WHERE r.status = 'confirmed'
        AND r.created_at >= datetime('now', '-90 days')
      GROUP BY u.id, u.first_name, u.last_name, u.membership_tier
      ORDER BY events_attended DESC, total_spent DESC
      LIMIT 20
    `
    
    const topMembers = await pool.query(activeMembers)
    
    // Get retention metrics
    const retentionQuery = `
      SELECT 
        strftime('%Y-%m', u.created_at) as cohort_month,
        COUNT(DISTINCT u.id) as cohort_size,
        COUNT(DISTINCT CASE WHEN r.created_at >= datetime('now', '-30 days') THEN u.id END) as active_this_month,
        (COUNT(DISTINCT CASE WHEN r.created_at >= datetime('now', '-30 days') THEN u.id END)::float / COUNT(DISTINCT u.id) * 100) as retention_rate
      FROM users u
      LEFT JOIN registrations r ON u.id = r.user_id AND r.status != 'cancelled'
      WHERE u.created_at >= datetime('now', '-12 months')
      GROUP BY strftime('%Y-%m', u.created_at)
      ORDER BY cohort_month DESC
    `
    
    const retention = await pool.query(retentionQuery)

    res.json({
      success: true,
      data: {
        engagement: engagement.rows || [],
        topMembers: topMembers.rows || [],
        retention: retention.rows || []
      }
    })
  } catch (error) {
    logger.error('Member engagement analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch member engagement data'
    })
  }
})

/**
 * GET /api/analytics/events/:id/performance
 * Get individual event performance metrics
 */
router.get('/events/:id/performance', async (req, res) => {
  try {
    const { id } = req.params
    
    // Get event details with performance metrics
    const eventQuery = `
      SELECT 
        e.*,
        c.name as category_name,
        v.name as venue_name,
        (e.current_registrations::float / e.capacity_max::float * 100) as fill_rate,
        (e.price_platinum * e.current_registrations) as current_revenue,
        (e.price_platinum * e.capacity_max) as potential_revenue
      FROM events e
      JOIN categories c ON e.category_id = c.id
      JOIN venues v ON e.venue_id = v.id
      WHERE e.id = ?
    `
    
    const eventData = await pool.query(eventQuery, [id])
    
    if (!eventData.rows || eventData.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Event not found'
      })
    }
    
    // Get registration timeline
    const registrationTimelineQuery = `
      SELECT 
        DATE(created_at) as date,
        COUNT(*) as registrations,
        SUM(COUNT(*)) OVER (ORDER BY DATE(created_at)) as cumulative_registrations
      FROM registrations
      WHERE event_id = ? AND status != 'cancelled'
      GROUP BY DATE(created_at)
      ORDER BY date
    `
    
    const registrationTimeline = await pool.query(registrationTimelineQuery, [id])
    
    // Get membership tier breakdown
    const membershipBreakdownQuery = `
      SELECT 
        u.membership_tier,
        COUNT(*) as count,
        (COUNT(*)::float / (SELECT COUNT(*) FROM registrations WHERE event_id = ? AND status != 'cancelled') * 100) as percentage
      FROM registrations r
      JOIN users u ON r.user_id = u.id
      WHERE r.event_id = ? AND r.status != 'cancelled'
      GROUP BY u.membership_tier
      ORDER BY count DESC
    `
    
    const membershipBreakdown = await pool.query(membershipBreakdownQuery, [id, id])
    
    // Get registration status breakdown
    const statusBreakdownQuery = `
      SELECT 
        status,
        COUNT(*) as count
      FROM registrations
      WHERE event_id = ?
      GROUP BY status
    `
    
    const statusBreakdown = await pool.query(statusBreakdownQuery, [id])

    res.json({
      success: true,
      data: {
        event: eventData.rows[0],
        registrationTimeline: registrationTimeline.rows || [],
        membershipBreakdown: membershipBreakdown.rows || [],
        statusBreakdown: statusBreakdown.rows || []
      }
    })
  } catch (error) {
    logger.error('Event performance analytics error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to fetch event performance data'
    })
  }
})

export default router
