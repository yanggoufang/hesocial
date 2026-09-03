import { Router, Request, Response } from 'express'
import { authenticateToken, requireAdmin, requireSuperAdmin } from '../middleware/auth.js'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'

const router = Router()

/**
 * GET /api/users
 * Get all users with pagination and filtering
 * Requires admin authentication
 */
router.get('/', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const page = parseInt(req.query.page as string) || 1
    const limit = parseInt(req.query.limit as string) || 20
    const search = req.query.search as string || ''
    const role = req.query.role as string || ''
    const membershipTier = req.query.membershipTier as string || ''
    const verificationStatus = req.query.verificationStatus as string || ''

    const offset = (page - 1) * limit

    let whereConditions = []
    let params: any[] = []

    if (search) {
      whereConditions.push(`(first_name ILIKE ? OR last_name ILIKE ? OR email ILIKE ?)`)
      params.push(`%${search}%`, `%${search}%`, `%${search}%`)
    }

    if (role) {
      whereConditions.push(`role = ?`)
      params.push(role)
    }

    if (membershipTier) {
      whereConditions.push(`membership_tier = ?`)
      params.push(membershipTier)
    }

    if (verificationStatus) {
      whereConditions.push(`verification_status = ?`)
      params.push(verificationStatus)
    }

    const whereClause = whereConditions.length > 0 ? `WHERE ${whereConditions.join(' AND ')}` : ''

    // Get total count
    const countQuery = `SELECT COUNT(*) as total FROM users ${whereClause}`
    const countResult = await duckdb.query(countQuery, params)
    const total = countResult.rows[0]?.total || 0

    // Get users
    const usersQuery = `
      SELECT 
        id, email, first_name, last_name, age, profession, 
        annual_income, net_worth, membership_tier, privacy_level,
        is_verified, verification_status, role, profile_picture, bio,
        interests, created_at, updated_at
      FROM users 
      ${whereClause}
      ORDER BY created_at DESC 
      LIMIT ? OFFSET ?
    `

    const usersResult = await duckdb.query(usersQuery, [...params, limit, offset])
    const users = usersResult.rows

    return res.json({
      success: true,
      data: users,
      pagination: {
        page,
        limit,
        total,
        totalPages: Math.ceil(total / limit)
      }
    })
  } catch (error) {
    logger.error('Failed to get users:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve users'
    })
  }
})

/**
 * GET /api/users/:id
 * Get user by ID
 * Requires admin authentication
 */
router.get('/:id', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params

    const userResult = await duckdb.query(`
      SELECT 
        id, email, first_name, last_name, age, profession, 
        annual_income, net_worth, membership_tier, privacy_level,
        is_verified, verification_status, role, profile_picture, bio,
        interests, created_at, updated_at
      FROM users 
      WHERE id = ?
    `, [id])
    const user = userResult.rows[0]

    if (!user) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    return res.json({
      success: true,
      data: user
    })
  } catch (error) {
    logger.error('Failed to get user:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve user'
    })
  }
})

/**
 * PUT /api/users/:id
 * Update user information
 * Requires admin authentication
 */
router.put('/:id', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const {
      firstName,
      lastName,
      age,
      profession,
      annualIncome,
      netWorth,
      membershipTier,
      privacyLevel,
      isVerified,
      verificationStatus,
      role,
      bio,
      interests
    } = req.body

    // Check if user exists
    const existingUserResult = await duckdb.query('SELECT id FROM users WHERE id = ?', [id])
    const existingUser = existingUserResult.rows[0]
    if (!existingUser) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    // Build update query dynamically
    const updates = []
    const params = []

    if (firstName !== undefined) {
      updates.push('first_name = ?')
      params.push(firstName)
    }
    if (lastName !== undefined) {
      updates.push('last_name = ?')
      params.push(lastName)
    }
    if (age !== undefined) {
      updates.push('age = ?')
      params.push(age)
    }
    if (profession !== undefined) {
      updates.push('profession = ?')
      params.push(profession)
    }
    if (annualIncome !== undefined) {
      updates.push('annual_income = ?')
      params.push(annualIncome)
    }
    if (netWorth !== undefined) {
      updates.push('net_worth = ?')
      params.push(netWorth)
    }
    if (membershipTier !== undefined) {
      updates.push('membership_tier = ?')
      params.push(membershipTier)
    }
    if (privacyLevel !== undefined) {
      updates.push('privacy_level = ?')
      params.push(privacyLevel)
    }
    if (isVerified !== undefined) {
      updates.push('is_verified = ?')
      params.push(isVerified)
    }
    if (verificationStatus !== undefined) {
      updates.push('verification_status = ?')
      params.push(verificationStatus)
    }
    if (role !== undefined) {
      updates.push('role = ?')
      params.push(role)
    }
    if (bio !== undefined) {
      updates.push('bio = ?')
      params.push(bio)
    }
    if (interests !== undefined) {
      updates.push('interests = ?')
      params.push(JSON.stringify(interests))
    }

    if (updates.length === 0) {
      return res.status(400).json({
        success: false,
        error: 'No valid fields to update'
      })
    }

    updates.push('updated_at = ?')
    params.push(new Date().toISOString())
    params.push(id)

    await duckdb.query(`
      UPDATE users 
      SET ${updates.join(', ')} 
      WHERE id = ?
    `, params)

    logger.info(`User updated by admin: ${id}`)

    return res.json({
      success: true,
      message: 'User updated successfully'
    })
  } catch (error) {
    logger.error('Failed to update user:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update user'
    })
  }
})

/**
 * DELETE /api/users/:id
 * Delete user (soft delete by setting a deleted flag or hard delete)
 * Requires super admin authentication
 */
router.delete('/:id', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    const { id } = req.params

    // Check if user exists
    const existingUserResult = await duckdb.query('SELECT id, email FROM users WHERE id = ?', [id])
    const existingUser = existingUserResult.rows[0]
    if (!existingUser) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    // Hard delete (you might want to implement soft delete instead)
    await duckdb.query('DELETE FROM users WHERE id = ?', [id])

    logger.warn(`User deleted by super admin: ${existingUser.email} (ID: ${id})`)

    return res.json({
      success: true,
      message: 'User deleted successfully'
    })
  } catch (error) {
    logger.error('Failed to delete user:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to delete user'
    })
  }
})

/**
 * POST /api/users/:id/verify
 * Verify user account
 * Requires admin authentication
 */
router.post('/:id/verify', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const { status } = req.body // 'approved' or 'rejected'

    if (!['approved', 'rejected'].includes(status)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid verification status'
      })
    }

    const result = await duckdb.query(`
      UPDATE users 
      SET verification_status = ?, is_verified = ?, updated_at = ?
      WHERE id = ?
    `, [status, status === 'approved', new Date().toISOString(), id])

    if (result.changes === 0) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    logger.info(`User verification updated: ${id} -> ${status}`)

    return res.json({
      success: true,
      message: `User ${status === 'approved' ? 'verified' : 'rejected'} successfully`
    })
  } catch (error) {
    logger.error('Failed to verify user:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to verify user'
    })
  }
})

/**
 * POST /api/users/:id/role
 * Update user role
 * Requires super admin authentication
 */
router.post('/:id/role', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const { role } = req.body

    if (!['user', 'admin', 'super_admin'].includes(role)) {
      return res.status(400).json({
        success: false,
        error: 'Invalid role'
      })
    }

    const result = await duckdb.query(`
      UPDATE users 
      SET role = ?, updated_at = ?
      WHERE id = ?
    `, [role, new Date().toISOString(), id])

    if (result.changes === 0) {
      return res.status(404).json({
        success: false,
        error: 'User not found'
      })
    }

    logger.info(`User role updated: ${id} -> ${role}`)

    return res.json({
      success: true,
      message: 'User role updated successfully'
    })
  } catch (error) {
    logger.error('Failed to update user role:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update user role'
    })
  }
})

/**
 * GET /api/users/stats
 * Get user statistics
 * Requires admin authentication
 */
router.get('/stats/overview', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const stats = await Promise.all([
      // Total users
      duckdb.query('SELECT COUNT(*) as total FROM users'),
      // Users by role
      duckdb.query('SELECT role, COUNT(*) as count FROM users GROUP BY role'),
      // Users by membership tier
      duckdb.query('SELECT membership_tier, COUNT(*) as count FROM users GROUP BY membership_tier'),
      // Users by verification status
      duckdb.query('SELECT verification_status, COUNT(*) as count FROM users GROUP BY verification_status'),
      // Recent registrations (last 30 days)
      duckdb.query(`
        SELECT COUNT(*) as recent 
        FROM users 
        WHERE created_at >= date('now', '-30 days')
      `)
    ])

    return res.json({
      success: true,
      data: {
        totalUsers: stats[0]?.rows[0]?.total || 0,
        usersByRole: stats[1]?.rows || [],
        usersByMembershipTier: stats[2]?.rows || [],
        usersByVerificationStatus: stats[3]?.rows || [],
        recentRegistrations: stats[4]?.rows[0]?.recent || 0
      }
    })
  } catch (error) {
    logger.error('Failed to get user stats:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve user statistics'
    })
  }
})

export default router