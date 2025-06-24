import { Router } from 'express'
import authRoutes from './authRoutes.js'
import eventRoutes from './eventRoutes.js'
import registrationRoutes from './registrationRoutes.js'
import paymentRoutes from './paymentRoutes.js'

const router = Router()

// Mount routes
router.use('/auth', authRoutes)
router.use('/events', eventRoutes)
router.use('/registrations', registrationRoutes)
router.use('/payments', paymentRoutes)

// API info endpoint
router.get('/', (req, res) => {
  res.json({
    success: true,
    message: 'HeSocial API - Luxury Social Event Platform',
    version: '1.0.0',
    endpoints: {
      auth: '/api/auth',
      events: '/api/events', 
      registrations: '/api/registrations',
      payments: '/api/payments'
    },
    documentation: 'https://api.hesocial.com/docs'
  })
})

export default router