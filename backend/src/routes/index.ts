import { Router } from 'express'
import {
  getEvents,
  getEventById,
  getEventCategories,
  getVenues
} from '../controllers/eventController.js'
import { optionalAuth, protect, requireAdmin } from '../middleware/auth.js'
import healthRoutes from './health.js'
import placeholderRoutes from './placeholderRoutes.js'
import authRoutes from './authRoutes.js'
import salesManagementRoutes from './salesManagement.js'
import participantRoutes from './participants.js'
import registrationRoutes from './registrationRoutes.js'
import systemHealthRoutes from './systemHealthRoutes.js'
import analyticsRoutes from './analyticsRoutes.js'
import deploymentRoutes from './deploymentRoutes.js'
import emergencyRoutes from './emergencyRoutes.js'
import debugRoutes from './debugRoutes.js'
import adminRoutes from './admin.js'
import userManagementRoutes from './userManagement.js'
import eventManagementRoutes from './eventManagement.js'
import venueManagementRoutes from './venueManagement.js'
import categoryManagementRoutes from './categoryManagement.js'
import mediaRoutes from './mediaRoutes.js'

const router = Router()

// Public endpoints
router.use('/health', healthRoutes)
router.use('/placeholder', placeholderRoutes)
router.use('/auth', authRoutes)

router.get('/events', optionalAuth, getEvents)
router.get('/events/:id', optionalAuth, getEventById)
router.get('/categories', getEventCategories)
router.get('/venues', getVenues)

router.get('/media/events/:eventId', async (req, res) => {
  res.json({
    success: true,
    data: []
  })
})

router.get('/', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Luxury Social Event Platform (DuckDB)',
    version: '1.0.0',
    database: 'DuckDB',
    features: {
      authentication: 'Production Ready - JWT + Google OAuth',
      events: 'Event browsing enabled; management routes pending remediation',
      venues: 'Venue browsing enabled; management routes pending remediation',
      categories: 'Event category browsing enabled',
      salesManagement: 'Sales pipeline API enabled',
      payments: 'Payment Processing - Coming soon'
    },
    publicEndpoints: {
      events: 'GET /api/events',
      eventDetails: 'GET /api/events/:id',
      categories: 'GET /api/categories',
      venues: 'GET /api/venues',
      register: 'POST /api/auth/register',
      login: 'POST /api/auth/login',
      googleAuth: 'GET /api/auth/google',
      health: 'GET /api/health/*'
    },
    protectedEndpoints: {
      profile: 'GET/PUT /api/auth/profile',
      registrations: '/api/registrations/*',
      participants: '/api/events/:eventId/participants',
      sales: '/api/sales/*',
      system: '/api/system/*',
      analytics: '/api/analytics/*',
      deployment: '/api/deployment/*',
      emergency: '/api/emergency/*'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

router.get('/test', (req, res) => {
  res.json({
    success: true,
    message: 'API routes are working!',
    timestamp: new Date().toISOString()
  })
})

// Protected endpoints
router.use('/registrations', registrationRoutes)
router.use('/', participantRoutes)
router.use('/sales', salesManagementRoutes)
router.use('/system', systemHealthRoutes)
router.use('/analytics', analyticsRoutes)
router.use('/deployment', deploymentRoutes)
router.use('/emergency', emergencyRoutes)

// Temporarily disabled routes that may have import/runtime issues.
router.use('/admin', adminRoutes)
router.use('/users', userManagementRoutes)
router.use('/events', eventManagementRoutes)
router.use('/venues', venueManagementRoutes)
router.use('/categories', categoryManagementRoutes)
router.use('/media', mediaRoutes)

// Legacy event routes for backwards compatibility during transition.
router.get('/legacy/events', optionalAuth, getEvents)
router.get('/legacy/events/categories', getEventCategories)
router.get('/legacy/events/venues', getVenues)
router.get('/legacy/events/:id', optionalAuth, getEventById)

if (process.env.NODE_ENV !== 'production') {
  router.use('/debug', protect, requireAdmin, debugRoutes)
}

router.use('*', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'Endpoint not implemented yet',
    message: 'This endpoint will be available in a later remediation phase',
    availableEndpoints: ['/events', '/categories', '/venues', '/auth/register', '/auth/login']
  })
})

export default router
