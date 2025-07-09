import { Router } from 'express';
import { getUserRegistrations, registerForEvent, getRegistrationById, updateRegistration, cancelRegistration } from '../controllers/registrationController.js';
import { protect, optionalAuth } from '../middleware/auth.js';

const router = Router();

// =============================================================================
// PUBLIC ENDPOINTS - For browsing registration information
// =============================================================================

// Get registration statistics for an event (public - for display purposes)
router.get('/stats/:eventId', optionalAuth, async (req, res) => {
  try {
    const { pool } = await import('../database/duckdb-pool.js')
    const { eventId } = req.params
    
    const result = await pool.query(`
      SELECT 
        COUNT(*) as total_registrations,
        COUNT(CASE WHEN status = 'confirmed' THEN 1 END) as confirmed_registrations,
        COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_registrations,
        COUNT(CASE WHEN status = 'waitlist' THEN 1 END) as waitlist_registrations
      FROM event_registrations 
      WHERE event_id = ?
    `, [eventId])
    
    res.json({
      success: true,
      data: result.rows[0] || {
        total_registrations: 0,
        confirmed_registrations: 0,
        pending_registrations: 0,
        waitlist_registrations: 0
      }
    })
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error instanceof Error ? error.message : 'Failed to get registration stats'
    })
  }
});

// =============================================================================
// PROTECTED ENDPOINTS - Require authentication
// =============================================================================

// Get user's own registrations
router.get('/user', protect, getUserRegistrations as (req: Request, res: Response) => Promise<void>);

// Register for an event (requires authentication)
router.post('/', protect, registerForEvent as (req: Request, res: Response) => Promise<void>);

// Get specific registration details (user can only see their own)
router.get('/:id', protect, getRegistrationById as (req: Request, res: Response) => Promise<void>);

// Update registration (user can only update their own)
router.put('/:id', protect, updateRegistration as (req: Request, res: Response) => Promise<void>);

// Cancel registration (user can only cancel their own)
router.delete('/:id', protect, cancelRegistration as (req: Request, res: Response) => Promise<void>);

export default router;
