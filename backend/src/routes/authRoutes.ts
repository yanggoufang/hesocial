import { Router } from 'express'
import jwt from 'jsonwebtoken'
import passport from '../config/passport.js'
import {
  register,
  login,
  getProfile,
  updateProfile,
  refreshToken
} from '../controllers/authController.js'
import { authenticateToken } from '../middleware/auth.js'
import { User } from '../types/index.js'
import config from '../utils/config.js'
import logger from '../utils/logger.js'

const router = Router()

// Initialize passport
router.use(passport.initialize())

// Regular authentication routes
router.post('/register', register)
router.post('/login', login)
router.get('/profile', authenticateToken, getProfile)
router.put('/profile', authenticateToken, updateProfile)
router.post('/refresh', authenticateToken, refreshToken)

// Google OAuth routes
router.get('/google',
  passport.authenticate('google', {
    scope: ['profile', 'email']
  })
)

router.get('/google/callback',
  passport.authenticate('google', { session: false }),
  async (req, res) => {
    try {
      const user = req.user as User
      
      if (!user) {
        logger.error('Google OAuth callback: No user data')
        return res.redirect(`${config.corsOrigins[0]}/login?error=oauth_failed`)
      }

      // Generate JWT token for the authenticated user
      const tokenPayload = {
        userId: user.id,
        email: user.email,
        membershipTier: user.membershipTier
      }
      const token = jwt.sign(tokenPayload, config.jwtSecret, { expiresIn: config.jwtExpiresIn } as jwt.SignOptions)

      logger.info(`Google OAuth successful for user: ${user.email}`)

      // Check if user needs to complete profile (for new Google users)
      const needsProfileCompletion = !user.age || !user.profession || !user.annualIncome || !user.netWorth

      // Redirect to frontend with token and status
      const redirectUrl = needsProfileCompletion 
        ? `${config.corsOrigins[0]}/complete-profile?token=${token}`
        : `${config.corsOrigins[0]}/dashboard?token=${token}`

      res.redirect(redirectUrl)
    } catch (error) {
      logger.error('Google OAuth callback error:', error)
      res.redirect(`${config.corsOrigins[0]}/login?error=oauth_error`)
    }
  }
)

// LinkedIn OAuth routes (placeholder for future implementation)
router.get('/linkedin', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'LinkedIn OAuth not implemented yet',
    message: 'LinkedIn OAuth will be added in next phase'
  })
})

router.get('/linkedin/callback', (req, res) => {
  res.status(501).json({
    success: false,
    error: 'LinkedIn OAuth not implemented yet',
    message: 'LinkedIn OAuth will be added in next phase'
  })
})

// Logout route (mainly for clearing any server-side sessions if needed)
router.post('/logout', authenticateToken, (req, res) => {
  try {
    // In a JWT-based system, logout is typically handled client-side
    // by removing the token from storage. This endpoint is for any
    // server-side cleanup if needed in the future.
    
    logger.info(`User logged out: ${(req as any).user?.email}`)
    
    res.json({
      success: true,
      message: 'Logged out successfully'
    })
  } catch (error) {
    logger.error('Logout error:', error)
    res.status(500).json({
      success: false,
      error: 'Logout failed'
    })
  }
})

// Validate token endpoint
router.get('/validate', authenticateToken, (req, res) => {
  res.json({
    success: true,
    data: {
      user: (req as any).user,
      valid: true
    },
    message: 'Token is valid'
  })
})

export default router