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
router.get('/leads', getLeads as (req: Request, res: Response) => Promise<void>);
router.get('/leads/:id', getLeadById as (req: Request, res: Response) => Promise<void>);
router.post('/leads', createLead as (req: Request, res: Response) => Promise<void>);
router.put('/leads/:id', updateLead as (req: Request, res: Response) => Promise<void>);
router.delete('/leads/:id', requireAdmin, deleteLead as (req: Request, res: Response) => Promise<void>); // Only admins can delete

// Sales Opportunities Routes
router.get('/opportunities', getOpportunities as (req: Request, res: Response) => Promise<void>);
router.post('/opportunities', createOpportunity as (req: Request, res: Response) => Promise<void>);
router.put('/opportunities/:id', updateOpportunity as (req: Request, res: Response) => Promise<void>);

// Sales Activities Routes
router.get('/activities', getActivities as (req: Request, res: Response) => Promise<void>);
router.post('/activities', createActivity as (req: Request, res: Response) => Promise<void>);

// Sales Metrics & Analytics Routes
router.get('/metrics', getSalesMetrics as (req: Request, res: Response) => Promise<void>);

// Pipeline Management Routes
router.get('/pipeline/stages', getPipelineStages as (req: Request, res: Response) => Promise<void>);

// Sales Team Routes
router.get('/team', getSalesTeam as (req: Request, res: Response) => Promise<void>);

export default router