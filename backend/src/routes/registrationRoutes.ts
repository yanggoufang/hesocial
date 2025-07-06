import { Router } from 'express'
import { authenticateToken as requireAuth, requireAdmin } from '../middleware/auth.js'
import {
  // User registration functions
  registerForEvent,
  getUserRegistrations,
  getRegistrationById,
  updateRegistration,
  cancelRegistration,
  
  // Admin registration functions
  getAllRegistrations,
  approveRegistration,
  updatePaymentStatus
} from '../controllers/registrationController.js'

const router = Router()

// User Registration Routes (require authentication)
router.use(requireAuth)

/**
 * POST /api/registrations/events/:eventId
 * Register for an event
 */
router.post('/events/:eventId', registerForEvent)

/**
 * GET /api/registrations/user
 * Get current user's registrations
 */
router.get('/user', getUserRegistrations)

/**
 * GET /api/registrations/:id
 * Get specific registration details
 */
router.get('/:id', getRegistrationById)

/**
 * PUT /api/registrations/:id
 * Update registration (special requests)
 */
router.put('/:id', updateRegistration)

/**
 * DELETE /api/registrations/:id
 * Cancel registration
 */
router.delete('/:id', cancelRegistration)

// Admin Registration Routes (require admin privileges)

/**
 * GET /api/registrations/admin/all
 * Get all registrations (admin only)
 */
router.get('/admin/all', requireAdmin, getAllRegistrations)

/**
 * POST /api/registrations/:id/approve
 * Approve or reject registration (admin only)
 */
router.post('/:id/approve', requireAdmin, approveRegistration)

/**
 * POST /api/registrations/:id/payment
 * Update payment status (admin only)
 */
router.post('/:id/payment', requireAdmin, updatePaymentStatus)

export default router