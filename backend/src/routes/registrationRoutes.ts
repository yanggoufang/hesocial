import { Router } from 'express';
import { getUserRegistrations, registerForEvent, getRegistrationById, updateRegistration, cancelRegistration } from '../controllers/registrationController.js';
import { protect } from '../middleware/auth.js';

const router = Router();

// All routes in this file are protected
router.use(protect);

router.get('/user', getUserRegistrations as (req: Request, res: Response) => Promise<void>);
router.post('/', registerForEvent as (req: Request, res: Response) => Promise<void>);
router.get('/:id', getRegistrationById as (req: Request, res: Response) => Promise<void>);
router.put('/:id', updateRegistration as (req: Request, res: Response) => Promise<void>);
router.delete('/:id', cancelRegistration as (req: Request, res: Response) => Promise<void>);

export default router;
