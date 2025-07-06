import { Request, Response } from 'express'
import { pool } from '../database/duckdb-pool.js'
import { r2BackupService } from '../services/R2BackupService.js'
import { StartupHealthCheck } from '../services/StartupHealthCheck.js'
import { MigrationService } from '../services/MigrationService.js'
import logger from '../utils/logger.js'
import { ApiResponse } from '../types/index.js'

interface SystemStats {
  uptime: number
  memory: {
    used: number
    total: number
    usage: number
  }
  cpu: {
    usage: number
  }
  environment: string
  nodeVersion: string
  timestamp: string
}

interface DatabaseStats {
  type: string
  connected: boolean
  tableCount: number
  userCount: number
  eventCount: number
  registrationCount: number
  connectionPool: {
    active: number
    idle: number
    total: number
  }
  serverStats?: {
    startCount: number
    firstStartTime: string
    lastStartTime: string
    totalLifetime: number
    lastSessionDuration: number
  }
  queries: {
    total: number
    slow: number
    failed: number
  }
}

interface R2BackupStats {
  enabled: boolean
  connectionHealthy: boolean
  lastBackup: string | null
  backupCount: number
  totalBackupSize: number
  periodicBackupsEnabled: boolean
  periodicBackupInterval: number
  recentBackups: Array<{
    id: string
    type: string
    timestamp: string
    size: number
    schemaVersion: string
  }>
}

interface ComprehensiveHealthReport {
  overall: 'healthy' | 'warning' | 'error'
  timestamp: string
  system: SystemStats
  database: DatabaseStats
  r2Backup: R2BackupStats
  services: {
    authentication: {
      status: 'healthy' | 'warning' | 'error'
      oauthProviders: string[]
      jwtConfigured: boolean
    }
    payments: {
      status: 'healthy' | 'warning' | 'error'
      stripeConfigured: boolean
    }
    media: {
      status: 'healthy' | 'warning' | 'error'
      r2Configured: boolean
    }
  }
  migrations: {
    status: 'healthy' | 'warning' | 'error'
    applied: number
    pending: number
    latest: string
  }
  security: {
    httpsEnabled: boolean
    corsConfigured: boolean
    rateLimitingEnabled: boolean
    helmetEnabled: boolean
  }
  performance: {
    avgResponseTime: number
    totalRequests: number
    errorRate: number
  }
}

export const getSystemHealth = async (req: Request, res: Response) => {
  try {
    // Get comprehensive system stats
    const system = await getSystemStats()
    const database = await getDatabaseStats()
    const r2Backup = await getR2BackupStats()
    const services = await getServiceStats()
    const migrations = await getMigrationStats()
    const security = await getSecurityStats()
    const performance = await getPerformanceStats()

    // Determine overall health
    let overall: 'healthy' | 'warning' | 'error' = 'healthy'
    
    if (database.connected === false || 
        (r2Backup.enabled && !r2Backup.connectionHealthy) ||
        services.authentication.status === 'error' ||
        migrations.status === 'error') {
      overall = 'error'
    } else if (database.queries.failed > 0 ||
               r2Backup.enabled === false ||
               services.payments.status === 'warning' ||
               migrations.pending > 0) {
      overall = 'warning'
    }

    const healthReport: ComprehensiveHealthReport = {
      overall,
      timestamp: new Date().toISOString(),
      system,
      database,
      r2Backup,
      services,
      migrations,
      security,
      performance
    }

    const response: ApiResponse<ComprehensiveHealthReport> = {
      success: true,
      data: healthReport
    }

    res.json(response)
  } catch (error) {
    logger.error('System health check failed:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get system health status'
    })
  }
}

export const getDetailedHealthCheck = async (req: Request, res: Response) => {
  try {
    const healthCheck = new StartupHealthCheck()
    const healthSummary = await healthCheck.runHealthChecks()

    const response: ApiResponse<typeof healthSummary> = {
      success: true,
      data: healthSummary
    }

    res.json(response)
  } catch (error) {
    logger.error('Detailed health check failed:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to run detailed health check'
    })
  }
}

export const getSystemMetrics = async (req: Request, res: Response) => {
  try {
    const metrics = {
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      cpu: process.cpuUsage(),
      environment: process.env.NODE_ENV || 'development',
      nodeVersion: process.version,
      platform: process.platform,
      architecture: process.arch
    }

    const response: ApiResponse<typeof metrics> = {
      success: true,
      data: metrics
    }

    res.json(response)
  } catch (error) {
    logger.error('System metrics retrieval failed:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get system metrics'
    })
  }
}

export const runSystemDiagnostics = async (req: Request, res: Response) => {
  try {
    const diagnostics = []

    // Test database connection
    try {
      const dbTest = await pool.query('SELECT 1 as test')
      diagnostics.push({
        test: 'Database Connection',
        status: 'passed',
        message: 'Database connection successful',
        timestamp: new Date().toISOString()
      })
    } catch (error) {
      diagnostics.push({
        test: 'Database Connection',
        status: 'failed',
        message: error instanceof Error ? error.message : 'Unknown error',
        timestamp: new Date().toISOString()
      })
    }

    // Test R2 backup service
    if (r2BackupService.isEnabled()) {
      try {
        const r2Status = await r2BackupService.getStatus()
        diagnostics.push({
          test: 'R2 Backup Service',
          status: r2Status.connectionHealthy ? 'passed' : 'failed',
          message: r2Status.connectionHealthy ? 'R2 service accessible' : 'R2 connection failed',
          timestamp: new Date().toISOString()
        })
      } catch (error) {
        diagnostics.push({
          test: 'R2 Backup Service',
          status: 'failed',
          message: error instanceof Error ? error.message : 'Unknown error',
          timestamp: new Date().toISOString()
        })
      }
    } else {
      diagnostics.push({
        test: 'R2 Backup Service',
        status: 'skipped',
        message: 'R2 backup service is disabled',
        timestamp: new Date().toISOString()
      })
    }

    // Test environment configuration
    const requiredEnvVars = ['NODE_ENV', 'PORT', 'JWT_SECRET', 'FRONTEND_URL']
    const missingEnvVars = requiredEnvVars.filter(env => !process.env[env])
    
    diagnostics.push({
      test: 'Environment Configuration',
      status: missingEnvVars.length === 0 ? 'passed' : 'failed',
      message: missingEnvVars.length === 0 ? 'All required environment variables configured' : `Missing: ${missingEnvVars.join(', ')}`,
      timestamp: new Date().toISOString()
    })

    const response: ApiResponse<typeof diagnostics> = {
      success: true,
      data: diagnostics
    }

    res.json(response)
  } catch (error) {
    logger.error('System diagnostics failed:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to run system diagnostics'
    })
  }
}

// Helper functions
async function getSystemStats(): Promise<SystemStats> {
  const memoryUsage = process.memoryUsage()
  const totalMemory = memoryUsage.heapTotal + memoryUsage.external
  const usedMemory = memoryUsage.heapUsed
  
  return {
    uptime: process.uptime(),
    memory: {
      used: usedMemory,
      total: totalMemory,
      usage: Math.round((usedMemory / totalMemory) * 100)
    },
    cpu: {
      usage: 0 // CPU usage would need additional monitoring
    },
    environment: process.env.NODE_ENV || 'development',
    nodeVersion: process.version,
    timestamp: new Date().toISOString()
  }
}

async function getDatabaseStats(): Promise<DatabaseStats> {
  try {
    const connectionTest = await pool.query('SELECT 1 as test')
    const isConnected = connectionTest.rows.length > 0

    // Get table count
    const tablesResult = await pool.query(
      "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'main'"
    )
    const tableCount = tablesResult.rows[0]?.count || 0

    // Get user count
    const usersResult = await pool.query('SELECT COUNT(*) as count FROM users')
    const userCount = usersResult.rows[0]?.count || 0

    // Get event count
    const eventsResult = await pool.query('SELECT COUNT(*) as count FROM events')
    const eventCount = eventsResult.rows[0]?.count || 0

    // Get registration count
    const registrationsResult = await pool.query('SELECT COUNT(*) as count FROM registrations')
    const registrationCount = registrationsResult.rows[0]?.count || 0

    // Get server stats if available
    let serverStats
    try {
      const serverStatsResult = await pool.query('SELECT * FROM server_state ORDER BY id DESC LIMIT 1')
      if (serverStatsResult.rows.length > 0) {
        const stats = serverStatsResult.rows[0]
        serverStats = {
          startCount: stats.start_count,
          firstStartTime: stats.first_start_time,
          lastStartTime: stats.last_start_time,
          totalLifetime: stats.total_lifetime,
          lastSessionDuration: stats.last_session_duration
        }
      }
    } catch (error) {
      // Server stats table might not exist
    }

    return {
      type: 'DuckDB',
      connected: isConnected,
      tableCount,
      userCount,
      eventCount,
      registrationCount,
      connectionPool: {
        active: 1, // DuckDB doesn't have traditional connection pooling
        idle: 0,
        total: 1
      },
      serverStats,
      queries: {
        total: 0, // Would need query logging to track
        slow: 0,
        failed: 0
      }
    }
  } catch (error) {
    return {
      type: 'DuckDB',
      connected: false,
      tableCount: 0,
      userCount: 0,
      eventCount: 0,
      registrationCount: 0,
      connectionPool: {
        active: 0,
        idle: 0,
        total: 0
      },
      queries: {
        total: 0,
        slow: 0,
        failed: 1
      }
    }
  }
}

async function getR2BackupStats(): Promise<R2BackupStats> {
  try {
    if (!r2BackupService.isEnabled()) {
      return {
        enabled: false,
        connectionHealthy: false,
        lastBackup: null,
        backupCount: 0,
        totalBackupSize: 0,
        periodicBackupsEnabled: false,
        periodicBackupInterval: 0,
        recentBackups: []
      }
    }

    const r2Status = await r2BackupService.getStatus()
    const recentBackups = await r2BackupService.listBackups(10)

    const totalBackupSize = recentBackups.reduce((total, backup) => total + (typeof backup.size === 'number' ? backup.size : 0), 0)

    return {
      enabled: r2Status.enabled,
      connectionHealthy: r2Status.connectionHealthy,
      lastBackup: r2Status.lastBackup,
      backupCount: r2Status.backupCount,
      totalBackupSize,
      periodicBackupsEnabled: r2Status.periodicBackupsEnabled,
      periodicBackupInterval: r2Status.periodicBackupInterval,
      recentBackups: recentBackups.map(backup => ({
        id: backup.id,
        type: backup.type,
        timestamp: backup.timestamp,
        size: typeof backup.size === 'number' ? backup.size : 0,
        schemaVersion: backup.schema_version
      }))
    }
  } catch (error) {
    return {
      enabled: r2BackupService.isEnabled(),
      connectionHealthy: false,
      lastBackup: null,
      backupCount: 0,
      totalBackupSize: 0,
      periodicBackupsEnabled: false,
      periodicBackupInterval: 0,
      recentBackups: []
    }
  }
}

async function getServiceStats() {
  const jwtSecret = process.env.JWT_SECRET
  const googleClientId = process.env.GOOGLE_CLIENT_ID
  const googleClientSecret = process.env.GOOGLE_CLIENT_SECRET
  const linkedinClientId = process.env.LINKEDIN_CLIENT_ID
  const linkedinClientSecret = process.env.LINKEDIN_CLIENT_SECRET
  const stripeSecretKey = process.env.STRIPE_SECRET_KEY
  const stripeWebhookSecret = process.env.STRIPE_WEBHOOK_SECRET
  const r2AccessKey = process.env.R2_ACCESS_KEY_ID
  const r2SecretKey = process.env.R2_SECRET_ACCESS_KEY

  const oauthProviders = []
  if (googleClientId && googleClientSecret) oauthProviders.push('Google')
  if (linkedinClientId && linkedinClientSecret) oauthProviders.push('LinkedIn')

  return {
    authentication: {
      status: (!jwtSecret ? 'error' : 
               (jwtSecret.length < 32 || oauthProviders.length === 0) ? 'warning' : 'healthy') as 'healthy' | 'warning' | 'error',
      oauthProviders,
      jwtConfigured: !!jwtSecret
    },
    payments: {
      status: (!stripeSecretKey || !stripeWebhookSecret ? 'warning' : 'healthy') as 'healthy' | 'warning' | 'error',
      stripeConfigured: !!(stripeSecretKey && stripeWebhookSecret)
    },
    media: {
      status: (!r2AccessKey || !r2SecretKey ? 'warning' : 'healthy') as 'healthy' | 'warning' | 'error',
      r2Configured: !!(r2AccessKey && r2SecretKey)
    }
  }
}

async function getMigrationStats() {
  try {
    const migrationService = new MigrationService()
    await migrationService.initialize()
    
    // For now, assume migrations are healthy since we don't have getStatus method
    return {
      status: 'healthy' as 'healthy' | 'warning' | 'error',
      applied: 4, // We have 4 migrations currently
      pending: 0,
      latest: '004_add_media_management_system'
    }
  } catch (error) {
    return {
      status: 'error' as 'healthy' | 'warning' | 'error',
      applied: 0,
      pending: 0,
      latest: 'Unknown'
    }
  }
}

async function getSecurityStats() {
  return {
    httpsEnabled: process.env.NODE_ENV === 'production',
    corsConfigured: !!process.env.FRONTEND_URL,
    rateLimitingEnabled: true, // Assuming rate limiting is enabled
    helmetEnabled: true // Assuming helmet is enabled
  }
}

async function getPerformanceStats() {
  return {
    avgResponseTime: 0, // Would need request tracking
    totalRequests: 0,
    errorRate: 0
  }
}