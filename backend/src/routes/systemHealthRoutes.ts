import { Router } from 'express'
import { requireAdmin } from '../middleware/auth.js'
import {
  getSystemHealth,
  getDetailedHealthCheck,
  getSystemMetrics,
  runSystemDiagnostics
} from '../controllers/systemHealthController.js'

const router = Router()

// All system health routes require admin access
router.use(requireAdmin)

/**
 * GET /api/system/health
 * Comprehensive system health dashboard data
 */
router.get('/health', getSystemHealth)

/**
 * GET /api/system/health/detailed
 * Detailed health check with all components
 */
router.get('/health/detailed', getDetailedHealthCheck)

/**
 * GET /api/system/metrics
 * System performance metrics
 */
router.get('/metrics', getSystemMetrics)

/**
 * GET /api/system/diagnostics
 * Run system diagnostics tests
 */
router.get('/diagnostics', runSystemDiagnostics)

export default router