import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import { connectDatabases, closeDatabases } from './database/duckdb-connection.js'
import config from './utils/config.js'
import logger from './utils/logger.js'
import createRoutes from './routes/main.js'
import { r2BackupService } from './services/R2BackupService.js'

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
  res.json({
    success: true,
    message: 'HeSocial API is running (DuckDB)',
    timestamp: new Date().toISOString(),
    environment: config.nodeEnv,
    database: 'DuckDB with R2 sync'
  })
})

app.get('/api/health', (req, res) => {
  res.json({
    success: true,
    message: 'API health check passed (DuckDB)',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    database: 'duckdb'
  })
})

app.use((req, res) => {
  res.status(404).json({
    success: false,
    error: 'Route not found',
    message: `${req.method} ${req.path} is not a valid endpoint`
  })
})

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
    
    // Connect to database (includes R2 restore if enabled)
    await connectDatabases()
    
    // Start periodic backups if enabled
    r2BackupService.startPeriodicBackups()
    
    // Dynamically load and mount API routes
    const apiRoutes = await createRoutes()
    app.use('/api', apiRoutes)
    
    const server = app.listen(config.port, () => {
      logger.info('='.repeat(60))
      logger.info('🎉 HESOCIAL API SERVER STARTED SUCCESSFULLY')
      logger.info('='.repeat(60))
      logger.info(`🚀 Server: http://localhost:${config.port}`)
      logger.info(`📱 Environment: ${config.nodeEnv}`)
      logger.info(`🗄️  Database: DuckDB with R2 sync enabled`)
      logger.info(`🔒 CORS Origins: ${config.corsOrigins.join(', ')}`)
      logger.info(`📝 Health Check: http://localhost:${config.port}/api/health`)
      logger.info(`🎯 Events API: http://localhost:${config.port}/api/events`)
      logger.info('='.repeat(60))
    })

    const gracefulShutdown = async (signal: string): Promise<void> => {
      logger.info('='.repeat(60))
      logger.info(`🛑 Received ${signal}. Starting graceful shutdown...`)
      logger.info('='.repeat(60))
      
      // Set a timeout to force shutdown if graceful shutdown takes too long
      const forceShutdownTimeout = setTimeout(() => {
        logger.error('⏰ Forced shutdown due to timeout')
        process.exit(1)
      }, 30000) // 30 seconds timeout
      
      server.close(async () => {
        logger.info('🔒 HTTP server closed')
        
        try {
          // Stop periodic backups
          r2BackupService.stopPeriodicBackups()
          
          // This will trigger R2 backup before closing DuckDB
          await closeDatabases()
          clearTimeout(forceShutdownTimeout)
          logger.info('='.repeat(60))
          logger.info('✅ GRACEFUL SHUTDOWN COMPLETED')
          logger.info('='.repeat(60))
          process.exit(0)
        } catch (error) {
          clearTimeout(forceShutdownTimeout)
          logger.error('❌ Error during graceful shutdown:', error)
          process.exit(1)
        }
      })
      
      // If server.close() callback doesn't execute, force shutdown
      setTimeout(() => {
        logger.error('⏰ Force shutdown - server.close() callback timeout')
        process.exit(1)
      }, 25000) // 25 seconds timeout for server.close()
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