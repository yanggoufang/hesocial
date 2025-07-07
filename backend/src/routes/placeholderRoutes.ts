import { Router } from 'express';
import { getPlaceholderImage } from '../controllers/placeholderController';

const router = Router();

// Route for /api/placeholder/:width/:height
router.get('/:width/:height', getPlaceholderImage);

export default router;
