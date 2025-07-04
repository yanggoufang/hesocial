import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import { connectDatabases, closeDatabases, getDatabaseInfo } from './database/connection.js'
import config from './utils/config.js'
import logger from './utils/logger.js'
import createRoutes from './routes/main.js'
import { R2BackupService } from './services/r2-backup.js'
import { StartupHealthCheck } from './services/StartupHealthCheck.js'
import { ServerStateService } from './services/ServerStateService.js'
import { productionConfig } from './config/production.js'

const app = express()

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'", "https:"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'"],
      fontSrc: ["'self'", "https:", "data:"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"],
    },
  },
  crossOriginEmbedderPolicy: false
}))

app.use(cors({
  origin: config.corsOrigins,
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With']
}))

app.use(compression())

const limiter = rateLimit({
  windowMs: config.rateLimitWindowMinutes * 60 * 1000,
  max: config.rateLimitMaxRequests,
  message: JSON.stringify({
    success: false,
    error: 'Too many requests, please try again later.'
  }),
  standardHeaders: true,
  legacyHeaders: false,
})

app.use(limiter)

app.use(morgan('combined', {
  stream: {
    write: (message: string) => {
      logger.info(message.trim())
    }
  }
}))

app.use(express.json({ limit: '10mb' }))
app.use(express.urlencoded({ extended: true, limit: '10mb' }))

// Health check endpoints
app.get('/health', (req, res) => {
  const dbInfo = getDatabaseInfo()
  res.json({
    success: true,
    message: `HeSocial API is running (${dbInfo.type.toUpperCase()})`,
    timestamp: new Date().toISOString(),
    environment: config.nodeEnv,
    database: dbInfo.description
  })
})

app.get('/api/health', (req, res) => {
  const dbInfo = getDatabaseInfo()
  res.json({
    success: true,
    message: `API health check passed (${dbInfo.type.toUpperCase()})`,
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    database: dbInfo.type
  })
})

// Initialize backup service and API routes in startServer function

app.use((error: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
  logger.error('Unhandled error:', {
    error: error.message,
    stack: error.stack,
    url: req.url,
    method: req.method,
    body: req.body,
    params: req.params,
    query: req.query
  })

  res.status(500).json({
    success: false,
    error: config.nodeEnv === 'production' ? 'Internal server error' : error.message,
    message: 'An unexpected error occurred'
  })
})

const startServer = async (): Promise<void> => {
  try {
    logger.info('🚀 Starting HeSocial API Server...')
    
    // 1. Validate production configuration
    logger.info('⚙️  Validating configuration...')
    productionConfig.logConfigurationStatus()
    
    const configValidation = productionConfig.validateConfiguration()
    if (!configValidation.isValid) {
      logger.error('❌ Configuration validation failed:')
      configValidation.errors.forEach(error => logger.error(`   • ${error}`))
      throw new Error('Invalid configuration - server startup aborted')
    }
    
    // 2. Initialize services
    logger.info('🔧 Initializing services...')
    const backupService = new R2BackupService()
    const healthCheck = new StartupHealthCheck()
    const serverState = new ServerStateService()
    
    // 3. Attempt smart restore before database connection
    logger.info('💾 Checking for database restore...')
    const restored = await backupService.smartRestore()
    if (restored) {
      logger.info('✅ Database restored from backup')
    }
    
    // 4. Connect to database
    logger.info('🗄️  Connecting to database...')
    const dbInfo = getDatabaseInfo()
    await connectDatabases()
    
    // 5. Initialize server state tracking
    logger.info('📊 Initializing server state tracking...')
    await serverState.initialize()
    
    // 6. Run comprehensive health checks
    logger.info('🏥 Running health checks...')
    const healthSummary = await healthCheck.runHealthChecks()
    
    // 7. Validate production readiness
    const isProductionReady = healthCheck.validateProductionReadiness(healthSummary)
    if (!isProductionReady) {
      throw new Error('System failed health checks - startup aborted')
    }
    
    // 8. Load and mount API routes
    logger.info('🛣️  Loading API routes...')
    const apiRoutes = await createRoutes()
    app.use('/api', apiRoutes)
    
    // Enhanced health check with server stats
    app.get('/api/health/detailed', async (req, res) => {
      try {
        const stats = await serverState.getServerStats()
        const healthSummary = await healthCheck.runHealthChecks()
        
        res.json({
          success: true,
          message: 'Detailed health check',
          timestamp: new Date().toISOString(),
          environment: config.nodeEnv,
          database: dbInfo.description,
          health: healthSummary,
          serverStats: stats,
          uptime: serverState.getFormattedUptime()
        })
      } catch (error) {
        res.status(500).json({
          success: false,
          error: 'Health check failed',
          message: error instanceof Error ? error.message : 'Unknown error'
        })
      }
    })
    
    // Catch-all route for unmatched requests (must be after API routes)
    app.use((req, res) => {
      res.status(404).json({
        success: false,
        error: 'Route not found',
        message: `${req.method} ${req.path} is not a valid endpoint`
      })
    })
    
    // 9. Start the server
    logger.info('🌐 Starting HTTP server...')
    const server = app.listen(config.port, () => {
      logger.info('='.repeat(60))
      logger.info('🎉 HESOCIAL API SERVER STARTED SUCCESSFULLY')
      logger.info('='.repeat(60))
      logger.info(`🚀 Server: http://localhost:${config.port}`)
      logger.info(`📱 Environment: ${config.nodeEnv}`)
      logger.info(`🗄️  Database: ${dbInfo.description}`)
      logger.info(`🔒 CORS Origins: ${config.corsOrigins.join(', ')}`)
      logger.info(`📝 Health Check: http://localhost:${config.port}/api/health`)
      logger.info(`🔍 Detailed Health: http://localhost:${config.port}/api/health/detailed`)
      logger.info(`🎯 Events API: http://localhost:${config.port}/api/events`)
      logger.info(`⏱️  Server Uptime: ${serverState.getFormattedUptime()}`)
      logger.info('='.repeat(60))
      
      // Log server statistics
      serverState.logServerStats()
    })

    const gracefulShutdown = async (signal: string): Promise<void> => {
      logger.info('='.repeat(60))
      logger.info(`🛑 Received ${signal}. Starting graceful shutdown...`)
      logger.info('='.repeat(60))
      
      server.close(async () => {
        logger.info('📡 HTTP server closed')
        
        try {
          // Record server stop time
          logger.info('📊 Recording server state...')
          await serverState.recordServerStop()
          
          // Create backup before closing database
          logger.info('💾 Creating backup before shutdown...')
          const backupId = await backupService.backupOnShutdown()
          if (backupId) {
            logger.info(`✅ Shutdown backup created: ${backupId}`)
          } else {
            logger.warn('⚠️  Shutdown backup failed or disabled')
          }
          
          // Close database connections
          logger.info('🗄️  Closing database connections...')
          await closeDatabases()
          
          // Log final server statistics
          await serverState.logServerStats()
          
          logger.info('='.repeat(60))
          logger.info('✅ GRACEFUL SHUTDOWN COMPLETED')
          logger.info('='.repeat(60))
          process.exit(0)
        } catch (error) {
          logger.error('❌ Error during graceful shutdown:', error)
          process.exit(1)
        }
      })
      
      // Force shutdown after 30 seconds
      setTimeout(() => {
        logger.error('🚨 Forced shutdown after timeout')
        process.exit(1)
      }, 30000)
    }

    process.on('SIGTERM', () => gracefulShutdown('SIGTERM'))
    process.on('SIGINT', () => gracefulShutdown('SIGINT'))

  } catch (error) {
    logger.error('Failed to start server:', error)
    process.exit(1)
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  startServer()
}

export default app