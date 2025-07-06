import { Router } from 'express'
import { requireAuth, requireAdmin } from '../middleware/auth.js'
import {
  // Sales Leads
  getLeads,
  getLeadById,
  createLead,
  updateLead,
  deleteLead,
  
  // Sales Opportunities
  getOpportunities,
  createOpportunity,
  updateOpportunity,
  
  // Sales Activities
  getActivities,
  createActivity,
  
  // Sales Metrics
  getSalesMetrics,
  
  // Pipeline & Team
  getPipelineStages,
  getSalesTeam
} from '../controllers/salesController.js'

const router = Router()

// Apply authentication middleware to all routes
router.use(requireAuth)

// Sales Leads Routes
router.get('/leads', getLeads)
router.get('/leads/:id', getLeadById)
router.post('/leads', createLead)
router.put('/leads/:id', updateLead)
router.delete('/leads/:id', requireAdmin, deleteLead) // Only admins can delete

// Sales Opportunities Routes
router.get('/opportunities', getOpportunities)
router.post('/opportunities', createOpportunity)
router.put('/opportunities/:id', updateOpportunity)

// Sales Activities Routes
router.get('/activities', getActivities)
router.post('/activities', createActivity)

// Sales Metrics & Analytics Routes
router.get('/metrics', getSalesMetrics)

// Pipeline Management Routes
router.get('/pipeline/stages', getPipelineStages)

// Sales Team Routes
router.get('/team', getSalesTeam)

export default router