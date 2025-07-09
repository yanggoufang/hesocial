import { Request, Response } from 'express'
import { pool } from '../database/duckdb-pool.js'
import { 
  SalesLead, 
  SalesOpportunity, 
  SalesActivity, 
  SalesMetrics, 
  SalesLeadFilters, 
  SalesOpportunityFilters,
  ApiResponse,
  AuthenticatedRequest
} from '../types/index.js'
import logger from '../utils/logger.js'

// Sales Leads Management
export const getLeads = async (req: Request, res: Response): Promise<Response> => {
  try {
    const { 
      page = 1, 
      limit = 20, 
      status, 
      source, 
      assignedTo, 
      membershipTier, 
      search 
    } = req.query

    let whereClause = '1=1'
    const params: any[] = []

    if (status) {
      whereClause += ' AND status = ?'
      params.push(status)
    }

    if (source) {
      whereClause += ' AND source = ?'
      params.push(source)
    }

    if (assignedTo) {
      whereClause += ' AND assigned_to = ?'
      params.push(assignedTo)
    }

    if (membershipTier) {
      whereClause += ' AND interested_membership_tier = ?'
      params.push(membershipTier)
    }

    if (search) {
      whereClause += ' AND (first_name ILIKE ? OR last_name ILIKE ? OR email ILIKE ? OR company ILIKE ?)'
      const searchTerm = `%${search}%`
      params.push(searchTerm, searchTerm, searchTerm, searchTerm)
    }

    const offset = (Number(page) - 1) * Number(limit)
    
    // Get total count
    const countQuery = `SELECT COUNT(*) as total FROM sales_leads WHERE ${whereClause}`
    const countResult = await pool.query(countQuery, params)
    const total = Number(countResult.rows[0]?.total) || 0

    // Get leads data
    const dataQuery = `
      SELECT 
        l.*,
        u.first_name as assigned_to_first_name,
        u.last_name as assigned_to_last_name
      FROM sales_leads l
      LEFT JOIN users u ON l.assigned_to = u.id
      WHERE ${whereClause}
      ORDER BY l.created_at DESC
      LIMIT ? OFFSET ?
    `
    const dataResult = await pool.query(dataQuery, [...params, limit, offset])

    const response: ApiResponse<SalesLead[]> = {
      success: true,
      data: dataResult.rows,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages: Math.ceil(total / Number(limit))
      }
    }

    return res.json(response)
  } catch (error) {
    logger.error('Error fetching sales leads:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch sales leads'
    });
  }
}

export const getLeadById = async (req: Request, res: Response): Promise<Response> => {
  try {
    const { id } = req.params
    
    const query = `
      SELECT 
        l.*,
        u.first_name as assigned_to_first_name,
        u.last_name as assigned_to_last_name
      FROM sales_leads l
      LEFT JOIN users u ON l.assigned_to = u.id
      WHERE l.id = ?
    `
    const result = await pool.query(query, [id])
    
    if (result.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Lead not found'
      })
    }

    return res.json({
      success: true,
      data: result.rows[0]
    });
  } catch (error) {
    logger.error('Error fetching lead by ID:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch lead'
    });
  }
}

export const createLead = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const {
      firstName,
      lastName,
      email,
      phone,
      company,
      jobTitle,
      annualIncome,
      netWorth,
      source,
      referralCode,
      interestedMembershipTier,
      budgetRange,
      timeline,
      painPoints,
      interests,
      notes,
      assignedTo
    } = req.body

    // Calculate lead score based on income and net worth
    let leadScore = 0
    if (annualIncome && annualIncome >= 20000000) leadScore += 40 // 20M+ TWD
    if (netWorth && netWorth >= 100000000) leadScore += 40 // 100M+ TWD
    if (interestedMembershipTier === 'Black Card') leadScore += 20
    else if (interestedMembershipTier === 'Diamond') leadScore += 15
    else if (interestedMembershipTier === 'Platinum') leadScore += 10

    const query = `
      INSERT INTO sales_leads (
        first_name, last_name, email, phone, company, job_title,
        annual_income, net_worth, source, referral_code, lead_score,
        interested_membership_tier, budget_range, timeline, pain_points,
        interests, notes, assigned_to
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      RETURNING *
    `
    
    const result = await pool.query(query, [
      firstName, lastName, email, phone, company, jobTitle,
      annualIncome, netWorth, source, referralCode, leadScore,
      interestedMembershipTier, budgetRange, timeline, painPoints,
      JSON.stringify(interests || []), notes, assignedTo
    ])

    res.status(201).json({
      success: true,
      data: result.rows[0],
      message: 'Lead created successfully'
    });
  } catch (error) {
    logger.error('Error creating lead:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to create lead'
    });
  }
}

export const updateLead = async (req: AuthenticatedRequest, res: Response): Promise<Response> => {
  try {
    const { id } = req.params
    const updates = req.body
    
    // Build dynamic update query
    const updateFields: string[] = []
    const updateValues: any[] = []
    
    Object.keys(updates).forEach(key => {
      if (updates[key] !== undefined) {
        updateFields.push(`${key} = ?`)
        updateValues.push(updates[key])
      }
    })
    
    updateFields.push('updated_at = CURRENT_TIMESTAMP')
    updateValues.push(id)

    const query = `
      UPDATE sales_leads 
      SET ${updateFields.join(', ')}
      WHERE id = ?
      RETURNING *
    `
    
    const result = await pool.query(query, updateValues)
    
    if (result.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Lead not found'
      })
    }

    return res.json({
      success: true,
      data: result.rows[0],
      message: 'Lead updated successfully'
    });
  } catch (error) {
    logger.error('Error updating lead:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update lead'
    });
  }
}

export const deleteLead = async (req: Request, res: Response): Promise<Response> => {
  try {
    const { id } = req.params
    
    const query = 'DELETE FROM sales_leads WHERE id = ?'
    const result = await pool.query(query, [id])
    
    if (result.rowCount === 0) {
      return res.status(404).json({
        success: false,
        error: 'Lead not found'
      })
    }

    return res.json({
      success: true,
      message: 'Lead deleted successfully'
    });
  } catch (error) {
    logger.error('Error deleting lead:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to delete lead'
    });
  }
}

// Sales Opportunities Management
export const getOpportunities = async (req: Request, res: Response) => {
  try {
    const { 
      page = 1, 
      limit = 20, 
      stage, 
      assignedTo, 
      membershipTier 
    } = req.query

    let whereClause = '1=1'
    const params: any[] = []

    if (stage) {
      whereClause += ' AND stage = ?'
      params.push(stage)
    }

    if (assignedTo) {
      whereClause += ' AND assigned_to = ?'
      params.push(assignedTo)
    }

    if (membershipTier) {
      whereClause += ' AND membership_tier = ?'
      params.push(membershipTier)
    }

    const offset = (Number(page) - 1) * Number(limit)
    
    // Get total count
    const countQuery = `SELECT COUNT(*) as total FROM sales_opportunities WHERE ${whereClause}`
    const countResult = await pool.query(countQuery, params)
    const total = Number(countResult.rows[0]?.total) || 0

    // Get opportunities data
    const dataQuery = `
      SELECT 
        o.*,
        l.first_name as lead_first_name,
        l.last_name as lead_last_name,
        l.email as lead_email,
        u.first_name as assigned_to_first_name,
        u.last_name as assigned_to_last_name
      FROM sales_opportunities o
      LEFT JOIN sales_leads l ON o.lead_id = l.id
      LEFT JOIN users u ON o.assigned_to = u.id
      WHERE ${whereClause}
      ORDER BY o.created_at DESC
      LIMIT ? OFFSET ?
    `
    const dataResult = await pool.query(dataQuery, [...params, limit, offset])

    const response: ApiResponse<SalesOpportunity[]> = {
      success: true,
      data: dataResult.rows,
      pagination: {
        page: Number(page),
        limit: Number(limit),
        total,
        totalPages: Math.ceil(total / Number(limit))
      }
    }

    res.json(response);
  } catch (error) {
    logger.error('Error fetching sales opportunities:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch sales opportunities'
    });
  }
}

export const createOpportunity = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const {
      leadId,
      name,
      description,
      stage,
      probability,
      value,
      expectedCloseDate,
      membershipTier,
      paymentTerms,
      assignedTo
    } = req.body

    const query = `
      INSERT INTO sales_opportunities (
        lead_id, name, description, stage, probability, value,
        expected_close_date, membership_tier, payment_terms, assigned_to
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      RETURNING *
    `
    
    const result = await pool.query(query, [
      leadId, name, description, stage, probability, value,
      expectedCloseDate, membershipTier, paymentTerms, assignedTo
    ])

    res.status(201).json({
      success: true,
      data: result.rows[0],
      message: 'Opportunity created successfully'
    });
  } catch (error) {
    logger.error('Error creating opportunity:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to create opportunity'
    });
  }
}

export const updateOpportunity = async (req: AuthenticatedRequest, res: Response): Promise<Response> => {
  try {
    const { id } = req.params
    const updates = req.body
    
    // Build dynamic update query
    const updateFields: string[] = []
    const updateValues: any[] = []
    
    Object.keys(updates).forEach(key => {
      if (updates[key] !== undefined) {
        updateFields.push(`${key} = ?`)
        updateValues.push(updates[key])
      }
    })
    
    updateFields.push('updated_at = CURRENT_TIMESTAMP')
    updateValues.push(id)

    const query = `
      UPDATE sales_opportunities 
      SET ${updateFields.join(', ')}
      WHERE id = ?
      RETURNING *
    `
    
    const result = await pool.query(query, updateValues)
    
    if (result.rows.length === 0) {
      return res.status(404).json({
        success: false,
        error: 'Opportunity not found'
      })
    }

    return res.json({
      success: true,
      data: result.rows[0],
      message: 'Opportunity updated successfully'
    });
  } catch (error) {
    logger.error('Error updating opportunity:', error)
    return res.status(500).json({
      success: false,
      error: 'Failed to update opportunity'
    });
  }
}

// Sales Activities Management
export const getActivities = async (req: Request, res: Response) => {
  try {
    const { leadId, opportunityId, page = 1, limit = 20 } = req.query

    let whereClause = '1=1'
    const params: any[] = []

    if (leadId) {
      whereClause += ' AND lead_id = ?'
      params.push(leadId)
    }

    if (opportunityId) {
      whereClause += ' AND opportunity_id = ?'
      params.push(opportunityId)
    }

    const offset = (Number(page) - 1) * Number(limit)
    
    const query = `
      SELECT 
        a.*,
        u.first_name as created_by_first_name,
        u.last_name as created_by_last_name
      FROM sales_activities a
      LEFT JOIN users u ON a.created_by = u.id
      WHERE ${whereClause}
      ORDER BY a.created_at DESC
      LIMIT ? OFFSET ?
    `
    
    const result = await pool.query(query, [...params, limit, offset])

    res.json({
      success: true,
      data: result.rows
    });
  } catch (error) {
    logger.error('Error fetching sales activities:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch sales activities'
    });
  }
}

export const createActivity = async (req: AuthenticatedRequest, res: Response) => {
  try {
    const {
      leadId,
      opportunityId,
      activityType,
      subject,
      description,
      outcome,
      durationMinutes,
      scheduledAt,
      completedAt
    } = req.body

    const query = `
      INSERT INTO sales_activities (
        lead_id, opportunity_id, activity_type, subject, description,
        outcome, duration_minutes, scheduled_at, completed_at, created_by
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      RETURNING *
    `
    
    const result = await pool.query(query, [
      leadId, opportunityId, activityType, subject, description,
      outcome, durationMinutes, scheduledAt, completedAt, req.user?.id
    ])

    res.status(201).json({
      success: true,
      data: result.rows[0],
      message: 'Activity created successfully'
    });
  } catch (error) {
    logger.error('Error creating activity:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to create activity'
    });
  }
}

// Sales Metrics and Analytics
export const getSalesMetrics = async (req: Request, res: Response) => {
  try {
    const { period = 'monthly', salesRepId } = req.query

    let dateFilter = ''
    if (period === 'monthly') {
      dateFilter = "WHERE created_at >= DATE_TRUNC('month', CURRENT_DATE)"
    } else if (period === 'quarterly') {
      dateFilter = "WHERE created_at >= DATE_TRUNC('quarter', CURRENT_DATE)"
    } else if (period === 'yearly') {
      dateFilter = "WHERE created_at >= DATE_TRUNC('year', CURRENT_DATE)"
    }

    let salesRepFilter = ''
    if (salesRepId) {
      salesRepFilter = salesRepId ? `AND assigned_to = ${salesRepId}` : ''
    }

    // Get comprehensive metrics
    const metricsQuery = `
      WITH lead_metrics AS (
        SELECT 
          COUNT(*) as total_leads,
          COUNT(CASE WHEN status IN ('qualified', 'contacted', 'nurturing') THEN 1 END) as qualified_leads,
          COUNT(CASE WHEN status = 'closed_won' THEN 1 END) as converted_leads
        FROM sales_leads 
        ${dateFilter} ${salesRepFilter}
      ),
      opportunity_metrics AS (
        SELECT 
          COUNT(*) as total_opportunities,
          SUM(value) as total_pipeline_value,
          AVG(value) as average_deal_size,
          COUNT(CASE WHEN stage = 'closed_won' THEN 1 END) as won_opportunities,
          SUM(CASE WHEN stage = 'closed_won' THEN value ELSE 0 END) as won_revenue
        FROM sales_opportunities 
        ${dateFilter} ${salesRepFilter}
      )
      SELECT 
        l.total_leads,
        l.qualified_leads,
        l.converted_leads,
        o.total_opportunities,
        o.total_pipeline_value,
        o.average_deal_size,
        o.won_opportunities,
        o.won_revenue,
        CASE 
          WHEN l.total_leads > 0 THEN (l.converted_leads * 100.0 / l.total_leads)
          ELSE 0
        END as conversion_rate,
        CASE 
          WHEN o.total_opportunities > 0 THEN (o.won_opportunities * 100.0 / o.total_opportunities)
          ELSE 0
        END as win_rate
      FROM lead_metrics l
      CROSS JOIN opportunity_metrics o
    `

    const result = await pool.query(metricsQuery)
    const metrics = result.rows[0] || {}

    const response: ApiResponse<SalesMetrics> = {
      success: true,
      data: {
        totalLeads: Number(metrics.total_leads) || 0,
        qualifiedLeads: Number(metrics.qualified_leads) || 0,
        totalOpportunities: Number(metrics.total_opportunities) || 0,
        totalPipelineValue: Number(metrics.total_pipeline_value) || 0,
        conversionRate: Number(metrics.conversion_rate) || 0,
        averageDealSize: Number(metrics.average_deal_size) || 0,
        salesCycleLength: 30, // Default - would be calculated from actual data
        winRate: Number(metrics.win_rate) || 0,
        monthlyRevenue: Number(metrics.won_revenue) || 0,
        quarterlyRevenue: Number(metrics.won_revenue) || 0,
        yearlyRevenue: Number(metrics.won_revenue) || 0
      }
    }

    res.json(response);
  } catch (error) {
    logger.error('Error fetching sales metrics:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch sales metrics'
    });
  }
}

// Pipeline Management
export const getPipelineStages = async (req: Request, res: Response) => {
  try {
    const query = `
      SELECT * FROM sales_pipeline_stages 
      WHERE is_active = true 
      ORDER BY display_order
    `
    const result = await pool.query(query)

    res.json({
      success: true,
      data: result.rows
    });
  } catch (error) {
    logger.error('Error fetching pipeline stages:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch pipeline stages'
    });
  }
}

export const getSalesTeam = async (req: Request, res: Response) => {
  try {
    const query = `
      SELECT 
        st.*,
        u.first_name,
        u.last_name,
        u.email,
        m.first_name as manager_first_name,
        m.last_name as manager_last_name
      FROM sales_team_members st
      LEFT JOIN users u ON st.user_id = u.id
      LEFT JOIN users m ON st.manager_id = m.id
      WHERE st.is_active = true
      ORDER BY st.role, u.first_name
    `
    const result = await pool.query(query)

    res.json({
      success: true,
      data: result.rows
    });
  } catch (error) {
    logger.error('Error fetching sales team:', error);
    return res.status(500).json({
      success: false,
      error: 'Failed to fetch sales team'
    });
  }
}