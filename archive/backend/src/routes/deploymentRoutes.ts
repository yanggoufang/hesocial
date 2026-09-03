// Blue-Green Deployment Management API Routes
// Provides endpoints for zero-downtime schema deployments

import { Router } from 'express';
import { authenticateToken, requireAdmin } from '../middleware/auth.js';
import { migrationService } from '../database/MigrationService.js';
import { blueGreenManager } from '../database/BlueGreenDatabaseManager.js';
import logger from '../utils/logger.js';

const router = Router();

// All deployment routes require admin access
router.use(authenticateToken);
router.use(requireAdmin);

/**
 * GET /api/deployment/status
 * Get current deployment system status
 */
router.get('/status', async (req, res) => {
  try {
    const status = await migrationService.getSystemStatus();
    
    res.json({
      success: true,
      data: {
        ...status,
        timestamp: new Date().toISOString()
      }
    });
  } catch (error) {
    logger.error('Failed to get deployment status', { error });
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve deployment status'
    });
  }
});

/**
 * GET /api/deployment/health
 * Get health status of all database environments
 */
router.get('/health', async (req, res) => {
  try {
    const health = await migrationService.getHealthStatus();
    
    res.json({
      success: true,
      data: health
    });
  } catch (error) {
    logger.error('Failed to get health status', { error });
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve health status'
    });
  }
});

/**
 * POST /api/deployment/deploy-visitor-tracking
 * Deploy visitor tracking tables with zero downtime
 */
router.post('/deploy-visitor-tracking', async (req, res) => {
  try {
    logger.info('🚀 Deployment request received: visitor tracking tables');
    
    // Check if deployment is already in progress
    const status = await migrationService.getSystemStatus();
    if (!status.canDeploy) {
      return res.status(409).json({
        success: false,
        error: 'Deployment already in progress'
      });
    }

    // Execute deployment
    const result = await migrationService.deployVisitorTracking();
    
    if (result.success) {
      res.json({
        success: true,
        data: {
          message: 'Visitor tracking tables deployed successfully',
          deployment: result
        }
      });
    } else {
      res.status(500).json({
        success: false,
        error: result.error || 'Deployment failed',
        deployment: result
      });
    }
  } catch (error) {
    logger.error('Deployment failed', { error });
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Deployment failed'
    });
  }
});

/**
 * POST /api/deployment/rollback
 * Perform instant rollback to previous environment
 */
router.post('/rollback', async (req, res) => {
  try {
    logger.warn('🔄 Rollback request received');
    
    const result = await migrationService.rollback();
    
    if (result.success) {
      res.json({
        success: true,
        data: {
          message: 'Rollback completed successfully',
          rollback: result
        }
      });
    } else {
      res.status(500).json({
        success: false,
        error: result.error || 'Rollback failed',
        rollback: result
      });
    }
  } catch (error) {
    logger.error('Rollback failed', { error });
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Rollback failed'
    });
  }
});

/**
 * POST /api/deployment/emergency-apply-visitor-tracking
 * Emergency: Apply visitor tracking schema directly (bypass blue-green)
 * Use only when blue-green system is not available
 */
router.post('/emergency-apply-visitor-tracking', async (req, res) => {
  try {
    const { confirm } = req.body;
    
    if (!confirm) {
      return res.status(400).json({
        success: false,
        error: 'Emergency deployment requires explicit confirmation',
        hint: 'Send { "confirm": true } in request body'
      });
    }

    logger.warn('⚠️ Emergency deployment request received: visitor tracking tables');
    
    await migrationService.emergencyApplyVisitorTracking();
    
    res.json({
      success: true,
      data: {
        message: 'Emergency visitor tracking schema applied successfully',
        warning: 'This bypassed the blue-green deployment system'
      }
    });
  } catch (error) {
    logger.error('Emergency deployment failed', { error });
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Emergency deployment failed'
    });
  }
});

/**
 * GET /api/deployment/migration-plans
 * Get available migration plans
 */
router.get('/migration-plans', async (req, res) => {
  try {
    const visitorTrackingPlan = await migrationService.createVisitorTrackingMigrationPlan();
    const analyticsFixPlan = await migrationService.createAnalyticsFixMigrationPlan();
    
    res.json({
      success: true,
      data: {
        available_plans: [
          {
            name: 'visitor_tracking',
            description: 'Deploy visitor tracking tables for analytics',
            plan: visitorTrackingPlan
          },
          {
            name: 'analytics_fix',
            description: 'Fix analytics queries for DuckDB compatibility',
            plan: analyticsFixPlan
          }
        ]
      }
    });
  } catch (error) {
    logger.error('Failed to get migration plans', { error });
    res.status(500).json({
      success: false,
      error: 'Failed to retrieve migration plans'
    });
  }
});

/**
 * POST /api/deployment/test-connection
 * Test database connections for both environments
 */
router.post('/test-connection', async (req, res) => {
  try {
    const health = await migrationService.getHealthStatus();
    
    const results = {
      blue: health.blue ? {
        status: health.blue.healthStatus,
        path: health.blue.path,
        schemaVersion: health.blue.schemaVersion,
        isActive: health.blue.isActive
      } : null,
      green: health.green ? {
        status: health.green.healthStatus,
        path: health.green.path,
        schemaVersion: health.green.schemaVersion,
        isActive: health.green.isActive
      } : null,
      active: health.active
    };
    
    res.json({
      success: true,
      data: {
        message: 'Database connection test completed',
        results
      }
    });
  } catch (error) {
    logger.error('Connection test failed', { error });
    res.status(500).json({
      success: false,
      error: 'Connection test failed'
    });
  }
});

export default router;