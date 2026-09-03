// Emergency Database Schema Routes
// Quick fixes for critical database schema issues

import { Router } from 'express';
import { authenticateToken, requireAdmin } from '../middleware/auth.js';
import { pool } from '../database/duckdb-pool.js';
import logger from '../utils/logger.js';

const router = Router();

// All emergency routes require admin access
router.use(authenticateToken);
router.use(requireAdmin);

/**
 * POST /api/emergency/apply-visitor-tracking
 * Emergency: Apply visitor tracking schema directly
 */
router.post('/apply-visitor-tracking', async (req, res) => {
  try {
    const { confirm } = req.body;
    
    if (!confirm) {
      return res.status(400).json({
        success: false,
        error: 'Emergency operation requires explicit confirmation',
        hint: 'Send { "confirm": true } in request body'
      });
    }

    logger.warn('⚠️ Emergency visitor tracking schema application starting...');
    
    // Create visitor tracking tables
    const visitorTrackingSQL = `
      -- Visitor sessions table - tracks unique visitors and their sessions
      CREATE TABLE IF NOT EXISTS visitor_sessions (
        id INTEGER PRIMARY KEY,
        visitor_id VARCHAR(50) UNIQUE NOT NULL,
        user_id INTEGER NULL, -- Links to users table when visitor converts
        ip_address VARCHAR(45) NOT NULL,
        user_agent TEXT NOT NULL,
        referer TEXT NULL,
        first_seen TIMESTAMP NOT NULL,
        last_seen TIMESTAMP NOT NULL,
        page_views INTEGER DEFAULT 1,
        session_count INTEGER DEFAULT 1,
        converted_at TIMESTAMP NULL, -- When visitor became registered user
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );

      -- Individual page views table - detailed tracking for analytics
      CREATE TABLE IF NOT EXISTS visitor_page_views (
        id INTEGER PRIMARY KEY,
        visitor_id VARCHAR(50) NOT NULL,
        path VARCHAR(500) NOT NULL,
        method VARCHAR(10) NOT NULL DEFAULT 'GET',
        query_params TEXT NULL,
        referer TEXT NULL,
        timestamp TIMESTAMP NOT NULL,
        ip_address VARCHAR(45) NOT NULL,
        user_agent TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );

      -- Visitor events table - track specific actions (future use)
      CREATE TABLE IF NOT EXISTS visitor_events (
        id INTEGER PRIMARY KEY,
        visitor_id VARCHAR(50) NOT NULL,
        event_type VARCHAR(50) NOT NULL, -- 'page_view', 'event_view', 'registration_attempt', etc.
        event_data TEXT NULL, -- JSON data for event details
        timestamp TIMESTAMP NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      );

      -- Indexes for performance
      CREATE INDEX IF NOT EXISTS idx_visitor_sessions_visitor_id ON visitor_sessions(visitor_id);
      CREATE INDEX IF NOT EXISTS idx_visitor_sessions_user_id ON visitor_sessions(user_id);
      CREATE INDEX IF NOT EXISTS idx_visitor_sessions_last_seen ON visitor_sessions(last_seen);
      CREATE INDEX IF NOT EXISTS idx_visitor_page_views_visitor_id ON visitor_page_views(visitor_id);
      CREATE INDEX IF NOT EXISTS idx_visitor_page_views_timestamp ON visitor_page_views(timestamp);
      CREATE INDEX IF NOT EXISTS idx_visitor_page_views_path ON visitor_page_views(path);
      CREATE INDEX IF NOT EXISTS idx_visitor_events_visitor_id ON visitor_events(visitor_id);
      CREATE INDEX IF NOT EXISTS idx_visitor_events_type ON visitor_events(event_type);
      CREATE INDEX IF NOT EXISTS idx_visitor_events_timestamp ON visitor_events(timestamp);
    `;

    // Execute the SQL
    await pool.query(visitorTrackingSQL);
    
    logger.info('✅ Emergency visitor tracking schema applied successfully');
    
    res.json({
      success: true,
      data: {
        message: 'Emergency visitor tracking schema applied successfully',
        warning: 'This operation bypassed the blue-green deployment system',
        tablesCreated: [
          'visitor_sessions',
          'visitor_page_views', 
          'visitor_events'
        ],
        indexesCreated: 9
      }
    });
  } catch (error) {
    logger.error('Emergency visitor tracking schema application failed', { error });
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Emergency operation failed'
    });
  }
});

/**
 * GET /api/emergency/test-visitor-tracking
 * Test if visitor tracking tables exist and are working
 */
router.get('/test-visitor-tracking', async (req, res) => {
  try {
    const tests = [
      'SELECT COUNT(*) as count FROM visitor_sessions',
      'SELECT COUNT(*) as count FROM visitor_page_views',
      'SELECT COUNT(*) as count FROM visitor_events'
    ];

    const results: any = {};
    
    for (const sql of tests) {
      try {
        const result = await pool.query(sql);
        const tableName = sql.match(/FROM (\w+)/)?.[1] || 'unknown';
        results[tableName] = {
          exists: true,
          count: result.rows[0]?.count || 0
        };
      } catch (error) {
        const tableName = sql.match(/FROM (\w+)/)?.[1] || 'unknown';
        results[tableName] = {
          exists: false,
          error: error instanceof Error ? error.message : 'Unknown error'
        };
      }
    }

    res.json({
      success: true,
      data: {
        message: 'Visitor tracking table test completed',
        results
      }
    });
  } catch (error) {
    logger.error('Visitor tracking test failed', { error });
    res.status(500).json({
      success: false,
      error: 'Test failed'
    });
  }
});

/**
 * POST /api/emergency/fix-analytics-queries
 * Fix analytics queries for DuckDB compatibility
 */
router.post('/fix-analytics-queries', async (req, res) => {
  try {
    const { confirm } = req.body;
    
    if (!confirm) {
      return res.status(400).json({
        success: false,
        error: 'Emergency operation requires explicit confirmation',
        hint: 'Send { "confirm": true } in request body'
      });
    }

    logger.warn('⚠️ Emergency analytics query fix starting...');
    
    // Create helper views for DuckDB-compatible analytics
    const analyticsFixSQL = `
      -- Create a view to help with analytics queries using proper DuckDB syntax
      CREATE OR REPLACE VIEW analytics_events AS
      SELECT 
        e.*,
        ec.name as category_name,
        v.name as venue_name,
        CAST(json_extract(e.pricing, '$.platinum') AS INTEGER) as platinum_price,
        CAST(json_extract(e.pricing, '$.diamond') AS INTEGER) as diamond_price,
        CAST(json_extract(e.pricing, '$.black_card') AS INTEGER) as black_card_price
      FROM events e
      JOIN event_categories ec ON e.category_id = ec.id
      JOIN venues v ON e.venue_id = v.id;

      -- Create analytics helper view for member engagement
      CREATE OR REPLACE VIEW member_engagement AS
      SELECT 
        u.id as user_id,
        u.membership_tier,
        u.created_at as member_since,
        COUNT(r.id) as total_registrations,
        COUNT(CASE WHEN r.created_at >= CURRENT_TIMESTAMP - INTERVAL '30 days' THEN 1 END) as recent_registrations,
        MAX(r.created_at) as last_registration
      FROM users u
      LEFT JOIN registrations r ON u.id = r.user_id AND r.status = 'confirmed'
      GROUP BY u.id, u.membership_tier, u.created_at;
    `;

    // Execute the SQL
    await pool.query(analyticsFixSQL);
    
    logger.info('✅ Emergency analytics query fix applied successfully');
    
    res.json({
      success: true,
      data: {
        message: 'Emergency analytics query fix applied successfully',
        viewsCreated: [
          'analytics_events',
          'member_engagement'
        ]
      }
    });
  } catch (error) {
    logger.error('Emergency analytics fix failed', { error });
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Emergency operation failed'
    });
  }
});

/**
 * GET /api/emergency/test-analytics
 * Test basic analytics queries
 */
router.get('/test-analytics', async (req, res) => {
  try {
    const tests = [
      'SELECT COUNT(*) as count FROM events',
      'SELECT COUNT(*) as count FROM event_categories', 
      'SELECT COUNT(*) as count FROM users',
      'SELECT SUM(current_attendees) as total_attendees FROM events'
    ];

    const results: any = {};
    
    for (const sql of tests) {
      try {
        const result = await pool.query(sql);
        const data = result.rows[0] || {};
        
        // Convert BigInt to number for JSON serialization
        const serializedData: any = {};
        for (const [key, value] of Object.entries(data)) {
          serializedData[key] = typeof value === 'bigint' ? Number(value) : value;
        }
        
        results[sql] = {
          success: true,
          data: serializedData,
          rowCount: result.rows?.length || 0
        };
      } catch (error) {
        results[sql] = {
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error'
        };
      }
    }

    res.json({
      success: true,
      data: {
        message: 'Analytics test completed',
        results
      }
    });
  } catch (error) {
    logger.error('Analytics test failed', { error });
    res.json({
      success: false,
      error: 'Test failed',
      details: error instanceof Error ? error.message : 'Unknown error'
    });
  }
});

export default router;