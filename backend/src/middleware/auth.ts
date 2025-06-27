import { Request, Response, NextFunction } from 'express'
import jwt from 'jsonwebtoken'
import { duckdb } from '@/database/duckdb-connection'
import { AuthenticatedRequest, JwtPayload, User } from '@/types/index.js'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

export const authenticateToken = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const authReq = req as AuthenticatedRequest
    const authHeader = req.headers.authorization
    const token = authHeader && authHeader.split(' ')[1]

    if (!token) {
      res.status(401).json({
        success: false,
        error: 'Access token required'
      })
      return
    }

    const decoded = jwt.verify(token, config.jwtSecret) as JwtPayload
    
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
    
    const result = await duckdb.query(userQuery, [decoded.userId])
    
    if (result.rows.length === 0) {
      res.status(401).json({
        success: false,
        error: 'Invalid token - user not found'
      })
      return
    }

    authReq.user = result.rows[0] as User
    next()
  } catch (error) {
    logger.error('Authentication error:', error)
    
    if (error instanceof jwt.JsonWebTokenError) {
      res.status(401).json({
        success: false,
        error: 'Invalid token'
      })
      return
    }

    res.status(500).json({
      success: false,
      error: 'Authentication failed'
    })
  }
}

export const requireMembership = (requiredTiers: string[]) => {
  return (req: Request, res: Response, next: NextFunction): void => {
    const authReq = req as AuthenticatedRequest
    if (!authReq.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    if (!requiredTiers.includes(authReq.user.membershipTier)) {
      res.status(403).json({
        success: false,
        error: `Access denied. Required membership: ${requiredTiers.join(' or ')}`
      })
      return
    }

    next()
  }
}

export const requireVerification = (req: Request, res: Response, next: NextFunction): void => {
  const authReq = req as AuthenticatedRequest
  if (!authReq.user) {
    res.status(401).json({
      success: false,
      error: 'Authentication required'
    })
    return
  }

  if (!authReq.user.isVerified || authReq.user.verificationStatus !== 'approved') {
    res.status(403).json({
      success: false,
      error: 'Account verification required'
    })
    return
  }

  next()
}

export const optionalAuth = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const authReq = req as AuthenticatedRequest
    const authHeader = req.headers.authorization
    const token = authHeader && authHeader.split(' ')[1]

    if (!token) {
      next()
      return
    }

    const decoded = jwt.verify(token, config.jwtSecret) as JwtPayload
    
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
    
    const result = await duckdb.query(userQuery, [decoded.userId])
    
    if (result.rows.length > 0) {
      authReq.user = result.rows[0] as User
    }

    next()
  } catch (error) {
    logger.warn('Optional auth error:', error)
    next()
  }
}