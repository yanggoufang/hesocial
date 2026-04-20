import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import cookieParser from 'cookie-parser'
import { connectDatabases, closeDatabases } from './database/duckdb-connection.js'
import config from './utils/config.js'
import logger from './utils/logger.js'
import createRoutes from './routes/main.js'
import { r2BackupService } from './services/R2BackupService.js'
import { visitorTracking } from './middleware/visitorTracking.js'

;(BigInt.prototype as unknown as { toJSON: () => number }).toJSON = function () {
  const value = this as unknown as bigint
  const asNumber = Number(value)
  if (!Number.isSafeInteger(asNumber)) {
    return value.toString() as unknown as number
  }
  return asNumber
}

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
  origin: [...config.corsOrigins, 'http://127.0.0.1:3000', 'http://localhost:5000'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With']
}))

app.use(compression())

const generalLimiter = rateLimit({
  windowMs: config.rateLimitWindowMinutes * 60 * 1000,
  max: config.rateLimitMaxRequests,
  message: JSON.stringify({
    success: false,
    error: 'Too many requests, please try again later.'
  }),
  standardHeaders: true,
  legacyHeaders: false,
});

const adminLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 2000, // More permissive for admin actions
  message: JSON.stringify({
    success: false,
    error: 'Admin rate limit exceeded, please try again later.'
  }),
  standardHeaders: true,
  legacyHeaders: false,
});

// Apply more permissive rate limiting to admin and event management routes first
app.use('/api/admin', adminLimiter);
app.use('/api/events/manage', adminLimiter);
app.use('/api/events/admin', adminLimiter);
app.use('/api/events/venues', adminLimiter);
app.use('/api/events/categories', adminLimiter);

// Apply general rate limiting to all other /api routes
app.use('/api', generalLimiter);

app.use(morgan('combined', {
  stream: {
    write: (message: string) => {
      logger.info(message.trim())
    }
  }
}))

app.use(express.json({ limit: '10mb' }))
app.use(express.urlencoded({ extended: true, limit: '10mb' }))
app.use(cookieParser())

// Visitor tracking middleware (before routes)
app.use(visitorTracking)

// Health check endpoints
app.get('/health', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API is running (DuckDB)',
    timestamp: new Date().toISOString(),
    environment: config.nodeEnv,
    database: `DuckDB with R2 sync ${r2BackupService.isEnabled() ? 'enabled' : 'disabled'}`
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

// Comprehensive health API interface
app.get('/api/health/status', (req, res) => {
  const uptime = process.uptime()
  const memUsage = process.memoryUsage()
  
  res.json({
    success: true,
    status: 'healthy',
    server: {
      uptime: Math.floor(uptime),
      uptimeFormatted: `${Math.floor(uptime / 3600)}h ${Math.floor((uptime % 3600) / 60)}m ${Math.floor(uptime % 60)}s`,
      memory: {
        rss: Math.round(memUsage.rss / 1024 / 1024) + 'MB',
        heapUsed: Math.round(memUsage.heapUsed / 1024 / 1024) + 'MB',
        heapTotal: Math.round(memUsage.heapTotal / 1024 / 1024) + 'MB'
      },
      nodeVersion: process.version,
      platform: process.platform
    },
    database: {
      type: 'DuckDB',
      r2Sync: r2BackupService.isEnabled() ? 'enabled' : 'disabled'
    },
    environment: config.nodeEnv,
    timestamp: new Date().toISOString()
  })
})

app.get('/api/health/routes', (req, res) => {
  const routes = []
  
  // Function to extract routes from Express app
  const extractRoutes = (stack, basePath = '') => {
    stack.forEach(layer => {
      if (layer.route) {
        // Direct route
        const methods = Object.keys(layer.route.methods).join(', ').toUpperCase()
        routes.push({
          path: basePath + layer.route.path,
          methods: methods,
          type: 'route'
        })
      } else if (layer.name === 'router' && layer.handle.stack) {
        // Router middleware
        const routerPath = layer.regexp.toString().match(/^\/\^\\?(.*?)\$?\//)?.[1] || ''
        const cleanPath = routerPath.replace(/\\\//g, '/').replace(/\?\?\$/g, '')
        extractRoutes(layer.handle.stack, basePath + cleanPath)
      }
    })
  }
  
  try {
    extractRoutes(app._router.stack)
    
    res.json({
      success: true,
      routes: {
        total: routes.length,
        endpoints: routes.sort((a, b) => a.path.localeCompare(b.path))
      },
      timestamp: new Date().toISOString()
    })
  } catch (error) {
    res.json({
      success: false,
      error: 'Failed to extract routes',
      message: error instanceof Error ? error.message : 'Unknown error',
      timestamp: new Date().toISOString()
    })
  }
})

app.get('/api/health/middleware', (req, res) => {
  const middlewares = []
  
  try {
    app._router.stack.forEach((layer, index) => {
      middlewares.push({
        index,
        name: layer.name || 'anonymous',
        regexp: layer.regexp.toString(),
        fast_star: layer.fast_star,
        fast_slash: layer.fast_slash
      })
    })
    
    res.json({
      success: true,
      middleware: {
        total: middlewares.length,
        stack: middlewares
      },
      timestamp: new Date().toISOString()
    })
  } catch (error) {
    res.json({
      success: false,
      error: 'Failed to inspect middleware',
      message: error instanceof Error ? error.message : 'Unknown error',
      timestamp: new Date().toISOString()
    })
  }
})

app.get('/api/health/config', (req, res) => {
  res.json({
    success: true,
    config: {
      port: config.port,
      nodeEnv: config.nodeEnv,
      corsOrigins: config.corsOrigins,
      rateLimit: {
        windowMinutes: config.rateLimitWindowMinutes,
        maxRequests: config.rateLimitMaxRequests
      },
      r2: {
        enabled: r2BackupService.isEnabled(),
        endpoint: process.env.R2_ENDPOINT || 'not configured',
        bucket: process.env.R2_BUCKET_NAME || 'not configured'
      }
    },
    timestamp: new Date().toISOString()
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
    try {
      logger.info('🔄 Loading API routes...')
      const apiRoutes = await createRoutes()
      
      if (apiRoutes) {
        logger.info('🔍 API Routes object type:', typeof apiRoutes)
        logger.info('🔍 API Routes constructor:', apiRoutes.constructor.name)
        logger.info('🔍 API Routes stack length:', apiRoutes.stack ? apiRoutes.stack.length : 'No stack')
        app.use('/api', apiRoutes)
        logger.info('✅ API routes loaded and mounted successfully')
      } else {
        throw new Error('API routes object is null/undefined')
      }
    } catch (error) {
      console.error('FATAL: API route mount failed:', error)
      logger.error('❌ Failed to load API routes:', error)
      throw error
    }
    
    // Add 404 handler after routes
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
    
    const server = app.listen(config.port, () => {
      logger.info('='.repeat(60))
      logger.info('🎉 HESOCIAL API SERVER STARTED SUCCESSFULLY')
      logger.info('='.repeat(60))
      logger.info(`🚀 Server: http://localhost:${config.port}`)
      logger.info(`📱 Environment: ${config.nodeEnv}`)
      const r2Status = r2BackupService.isEnabled() ? 'enabled' : 'disabled'
      logger.info(`🗄️  Database: DuckDB with R2 sync ${r2Status}`)
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
