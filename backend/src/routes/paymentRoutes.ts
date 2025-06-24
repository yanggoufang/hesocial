import { Router } from 'express'
import {
  createPaymentIntent,
  confirmPayment,
  getPaymentMethods,
  createRefund,
  handleWebhook
} from '@/controllers/paymentController.js'
import { authenticateToken, requireVerification } from '@/middleware/auth.js'
import { validateRequest } from '@/middleware/validation.js'
import Joi from 'joi'

const router = Router()

// Validation schemas
const createPaymentIntentSchema = {
  body: Joi.object({
    registrationId: Joi.string().uuid().required(),
    paymentMethodId: Joi.string().required()
  })
}

const confirmPaymentSchema = {
  body: Joi.object({
    paymentIntentId: Joi.string().required()
  })
}

const createRefundSchema = {
  body: Joi.object({
    registrationId: Joi.string().uuid().required(),
    reason: Joi.string().max(500).optional()
  })
}

// Webhook route (no authentication required)
router.post('/webhook', handleWebhook)

// Protected routes
router.post('/intent', 
  authenticateToken, 
  requireVerification,
  validateRequest(createPaymentIntentSchema), 
  createPaymentIntent
)

router.post('/confirm', 
  authenticateToken, 
  requireVerification,
  validateRequest(confirmPaymentSchema), 
  confirmPayment
)

router.get('/methods', 
  authenticateToken, 
  getPaymentMethods
)

router.post('/refund', 
  authenticateToken, 
  requireVerification,
  validateRequest(createRefundSchema), 
  createRefund
)

export default router