import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import { connectDatabases, closeDatabases } from '@/database/duckdb-connection.js'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'
import apiRoutes from '@/routes/index-duckdb.js'

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

app.get('/health', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API is running (DuckDB)',
    timestamp: new Date().toISOString(),
    environment: config.nodeEnv,
    database: 'DuckDB - Full functionality enabled'
  })
})

app.get('/api/health', (req, res) => {
  res.json({
    success: true,
    message: 'API health check passed (DuckDB)',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    database: 'DuckDB'
  })
})

// Mount API routes
app.use('/api', apiRoutes)

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
    await connectDatabases()
    
    const server = app.listen(config.port, () => {
      logger.info(`🚀 HeSocial API server running on port ${config.port} (DuckDB)`)
      logger.info(`📱 Environment: ${config.nodeEnv}`)
      logger.info(`🗄️  Database: DuckDB (Full functionality)`)
      logger.info(`🔒 CORS Origins: ${config.corsOrigins.join(', ')}`)
      logger.info(`📝 Health Check: http://localhost:${config.port}/api/health`)
      logger.info(`🎯 Events API: http://localhost:${config.port}/api/events`)
    })

    const gracefulShutdown = async (signal: string): Promise<void> => {
      logger.info(`Received ${signal}. Starting graceful shutdown...`)
      
      server.close(async () => {
        logger.info('HTTP server closed')
        
        try {
          await closeDatabases()
          logger.info('Graceful shutdown completed')
          process.exit(0)
        } catch (error) {
          logger.error('Error during graceful shutdown:', error)
          process.exit(1)
        }
      })
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