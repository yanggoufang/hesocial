import { Router } from 'express'
import { Request, Response } from 'express'
import { DuckDBConnection } from '../database/connection.js'
import { authenticateToken, requireAdmin } from '../middleware/auth.js'
import { logger } from '../utils/logger.js'

const router = Router()

// Apply authentication middleware to all analytics routes
router.use(authenticateToken)
router.use(requireAdmin)

interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  message?: string
}

// GET /api/analytics/events/overview - All events summary
router.get('/events/overview', async (req: Request, res: Response) => {
  try {
    const db = DuckDBConnection.getInstance()
    
    // Get overall event statistics
    const overviewQuery = `
      SELECT 
        COUNT(*) as total_events,
        COUNT(CASE WHEN status = 'published' THEN 1 END) as published_events,
        COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_events,
        COUNT(CASE WHEN status = 'cancelled' THEN 1 END) as cancelled_events,
        AVG(current_registrations) as avg_registrations,
        SUM(current_registrations) as total_registrations,
        SUM(price_platinum * current_registrations) as estimated_revenue
      FROM events
      WHERE created_at >= datetime('now', '-30 days')
    `
    
    const overview = await db.query(overviewQuery)
    
    // Get monthly trends
    const trendsQuery = `
      SELECT 
        strftime('%Y-%m', created_at) as month,
        COUNT(*) as events_created,
        SUM(current_registrations) as total_registrations,
        AVG(current_registrations::float / capacity_max::float * 100) as avg_fill_rate
      FROM events
      WHERE created_at >= datetime('now', '-12 months')
      GROUP BY strftime('%Y-%m', created_at)
      ORDER BY month DESC
      LIMIT 12
    `
    
    const trends = await db.query(trendsQuery)
    
    // Get top performing events
    const topEventsQuery = `
      SELECT 
        id,
        title,
        current_registrations,
        capacity_max,
        (current_registrations::float / capacity_max::float * 100) as fill_rate,
        price_platinum * current_registrations as revenue,
        status
      FROM events
      WHERE status IN ('published', 'completed')
      ORDER BY fill_rate DESC, revenue DESC
      LIMIT 10
    `
    
    const topEvents = await db.query(topEventsQuery)
    
    // Get category performance
    const categoryQuery = `
      SELECT 
        c.name as category_name,
        COUNT(e.id) as event_count,
        SUM(e.current_registrations) as total_registrations,
        AVG(e.current_registrations::float / e.capacity_max::float * 100) as avg_fill_rate,
        SUM(e.price_platinum * e.current_registrations) as total_revenue
      FROM events e
      JOIN categories c ON e.category_id = c.id
      WHERE e.created_at >= datetime('now', '-30 days')
      GROUP BY c.id, c.name
      ORDER BY total_revenue DESC
    `
    
    const categoryPerformance = await db.query(categoryQuery)

    const response: ApiResponse = {
      success: true,
      data: {
        overview: overview[0] || {},
        trends: trends || [],
        topEvents: topEvents || [],
        categoryPerformance: categoryPerformance || []
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Analytics overview error:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to fetch analytics overview'
    }
    res.status(500).json(response)
  }
})

// GET /api/analytics/events/:id/performance - Individual event metrics
router.get('/events/:id/performance', async (req: Request, res: Response) => {
  try {
    const { id } = req.params
    const db = DuckDBConnection.getInstance()
    
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
    
    const eventData = await db.query(eventQuery, [id])
    
    if (!eventData || eventData.length === 0) {
      const response: ApiResponse = {
        success: false,
        error: 'Event not found'
      }
      return res.status(404).json(response)
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
    
    const registrationTimeline = await db.query(registrationTimelineQuery, [id])
    
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
    
    const membershipBreakdown = await db.query(membershipBreakdownQuery, [id, id])
    
    // Get registration status breakdown
    const statusBreakdownQuery = `
      SELECT 
        status,
        COUNT(*) as count
      FROM registrations
      WHERE event_id = ?
      GROUP BY status
    `
    
    const statusBreakdown = await db.query(statusBreakdownQuery, [id])

    const response: ApiResponse = {
      success: true,
      data: {
        event: eventData[0],
        registrationTimeline: registrationTimeline || [],
        membershipBreakdown: membershipBreakdown || [],
        statusBreakdown: statusBreakdown || []
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Event performance analytics error:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to fetch event performance data'
    }
    res.status(500).json(response)
  }
})

// GET /api/analytics/revenue/events - Revenue analytics
router.get('/revenue/events', async (req: Request, res: Response) => {
  try {
    const db = DuckDBConnection.getInstance()
    
    // Get revenue by month
    const monthlyRevenueQuery = `
      SELECT 
        strftime('%Y-%m', e.start_datetime) as month,
        SUM(e.price_platinum * e.current_registrations) as revenue,
        COUNT(e.id) as event_count,
        SUM(e.current_registrations) as total_registrations
      FROM events e
      WHERE e.status IN ('published', 'completed')
        AND e.start_datetime >= datetime('now', '-12 months')
      GROUP BY strftime('%Y-%m', e.start_datetime)
      ORDER BY month DESC
    `
    
    const monthlyRevenue = await db.query(monthlyRevenueQuery)
    
    // Get revenue by category
    const categoryRevenueQuery = `
      SELECT 
        c.name as category,
        SUM(e.price_platinum * e.current_registrations) as revenue,
        COUNT(e.id) as event_count,
        AVG(e.price_platinum * e.current_registrations) as avg_revenue_per_event
      FROM events e
      JOIN categories c ON e.category_id = c.id
      WHERE e.status IN ('published', 'completed')
        AND e.start_datetime >= datetime('now', '-12 months')
      GROUP BY c.id, c.name
      ORDER BY revenue DESC
    `
    
    const categoryRevenue = await db.query(categoryRevenueQuery)
    
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
    
    const tierRevenue = await db.query(tierRevenueQuery)

    const response: ApiResponse = {
      success: true,
      data: {
        monthlyRevenue: monthlyRevenue || [],
        categoryRevenue: categoryRevenue || [],
        tierRevenue: tierRevenue || []
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Revenue analytics error:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to fetch revenue analytics'
    }
    res.status(500).json(response)
  }
})

// GET /api/analytics/engagement/members - Member engagement stats
router.get('/engagement/members', async (req: Request, res: Response) => {
  try {
    const db = DuckDBConnection.getInstance()
    
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
      WHERE u.created_at <= datetime('now', '-7 days') -- Only count members who joined more than a week ago
      GROUP BY u.membership_tier
      ORDER BY engagement_rate DESC
    `
    
    const engagement = await db.query(engagementQuery)
    
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
    
    const topMembers = await db.query(activeMembers)
    
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
    
    const retention = await db.query(retentionQuery)

    const response: ApiResponse = {
      success: true,
      data: {
        engagement: engagement || [],
        topMembers: topMembers || [],
        retention: retention || []
      }
    }

    res.json(response)
  } catch (error) {
    logger.error('Member engagement analytics error:', error)
    const response: ApiResponse = {
      success: false,
      error: 'Failed to fetch member engagement data'
    }
    res.status(500).json(response)
  }
})

export default router
