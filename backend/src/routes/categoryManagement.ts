import { Router, Request, Response } from 'express'
import { authenticateToken, requireAdmin, requireSuperAdmin } from '../middleware/auth.js'
import { duckdb } from '../database/duckdb-connection.js'
import logger from '../utils/logger.js'

const router = Router()

/**
 * GET /api/categories
 * Get all event categories
 */
router.get('/', async (req, res) => {
  try {
    const user = (req as any).user
    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)

    let whereClause = ''
    let params: any[] = []

    // Only show active categories for non-admin users
    if (!isAdmin) {
      whereClause = 'WHERE is_active = true'
    }

    const categoriesQuery = `
      SELECT 
        id, name, slug, description, icon, color, 
        target_membership_tiers, typical_duration_hours, typical_capacity,
        is_active, sort_order, created_at, updated_at
      FROM event_categories
      ${whereClause}
      ORDER BY sort_order ASC, name ASC
    `
    
    const categoriesResult = await duckdb.query(categoriesQuery, params)

    return res.json({
      success: true,
      data: categoriesResult.rows
    })
  } catch (error) {
    logger.error('Failed to get event categories:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve event categories'
    })
  }
})

/**
 * GET /api/categories/:id
 * Get category by ID
 */
router.get('/:id', async (req, res) => {
  try {
    const { id } = req.params
    
    const categoryQuery = `
      SELECT *
      FROM event_categories
      WHERE id = ?
    `
    const categoryResult = await duckdb.query(categoryQuery, [id])
    const category = categoryResult.rows[0]

    if (!category) {
      return res.status(404).json({
        success: false,
        error: 'Category not found'
      })
    }

    // Check if category is active for non-admin users
    const user = (req as any).user
    const isAdmin = user && ['admin', 'super_admin'].includes(user.role)
    
    if (!isAdmin && !category.is_active) {
      return res.status(404).json({
        success: false,
        error: 'Category not found'
      })
    }

    return res.json({
      success: true,
      data: category
    })
  } catch (error) {
    logger.error('Failed to get category:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to retrieve category'
    })
  }
})

/**
 * POST /api/categories
 * Create new event category (Admin only)
 */
router.post('/', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const {
      name,
      slug,
      description,
      icon,
      color,
      targetMembershipTiers,
      typicalDurationHours,
      typicalCapacity,
      sortOrder
    } = req.body

    const user = (req as any).user
    const categoryId = Date.now()

    // Generate slug if not provided
    const categorySlug = slug || name.toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-')
      .trim('-')

    const insertQuery = `
      INSERT INTO event_categories (
        id, name, slug, description, icon, color,
        target_membership_tiers, typical_duration_hours, typical_capacity,
        is_active, sort_order, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `

    await duckdb.query(insertQuery, [
      categoryId, name, categorySlug, description, icon, color,
      JSON.stringify(targetMembershipTiers || []), typicalDurationHours || 3, 
      JSON.stringify(typicalCapacity || {'min': 8, 'max': 20}),
      true, sortOrder || 0,
      new Date().toISOString(), new Date().toISOString()
    ])

    logger.info(`Event category created: ${name} (ID: ${categoryId}) by admin: ${user.email}`)

    return res.status(201).json({
      success: true,
      message: 'Event category created successfully',
      data: { categoryId, slug: categorySlug }
    })
  } catch (error) {
    logger.error('Failed to create event category:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to create event category'
    })
  }
})

/**
 * PUT /api/categories/:id
 * Update event category (Admin only)
 */
router.put('/:id', authenticateToken, requireAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const updates = req.body
    const user = (req as any).user

    // Check if category exists
    const existingCategoryResult = await duckdb.query('SELECT id FROM event_categories WHERE id = ?', [id])
    if (!existingCategoryResult.rows[0]) {
      return res.status(404).json({
        success: false,
        error: 'Category not found'
      })
    }

    // Build update query dynamically
    const updateFields = []
    const params = []

    const allowedFields = [
      'name', 'slug', 'description', 'icon', 'color',
      'target_membership_tiers', 'typical_duration_hours', 'typical_capacity',
      'is_active', 'sort_order'
    ]

    for (const field of allowedFields) {
      if (updates[field] !== undefined) {
        updateFields.push(`${field} = ?`)
        
        // Handle JSON fields
        if (['target_membership_tiers', 'typical_capacity'].includes(field)) {
          params.push(JSON.stringify(updates[field]))
        } else {
          params.push(updates[field])
        }
      }
    }

    if (updateFields.length === 0) {
      return res.status(400).json({
        success: false,
        error: 'No valid fields to update'
      })
    }

    updateFields.push('updated_at = ?')
    params.push(new Date().toISOString())
    params.push(id)

    await duckdb.query(`
      UPDATE event_categories 
      SET ${updateFields.join(', ')} 
      WHERE id = ?
    `, params)

    logger.info(`Event category updated: ${id} by admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Event category updated successfully'
    })
  } catch (error) {
    logger.error('Failed to update event category:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update event category'
    })
  }
})

/**
 * DELETE /api/categories/:id
 * Delete event category (Super Admin only)
 */
router.delete('/:id', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    const { id } = req.params
    const user = (req as any).user

    // Check if category exists
    const existingCategoryResult = await duckdb.query('SELECT id, name FROM event_categories WHERE id = ?', [id])
    const existingCategory = existingCategoryResult.rows[0]
    
    if (!existingCategory) {
      return res.status(404).json({
        success: false,
        error: 'Category not found'
      })
    }

    // Check if category has events
    const eventsResult = await duckdb.query('SELECT COUNT(*) as count FROM events WHERE category_id = ?', [id])
    const eventCount = eventsResult.rows[0]?.count || 0

    if (eventCount > 0) {
      return res.status(400).json({
        success: false,
        error: 'Cannot delete category with associated events. Deactivate the category instead.'
      })
    }

    // Delete category
    await duckdb.query('DELETE FROM event_categories WHERE id = ?', [id])

    logger.warn(`Event category deleted: ${existingCategory.name} (ID: ${id}) by super admin: ${user.email}`)

    return res.json({
      success: true,
      message: 'Event category deleted successfully'
    })
  } catch (error) {
    logger.error('Failed to delete event category:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to delete event category'
    })
  }
})

export default router