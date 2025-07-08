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

export default router
