import { Request, Response } from 'express'
import bcrypt from 'bcryptjs'
import jwt from 'jsonwebtoken'
import { v4 as uuidv4 } from 'uuid'
import { pool } from '@/database/connection.js'
import { AuthenticatedRequest, User, JwtPayload } from '@/types/index.js'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

export const register = async (req: Request, res: Response): Promise<void> => {
  try {
    const {
      email,
      password,
      firstName,
      lastName,
      age,
      profession,
      annualIncome,
      netWorth,
      membershipTier,
      bio,
      interests
    } = req.body

    // Check if user already exists
    const existingUser = await pool.query(
      'SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL',
      [email]
    )

    if (existingUser.rows.length > 0) {
      res.status(409).json({
        success: false,
        error: 'User with this email already exists'
      })
      return
    }

    // Hash password
    const saltRounds = 12
    const passwordHash = await bcrypt.hash(password, saltRounds)

    // Create user
    const userId = uuidv4()
    const userQuery = `
      INSERT INTO users (
        id, email, password_hash, first_name, last_name, age, 
        profession, annual_income, net_worth, membership_tier, bio, interests
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
      RETURNING 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        bio, interests, created_at as "createdAt"
    `

    const result = await pool.query(userQuery, [
      userId, email, passwordHash, firstName, lastName, age,
      profession, annualIncome, netWorth, membershipTier, bio, interests
    ])

    const user = result.rows[0]

    // Log registration
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        userId,
        'user_registration',
        'user',
        userId,
        JSON.stringify({ membershipTier, registrationSource: 'web' }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    // Generate JWT token
    const token = jwt.sign(
      { userId: user.id, email: user.email, membershipTier: user.membershipTier },
      config.jwtSecret,
      { expiresIn: config.jwtExpiresIn }
    )

    logger.info('New user registered:', { userId: user.id, email: user.email, membershipTier })

    res.status(201).json({
      success: true,
      message: 'Registration successful. Please complete financial verification.',
      data: {
        user: {
          ...user,
          password_hash: undefined // Don't return password hash
        },
        token
      }
    })
  } catch (error) {
    logger.error('Registration error:', error)
    res.status(500).json({
      success: false,
      error: 'Registration failed'
    })
  }
}

export const login = async (req: Request, res: Response): Promise<void> => {
  try {
    const { email, password } = req.body

    // Get user with password hash
    const userQuery = `
      SELECT 
        id, email, password_hash, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE email = $1 AND deleted_at IS NULL
    `

    const result = await pool.query(userQuery, [email])

    if (result.rows.length === 0) {
      res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      })
      return
    }

    const user = result.rows[0]

    // Verify password
    const isValidPassword = await bcrypt.compare(password, user.password_hash)

    if (!isValidPassword) {
      res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      })
      return
    }

    // Generate JWT token
    const token = jwt.sign(
      { userId: user.id, email: user.email, membershipTier: user.membershipTier },
      config.jwtSecret,
      { expiresIn: config.jwtExpiresIn }
    )

    // Create session record
    const sessionId = uuidv4()
    const tokenHash = await bcrypt.hash(token, 10)
    const expiresAt = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000) // 7 days

    await pool.query(
      `INSERT INTO user_sessions (id, user_id, token_hash, expires_at, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6)`,
      [sessionId, user.id, tokenHash, expiresAt, req.ip, req.get('User-Agent')]
    )

    // Log login
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        user.id,
        'user_login',
        'user',
        user.id,
        JSON.stringify({ sessionId, loginSource: 'web' }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('User logged in:', { userId: user.id, email: user.email })

    res.json({
      success: true,
      message: 'Login successful',
      data: {
        user: {
          ...user,
          password_hash: undefined // Don't return password hash
        },
        token
      }
    })
  } catch (error) {
    logger.error('Login error:', error)
    res.status(500).json({
      success: false,
      error: 'Login failed'
    })
  }
}

export const logout = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    const authHeader = req.headers.authorization
    const token = authHeader && authHeader.split(' ')[1]

    if (token && req.user) {
      // Remove session
      const tokenHash = await bcrypt.hash(token, 10)
      await pool.query(
        'DELETE FROM user_sessions WHERE user_id = $1 AND token_hash = $2',
        [req.user.id, tokenHash]
      )

      // Log logout
      await pool.query(
        `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
         VALUES ($1, $2, $3, $4, $5, $6, $7)`,
        [
          req.user.id,
          'user_logout',
          'user',
          req.user.id,
          JSON.stringify({ logoutSource: 'web' }),
          req.ip,
          req.get('User-Agent')
        ]
      )

      logger.info('User logged out:', { userId: req.user.id, email: req.user.email })
    }

    res.json({
      success: true,
      message: 'Logout successful'
    })
  } catch (error) {
    logger.error('Logout error:', error)
    res.status(500).json({
      success: false,
      error: 'Logout failed'
    })
  }
}

export const getProfile = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    // Get fresh user data
    const userQuery = `
      SELECT 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE id = $1 AND deleted_at IS NULL
    `

    const result = await pool.query(userQuery, [req.user.id])

    if (result.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'User not found'
      })
      return
    }

    res.json({
      success: true,
      data: { user: result.rows[0] }
    })
  } catch (error) {
    logger.error('Get profile error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get profile'
    })
  }
}

export const updateProfile = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { firstName, lastName, profession, bio, interests, privacyLevel } = req.body

    const updateQuery = `
      UPDATE users SET
        first_name = COALESCE($2, first_name),
        last_name = COALESCE($3, last_name),
        profession = COALESCE($4, profession),
        bio = COALESCE($5, bio),
        interests = COALESCE($6, interests),
        privacy_level = COALESCE($7, privacy_level),
        updated_at = CURRENT_TIMESTAMP
      WHERE id = $1 AND deleted_at IS NULL
      RETURNING 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
    `

    const result = await pool.query(updateQuery, [
      req.user.id, firstName, lastName, profession, bio, interests, privacyLevel
    ])

    if (result.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'User not found'
      })
      return
    }

    // Log profile update
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'profile_update',
        'user',
        req.user.id,
        JSON.stringify({ updatedFields: Object.keys(req.body) }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Profile updated:', { userId: req.user.id, email: req.user.email })

    res.json({
      success: true,
      message: 'Profile updated successfully',
      data: { user: result.rows[0] }
    })
  } catch (error) {
    logger.error('Update profile error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to update profile'
    })
  }
}

export const refreshToken = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    // Generate new JWT token
    const token = jwt.sign(
      { userId: req.user.id, email: req.user.email, membershipTier: req.user.membershipTier },
      config.jwtSecret,
      { expiresIn: config.jwtExpiresIn }
    )

    res.json({
      success: true,
      message: 'Token refreshed successfully',
      data: { token }
    })
  } catch (error) {
    logger.error('Token refresh error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to refresh token'
    })
  }
}