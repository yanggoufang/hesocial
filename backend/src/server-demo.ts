import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import config from './utils/config.js'
import logger from './utils/logger.js'

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
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100,
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
      console.log(message.trim())
    }
  }
}))

app.use(express.json({ limit: '10mb' }))
app.use(express.urlencoded({ extended: true, limit: '10mb' }))

// Health check endpoints
app.get('/health', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API is running (Demo Mode)',
    timestamp: new Date().toISOString(),
    environment: 'development-demo',
    database: 'disconnected'
  })
})

app.get('/api/health', (req, res) => {
  res.json({
    success: true,
    message: 'API health check passed (Demo Mode)',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    mode: 'demo'
  })
})

// Demo API endpoints
app.get('/api', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Luxury Social Event Platform (Demo Mode)',
    version: '1.0.0',
    endpoints: {
      health: '/api/health',
      note: 'Database required for full functionality'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})


// Catch all for API routes
app.use('/api/*', (req, res) => {
  res.status(503).json({
    success: false,
    error: 'Service temporarily unavailable',
    message: 'This endpoint requires database connection. Currently running in demo mode.',
    action: 'Please set up DuckDB for full API functionality'
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
  console.error('Unhandled error:', {
    error: error.message,
    stack: error.stack,
    url: req.url,
    method: req.method
  })

  res.status(500).json({
    success: false,
    error: error.message,
    message: 'An unexpected error occurred'
  })
})

const startDemoServer = (): void => {
  const port = 5000
  
  const server = app.listen(port, () => {
    console.log(`🚀 HeSocial API server running on port ${port} (Demo Mode)`)
    console.log(`📱 Environment: development-demo`)
    console.log(`🔒 CORS Origins: http://localhost:3000`)
    console.log(`⚠️  Database: Not connected - Demo mode only`)
    console.log(`📝 Health Check: http://localhost:${port}/api/health`)
  })

  const gracefulShutdown = (signal: string): void => {
    console.log(`Received ${signal}. Starting graceful shutdown...`)
    
    server.close(() => {
      console.log('HTTP server closed')
      console.log('Graceful shutdown completed')
      process.exit(0)
    })
  }

  process.on('SIGTERM', () => gracefulShutdown('SIGTERM'))
  process.on('SIGINT', () => gracefulShutdown('SIGINT'))
}

if (import.meta.url === `file://${process.argv[1]}`) {
  startDemoServer()
}

export default app