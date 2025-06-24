import { Router } from 'express'
import {
  getEvents,
  getEventById,
  getEventCategories,
  getVenues
} from '@/controllers/eventController-duckdb.js'

const router = Router()

// Event routes (public for demo)
router.get('/events', getEvents)
router.get('/events/categories', getEventCategories)
router.get('/events/venues', getVenues)
router.get('/events/:id', getEventById)

// API info endpoint
router.get('/', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Luxury Social Event Platform (DuckDB)',
    version: '1.0.0',
    database: 'DuckDB',
    features: {
      events: 'Full functionality',
      authentication: 'Coming soon',
      payments: 'Coming soon'
    },
    endpoints: {
      events: '/api/events',
      categories: '/api/events/categories',
      venues: '/api/events/venues',
      health: '/api/health'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

// Auth placeholder routes
router.get('/auth/profile', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

router.post('/auth/login', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

router.post('/auth/register', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Authentication not implemented yet',
    message: 'Auth system will be added in next phase'
  })
})

// Catch all for other API routes
router.use('*', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Endpoint not implemented yet',
    message: 'This endpoint will be available in the next phase',
    availableEndpoints: ['/events', '/events/categories', '/events/venues']
  })
})

export default router