import { Router } from 'express'
import {
  getEvents,
  getEventById,
  createEvent,
  getEventCategories,
  getVenues
} from '../controllers/eventController.js'
import { authenticateToken, requireMembership, requireVerification, optionalAuth } from '../middleware/auth.js'
import { validateRequest, eventCreationSchema, queryPaginationSchema } from '../middleware/validation.js'

const router = Router()

// Public routes (with optional authentication for personalization)
router.get('/', optionalAuth, validateRequest(queryPaginationSchema), getEvents)
router.get('/categories', getEventCategories)
router.get('/venues', getVenues)
router.get('/:id', optionalAuth, getEventById)

// Protected routes - require authentication and verification
router.post('/', 
  authenticateToken, 
  requireVerification,
  requireMembership(['Diamond', 'Black Card']), // Only Diamond+ can create events
  validateRequest(eventCreationSchema), 
  createEvent
)

export default router