import { Router } from 'express'
import { authenticateToken as requireAuth, requireAdmin } from '../middleware/auth.js'
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
router.get('/leads', getLeads as any);
router.get('/leads/:id', getLeadById as any);
router.post('/leads', createLead as any);
router.put('/leads/:id', updateLead as any);
router.delete('/leads/:id', requireAdmin, deleteLead as any); // Only admins can delete

// Sales Opportunities Routes
router.get('/opportunities', getOpportunities as any);
router.post('/opportunities', createOpportunity as any);
router.put('/opportunities/:id', updateOpportunity as any);

// Sales Activities Routes
router.get('/activities', getActivities as any);
router.post('/activities', createActivity as any);

// Sales Metrics & Analytics Routes
router.get('/metrics', getSalesMetrics as any);

// Pipeline Management Routes
router.get('/pipeline/stages', getPipelineStages as any);

// Sales Team Routes
router.get('/team', getSalesTeam as any);

export default router
