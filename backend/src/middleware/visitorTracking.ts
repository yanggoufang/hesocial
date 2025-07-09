import { Request, Response, NextFunction } from 'express'
import { v4 as uuidv4 } from 'uuid'
import { pool } from '../database/duckdb-pool.js'
import logger from '../utils/logger.js'

export interface VisitorRequest extends Request {
  visitorId?: string
  isNewVisitor?: boolean
}

export const visitorTracking = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const visitorReq = req as VisitorRequest
    
    // Check for existing visitor ID in headers or cookies
    let visitorId = req.headers['x-visitor-id'] as string || 
                   req.cookies?.visitorId ||
                   req.query.visitorId as string

    let isNewVisitor = false

    // Generate new visitor ID if none exists
    if (!visitorId) {
      visitorId = `visitor_${uuidv4()}`
      isNewVisitor = true
      
      // Set visitor ID cookie (expires in 1 year)
      res.cookie('visitorId', visitorId, {
        maxAge: 365 * 24 * 60 * 60 * 1000, // 1 year
        httpOnly: false, // Allow frontend access for analytics
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax'
      })
    }

    // Attach visitor info to request
    visitorReq.visitorId = visitorId
    visitorReq.isNewVisitor = isNewVisitor

    // Track visitor session (async, don't block request)
    trackVisitorSession(visitorId, req, isNewVisitor).catch(error => {
      logger.warn('Failed to track visitor session:', error)
    })

    next()
  } catch (error) {
    logger.error('Visitor tracking middleware error:', error)
    // Don't block request on tracking errors
    next()
  }
}

async function trackVisitorSession(
  visitorId: string, 
  req: Request, 
  isNewVisitor: boolean
): Promise<void> {
  try {
    const sessionData = {
      visitor_id: visitorId,
      ip_address: req.ip || req.connection.remoteAddress || 'unknown',
      user_agent: req.headers['user-agent'] || 'unknown',
      referer: req.headers.referer || null,
      path: req.path,
      method: req.method,
      query_params: JSON.stringify(req.query),
      timestamp: new Date().toISOString(),
      is_new_visitor: isNewVisitor
    }

    // Insert or update visitor session
    if (isNewVisitor) {
      // Create new visitor record
      await pool.query(`
        INSERT INTO visitor_sessions (
          visitor_id, ip_address, user_agent, referer, 
          first_seen, last_seen, page_views, session_count
        ) VALUES ($1, $2, $3, $4, $5, $5, 1, 1)
        ON CONFLICT (visitor_id) DO UPDATE SET
          last_seen = $5,
          page_views = visitor_sessions.page_views + 1,
          session_count = visitor_sessions.session_count + 1
      `, [
        visitorId,
        sessionData.ip_address,
        sessionData.user_agent,
        sessionData.referer,
        sessionData.timestamp
      ])
    } else {
      // Update existing visitor
      await pool.query(`
        UPDATE visitor_sessions 
        SET 
          last_seen = $2,
          page_views = page_views + 1,
          ip_address = $3,
          user_agent = $4
        WHERE visitor_id = $1
      `, [
        visitorId,
        sessionData.timestamp,
        sessionData.ip_address,
        sessionData.user_agent
      ])
    }

    // Track individual page view
    await pool.query(`
      INSERT INTO visitor_page_views (
        visitor_id, path, method, query_params, 
        referer, timestamp, ip_address, user_agent
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    `, [
      visitorId,
      sessionData.path,
      sessionData.method,
      sessionData.query_params,
      sessionData.referer,
      sessionData.timestamp,
      sessionData.ip_address,
      sessionData.user_agent
    ])

  } catch (error) {
    logger.error('Failed to track visitor session:', error)
  }
}

// Helper function to link visitor to user after registration
export const linkVisitorToUser = async (visitorId: string, userId: number): Promise<void> => {
  try {
    await pool.query(`
      UPDATE visitor_sessions 
      SET 
        user_id = $2,
        converted_at = $3
      WHERE visitor_id = $1
    `, [visitorId, userId, new Date().toISOString()])

    logger.info(`Linked visitor ${visitorId} to user ${userId}`)
  } catch (error) {
    logger.error('Failed to link visitor to user:', error)
  }
}

// Analytics helper functions
export const getVisitorAnalytics = async (days: number = 30) => {
  try {
    const result = await pool.query(`
      SELECT 
        COUNT(DISTINCT visitor_id) as unique_visitors,
        COUNT(*) as total_page_views,
        COUNT(DISTINCT CASE WHEN user_id IS NOT NULL THEN visitor_id END) as converted_visitors,
        AVG(page_views) as avg_pages_per_visitor,
        COUNT(CASE WHEN first_seen >= $1 THEN 1 END) as new_visitors
      FROM visitor_sessions 
      WHERE last_seen >= $1
    `, [new Date(Date.now() - days * 24 * 60 * 60 * 1000).toISOString()])

    return result.rows[0]
  } catch (error) {
    logger.error('Failed to get visitor analytics:', error)
    return null
  }
}
