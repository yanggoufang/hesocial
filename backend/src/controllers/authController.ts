import { Request, Response } from 'express'
import bcrypt from 'bcryptjs'
import jwt from 'jsonwebtoken'
import { v4 as uuidv4 } from 'uuid'
import { duckdb } from '../database/duckdb-connection.js'
import { User, AuthenticatedRequest } from '../types/index.js'
import config from '../utils/config.js'
import logger from '../utils/logger.js'

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
      bio,
      interests
    } = req.body

    // Check if user already exists
    const existingUserQuery = 'SELECT id FROM users WHERE email = $1 AND deleted_at IS NULL'
    const existingUser = await duckdb.query(existingUserQuery, [email])

    if (existingUser.rows.length > 0) {
      res.status(400).json({
        success: false,
        error: 'User with this email already exists'
      })
      return
    }

    // Hash password
    const hashedPassword = await bcrypt.hash(password, 12)

    // Determine membership tier based on financial info
    let membershipTier: 'Platinum' | 'Diamond' | 'Black Card' = 'Platinum'
    if (netWorth >= 100000000 || annualIncome >= 20000000) { // 100M NTD net worth or 20M annual income
      membershipTier = 'Black Card'
    } else if (netWorth >= 30000000 || annualIncome >= 5000000) { // 30M NTD net worth or 5M annual income
      membershipTier = 'Diamond'
    }

    // Create user
    const userId = uuidv4()
    const insertUserQuery = `
      INSERT INTO users (
        id, email, password_hash, first_name, last_name, age, profession,
        annual_income, net_worth, membership_tier, privacy_level,
        is_verified, verification_status, bio, interests, created_at, updated_at
      ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
      )
    `

    await duckdb.query(insertUserQuery, [
      userId,
      email,
      hashedPassword,
      firstName,
      lastName,
      age,
      profession,
      annualIncome,
      netWorth,
      membershipTier,
      3, // default privacy level
      false, // needs verification
      'pending',
      bio || null,
      JSON.stringify(interests || []),
      new Date().toISOString(),
      new Date().toISOString()
    ])

    // Generate JWT token
    const tokenPayload = {
      userId,
      email,
      membershipTier
    }
    const token = jwt.sign(tokenPayload, config.jwtSecret as string, { expiresIn: config.jwtExpiresIn as string })

    // Get created user (without password)
    const userQuery = `
      SELECT 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        role, profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE id = $1
    `
    
    const userResult = await duckdb.query(userQuery, [userId])
    const user = userResult.rows[0] as User

    logger.info(`User registered successfully: ${email}`)

    res.status(201).json({
      success: true,
      data: {
        user,
        token
      },
      message: 'User registered successfully'
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

    // Get user with password
    const userQuery = `
      SELECT 
        id, email, password_hash, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        role, profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE email = $1 AND deleted_at IS NULL
    `
    
    const result = await duckdb.query(userQuery, [email])

    if (result.rows.length === 0) {
      res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      })
      return
    }

    const user = result.rows[0]

    // Check password
    const isValidPassword = await bcrypt.compare(password, user.password_hash)
    if (!isValidPassword) {
      res.status(401).json({
        success: false,
        error: 'Invalid email or password'
      })
      return
    }

    // Generate JWT token
    const tokenPayload = {
      userId: user.id,
      email: user.email,
      membershipTier: user.membershipTier
    }
    const token = jwt.sign(tokenPayload, config.jwtSecret as string, { expiresIn: config.jwtExpiresIn as string })

    // Remove password from response
    delete user.password_hash

    logger.info(`User logged in successfully: ${email}`)

    res.json({
      success: true,
      data: {
        user,
        token
      },
      message: 'Login successful'
    })
  } catch (error) {
    logger.error('Login error:', error)
    res.status(500).json({
      success: false,
      error: 'Login failed'
    })
  }
}

export const getProfile = async (req: Request, res: Response): Promise<void> => {
  try {
    const authReq = req as AuthenticatedRequest
    const user = authReq.user

    if (!user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    res.json({
      success: true,
      data: { user },
      message: 'Profile retrieved successfully'
    })
  } catch (error) {
    logger.error('Get profile error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get profile'
    })
  }
}

export const updateProfile = async (req: Request, res: Response): Promise<void> => {
  try {
    const authReq = req as AuthenticatedRequest
    const user = authReq.user

    if (!user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const {
      firstName,
      lastName,
      age,
      profession,
      bio,
      interests,
      privacyLevel
    } = req.body

    const updateQuery = `
      UPDATE users SET
        first_name = COALESCE($2, first_name),
        last_name = COALESCE($3, last_name),
        age = COALESCE($4, age),
        profession = COALESCE($5, profession),
        bio = COALESCE($6, bio),
        interests = COALESCE($7, interests),
        privacy_level = COALESCE($8, privacy_level),
        updated_at = $9
      WHERE id = $1
    `

    await duckdb.query(updateQuery, [
      user.id,
      firstName,
      lastName,
      age,
      profession,
      bio,
      interests ? JSON.stringify(interests) : null,
      privacyLevel,
      new Date().toISOString()
    ])

    // Get updated user
    const userQuery = `
      SELECT 
        id, email, first_name as "firstName", last_name as "lastName",
        age, profession, annual_income as "annualIncome", net_worth as "netWorth",
        membership_tier as "membershipTier", privacy_level as "privacyLevel",
        is_verified as "isVerified", verification_status as "verificationStatus",
        role, profile_picture as "profilePicture", bio, interests,
        created_at as "createdAt", updated_at as "updatedAt"
      FROM users 
      WHERE id = $1
    `
    
    const userResult = await duckdb.query(userQuery, [user.id])
    const updatedUser = userResult.rows[0] as User

    logger.info(`Profile updated successfully: ${user.email}`)

    res.json({
      success: true,
      data: { user: updatedUser },
      message: 'Profile updated successfully'
    })
  } catch (error) {
    logger.error('Update profile error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to update profile'
    })
  }
}

export const refreshToken = async (req: Request, res: Response): Promise<void> => {
  try {
    const authReq = req as AuthenticatedRequest
    const user = authReq.user

    if (!user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    // Generate new JWT token
    const tokenPayload = {
      userId: user.id,
      email: user.email,
      membershipTier: user.membershipTier
    }
    const token = jwt.sign(tokenPayload, config.jwtSecret as string, { expiresIn: config.jwtExpiresIn as string })

    res.json({
      success: true,
      data: { token },
      message: 'Token refreshed successfully'
    })
  } catch (error) {
    logger.error('Refresh token error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to refresh token'
    })
  }
}