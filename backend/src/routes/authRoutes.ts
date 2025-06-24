import { Router } from 'express'
import {
  register,
  login,
  logout,
  getProfile,
  updateProfile,
  refreshToken
} from '@/controllers/authController.js'
import { authenticateToken } from '@/middleware/auth.js'
import { validateRequest, userRegistrationSchema, userLoginSchema, profileUpdateSchema } from '@/middleware/validation.js'

const router = Router()

// Public routes
router.post('/register', validateRequest(userRegistrationSchema), register)
router.post('/login', validateRequest(userLoginSchema), login)

// Protected routes
router.post('/logout', authenticateToken, logout)
router.get('/profile', authenticateToken, getProfile)
router.put('/profile', authenticateToken, validateRequest(profileUpdateSchema), updateProfile)
router.post('/refresh', authenticateToken, refreshToken)

export default router