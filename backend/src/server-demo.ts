import express from 'express'
import cors from 'cors'
import helmet from 'helmet'
import compression from 'compression'
import rateLimit from 'express-rate-limit'
import morgan from 'morgan'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

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
      events: '/api/events (demo)',
      auth: '/api/auth (demo)',
      note: 'Database required for full functionality'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

// Mock events endpoint
app.get('/api/events', (req, res) => {
  const mockEvents = [
    {
      id: '1',
      name: '星空下的法式晚宴',
      description: '與米其林三星主廚共享精緻法式料理，在台北101頂樓欣賞城市夜景',
      dateTime: '2024-12-27T19:00:00',
      venue: {
        name: '台北文華東方酒店',
        address: '台北市松山區敦化北路166號'
      },
      pricing: {
        vvip: 15000,
        vip: 10000,
        currency: 'TWD'
      },
      exclusivityLevel: 'VVIP',
      capacity: 20,
      currentAttendees: 12
    },
    {
      id: '2',
      name: '私人遊艇品酒之夜',
      description: '在豪華遊艇上品嚐世界頂級香檳，與成功企業家建立深度連結',
      dateTime: '2024-12-15T18:30:00',
      venue: {
        name: '基隆港VIP碼頭',
        address: '基隆市中正區中正路1號'
      },
      pricing: {
        vip: 8000,
        currency: 'TWD'
      },
      exclusivityLevel: 'VIP',
      capacity: 16,
      currentAttendees: 8
    }
  ]

  res.json({
    success: true,
    data: mockEvents,
    message: 'Demo data - Connect database for full functionality'
  })
})

// Mock categories endpoint
app.get('/api/events/categories', (req, res) => {
  const mockCategories = [
    { id: '1', name: '私人晚宴', description: '高級米其林餐廳的獨家用餐體驗', icon: 'utensils' },
    { id: '2', name: '遊艇派對', description: '豪華遊艇上的社交聚會', icon: 'anchor' },
    { id: '3', name: '藝術沙龍', description: '當代藝術展覽與收藏品鑑賞', icon: 'palette' },
    { id: '4', name: '商務社交', description: '高端商務人士的networking活動', icon: 'briefcase' }
  ]

  res.json({
    success: true,
    data: mockCategories,
    message: 'Demo data - Connect database for full functionality'
  })
})

// Catch all for API routes
app.use('/api/*', (req, res) => {
  res.status(503).json({
    success: false,
    error: 'Service temporarily unavailable',
    message: 'This endpoint requires database connection. Currently running in demo mode.',
    action: 'Please set up PostgreSQL and Redis to enable full API functionality'
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
    console.log(`⚠️  Database: Not connected - Running with mock data`)
    console.log(`📝 Health Check: http://localhost:${port}/api/health`)
    console.log(`🎯 Mock Events: http://localhost:${port}/api/events`)
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