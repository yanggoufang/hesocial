import { Router } from 'express'
import {
  registerForEvent,
  getUserRegistrations,
  cancelRegistration
} from '@/controllers/registrationController.js'
import { authenticateToken, requireVerification } from '@/middleware/auth.js'
import { validateRequest, eventRegistrationSchema, queryPaginationSchema } from '@/middleware/validation.js'

const router = Router()

// All routes require authentication and verification
router.post('/', 
  authenticateToken, 
  requireVerification,
  validateRequest(eventRegistrationSchema), 
  registerForEvent
)

router.get('/', 
  authenticateToken, 
  validateRequest(queryPaginationSchema),
  getUserRegistrations
)

router.delete('/:registrationId', 
  authenticateToken, 
  cancelRegistration
)

export default router