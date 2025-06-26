import { Response } from 'express'
import Stripe from 'stripe'
import { pool } from '@/database/connection.js'
import { AuthenticatedRequest } from '@/types/index.js'
import config from '@/utils/config.js'
import logger from '@/utils/logger.js'

const stripe = new Stripe(config.stripe.secretKey, {
  apiVersion: '2023-08-16'
})

export const createPaymentIntent = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { registrationId, paymentMethodId } = req.body

    // Get registration details
    const registrationQuery = `
      SELECT 
        r.id, r.user_id, r.event_id, r.status, r.payment_status,
        e.name as event_name, e.pricing, e.exclusivity_level,
        u.first_name, u.last_name, u.email
      FROM registrations r
      JOIN events e ON r.event_id = e.id
      JOIN users u ON r.user_id = u.id
      WHERE r.id = $1 AND r.user_id = $2
    `

    const registrationResult = await pool.query(registrationQuery, [registrationId, req.user.id])

    if (registrationResult.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
      return
    }

    const registration = registrationResult.rows[0]

    if (registration.payment_status === 'paid') {
      res.status(400).json({
        success: false,
        error: 'Registration already paid'
      })
      return
    }

    // Calculate amount based on user's membership tier and event exclusivity
    const pricing = registration.pricing
    let amount = 0

    if (req.user.membershipTier === 'Black Card' && pricing.vvip) {
      amount = pricing.vvip
    } else if (req.user.membershipTier === 'Diamond' && pricing.vvip) {
      amount = pricing.vvip
    } else if (pricing.vip) {
      amount = pricing.vip
    } else if (pricing.general) {
      amount = pricing.general
    } else {
      res.status(400).json({
        success: false,
        error: 'No pricing available for your membership tier'
      })
      return
    }

    // Create Stripe customer if not exists
    let stripeCustomerId: string

    const existingCustomerQuery = `
      SELECT stripe_customer_id FROM users WHERE id = $1
    `
    const customerResult = await pool.query(existingCustomerQuery, [req.user.id])

    if (customerResult.rows[0]?.stripe_customer_id) {
      stripeCustomerId = customerResult.rows[0].stripe_customer_id
    } else {
      const customer = await stripe.customers.create({
        email: req.user.email,
        name: `${registration.first_name} ${registration.last_name}`,
        metadata: {
          userId: req.user.id,
          membershipTier: req.user.membershipTier
        }
      })

      stripeCustomerId = customer.id

      // Save customer ID to database
      await pool.query(
        'UPDATE users SET stripe_customer_id = $1 WHERE id = $2',
        [stripeCustomerId, req.user.id]
      )
    }

    // Create payment intent
    const paymentIntent = await stripe.paymentIntents.create({
      amount: amount * 100, // Convert to cents
      currency: pricing.currency?.toLowerCase() || 'twd',
      customer: stripeCustomerId,
      payment_method: paymentMethodId,
      confirmation_method: 'manual',
      confirm: true,
      return_url: `${config.corsOrigins[0]}/events/${registration.event_id}`,
      metadata: {
        registrationId: registration.id,
        eventId: registration.event_id,
        userId: req.user.id,
        eventName: registration.event_name,
        membershipTier: req.user.membershipTier
      },
      description: `Event Registration: ${registration.event_name}`
    })

    // Update registration with payment intent ID
    await pool.query(
      'UPDATE registrations SET payment_intent_id = $1 WHERE id = $2',
      [paymentIntent.id, registrationId]
    )

    // Log payment attempt
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'payment_intent_created',
        'payment',
        paymentIntent.id,
        JSON.stringify({
          registrationId,
          eventName: registration.event_name,
          amount,
          currency: pricing.currency || 'TWD'
        }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Payment intent created:', {
      paymentIntentId: paymentIntent.id,
      userId: req.user.id,
      registrationId,
      amount
    })

    res.json({
      success: true,
      data: {
        paymentIntentId: paymentIntent.id,
        clientSecret: paymentIntent.client_secret,
        status: paymentIntent.status,
        amount,
        currency: pricing.currency || 'TWD'
      }
    })
  } catch (error) {
    logger.error('Create payment intent error:', error)
    
    if (error instanceof Stripe.errors.StripeError) {
      res.status(400).json({
        success: false,
        error: error.message
      })
      return
    }

    res.status(500).json({
      success: false,
      error: 'Failed to create payment intent'
    })
  }
}

export const confirmPayment = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { paymentIntentId } = req.body

    // Retrieve payment intent
    const paymentIntent = await stripe.paymentIntents.retrieve(paymentIntentId)

    if (paymentIntent.metadata.userId !== req.user.id) {
      res.status(403).json({
        success: false,
        error: 'Unauthorized access to payment'
      })
      return
    }

    // Confirm payment if required
    let confirmedPaymentIntent = paymentIntent
    if (paymentIntent.status === 'requires_confirmation') {
      confirmedPaymentIntent = await stripe.paymentIntents.confirm(paymentIntentId)
    }

    // Update registration status based on payment status
    if (confirmedPaymentIntent.status === 'succeeded') {
      await pool.query(
        `UPDATE registrations 
         SET payment_status = 'paid', status = 'approved', updated_at = CURRENT_TIMESTAMP 
         WHERE payment_intent_id = $1`,
        [paymentIntentId]
      )

      // Log successful payment
      await pool.query(
        `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
         VALUES ($1, $2, $3, $4, $5, $6, $7)`,
        [
          req.user.id,
          'payment_succeeded',
          'payment',
          paymentIntentId,
          JSON.stringify({
            registrationId: paymentIntent.metadata.registrationId,
            eventName: paymentIntent.metadata.eventName,
            amount: paymentIntent.amount / 100
          }),
          req.ip,
          req.get('User-Agent')
        ]
      )

      logger.info('Payment succeeded:', {
        paymentIntentId,
        userId: req.user.id,
        registrationId: paymentIntent.metadata.registrationId
      })
    }

    res.json({
      success: true,
      data: {
        paymentIntentId: confirmedPaymentIntent.id,
        status: confirmedPaymentIntent.status,
        message: confirmedPaymentIntent.status === 'succeeded' 
          ? 'Payment successful' 
          : 'Payment requires additional action'
      }
    })
  } catch (error) {
    logger.error('Confirm payment error:', error)
    
    if (error instanceof Stripe.errors.StripeError) {
      res.status(400).json({
        success: false,
        error: error.message
      })
      return
    }

    res.status(500).json({
      success: false,
      error: 'Failed to confirm payment'
    })
  }
}

export const getPaymentMethods = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    // Get user's Stripe customer ID
    const customerQuery = `SELECT stripe_customer_id FROM users WHERE id = $1`
    const result = await pool.query(customerQuery, [req.user.id])

    if (!result.rows[0]?.stripe_customer_id) {
      res.json({
        success: true,
        data: []
      })
      return
    }

    const stripeCustomerId = result.rows[0].stripe_customer_id

    // Get payment methods from Stripe
    const paymentMethods = await stripe.paymentMethods.list({
      customer: stripeCustomerId,
      type: 'card'
    })

    const formattedPaymentMethods = paymentMethods.data.map(pm => ({
      id: pm.id,
      type: pm.type,
      card: pm.card ? {
        brand: pm.card.brand,
        last4: pm.card.last4,
        expMonth: pm.card.exp_month,
        expYear: pm.card.exp_year
      } : null,
      created: pm.created
    }))

    res.json({
      success: true,
      data: formattedPaymentMethods
    })
  } catch (error) {
    logger.error('Get payment methods error:', error)
    res.status(500).json({
      success: false,
      error: 'Failed to get payment methods'
    })
  }
}

export const createRefund = async (req: AuthenticatedRequest, res: Response): Promise<void> => {
  try {
    if (!req.user) {
      res.status(401).json({
        success: false,
        error: 'Authentication required'
      })
      return
    }

    const { registrationId, reason } = req.body

    // Get registration and payment details
    const registrationQuery = `
      SELECT 
        r.id, r.user_id, r.payment_intent_id, r.payment_status,
        e.name as event_name, e.date_time
      FROM registrations r
      JOIN events e ON r.event_id = e.id
      WHERE r.id = $1 AND r.user_id = $2
    `

    const result = await pool.query(registrationQuery, [registrationId, req.user.id])

    if (result.rows.length === 0) {
      res.status(404).json({
        success: false,
        error: 'Registration not found'
      })
      return
    }

    const registration = result.rows[0]

    if (registration.payment_status !== 'paid') {
      res.status(400).json({
        success: false,
        error: 'Registration not paid, cannot refund'
      })
      return
    }

    // Check if event is within refund window (48 hours before event)
    const eventDate = new Date(registration.date_time)
    const refundDeadline = new Date(eventDate.getTime() - 48 * 60 * 60 * 1000)
    
    if (new Date() > refundDeadline) {
      res.status(400).json({
        success: false,
        error: 'Refund deadline has passed (48 hours before event)'
      })
      return
    }

    if (!registration.payment_intent_id) {
      res.status(400).json({
        success: false,
        error: 'No payment intent found for refund'
      })
      return
    }

    // Create refund
    const refund = await stripe.refunds.create({
      payment_intent: registration.payment_intent_id,
      reason: 'requested_by_customer',
      metadata: {
        registrationId: registration.id,
        userId: req.user.id,
        reason: reason || 'Customer requested refund'
      }
    })

    // Update registration status
    await pool.query(
      `UPDATE registrations 
       SET payment_status = 'refunded', status = 'cancelled', updated_at = CURRENT_TIMESTAMP 
       WHERE id = $1`,
      [registrationId]
    )

    // Log refund
    await pool.query(
      `INSERT INTO audit_logs (user_id, action, resource_type, resource_id, details, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [
        req.user.id,
        'refund_created',
        'payment',
        refund.id,
        JSON.stringify({
          registrationId,
          eventName: registration.event_name,
          amount: refund.amount / 100,
          reason
        }),
        req.ip,
        req.get('User-Agent')
      ]
    )

    logger.info('Refund created:', {
      refundId: refund.id,
      userId: req.user.id,
      registrationId,
      amount: refund.amount / 100
    })

    res.json({
      success: true,
      data: {
        refundId: refund.id,
        amount: refund.amount / 100,
        status: refund.status,
        message: 'Refund processed successfully'
      }
    })
  } catch (error) {
    logger.error('Create refund error:', error)
    
    if (error instanceof Stripe.errors.StripeError) {
      res.status(400).json({
        success: false,
        error: error.message
      })
      return
    }

    res.status(500).json({
      success: false,
      error: 'Failed to process refund'
    })
  }
}

export const handleWebhook = async (req: any, res: Response): Promise<void> => {
  const sig = req.headers['stripe-signature']
  const endpointSecret = config.stripe.webhookSecret

  let event: Stripe.Event

  try {
    event = stripe.webhooks.constructEvent(req.body, sig, endpointSecret)
  } catch (err: any) {
    logger.error('Webhook signature verification failed:', err.message)
    res.status(400).send(`Webhook Error: ${err.message}`)
    return
  }

  // Handle the event
  switch (event.type) {
    case 'payment_intent.succeeded':
      const paymentIntent = event.data.object as Stripe.PaymentIntent
      
      // Update registration status
      await pool.query(
        `UPDATE registrations 
         SET payment_status = 'paid', status = 'approved', updated_at = CURRENT_TIMESTAMP 
         WHERE payment_intent_id = $1`,
        [paymentIntent.id]
      )

      logger.info('Payment succeeded via webhook:', { paymentIntentId: paymentIntent.id })
      break

    case 'payment_intent.payment_failed':
      const failedPayment = event.data.object as Stripe.PaymentIntent
      
      // Log payment failure
      await pool.query(
        `INSERT INTO audit_logs (action, resource_type, resource_id, details)
         VALUES ($1, $2, $3, $4)`,
        [
          'payment_failed',
          'payment',
          failedPayment.id,
          JSON.stringify({
            registrationId: failedPayment.metadata.registrationId,
            error: failedPayment.last_payment_error?.message
          })
        ]
      )

      logger.warn('Payment failed via webhook:', { 
        paymentIntentId: failedPayment.id,
        error: failedPayment.last_payment_error?.message
      })
      break

    case 'refund.created':
      const refund = event.data.object as Stripe.Refund
      logger.info('Refund created via webhook:', { refundId: refund.id })
      break

    default:
      logger.info('Unhandled webhook event type:', event.type)
  }

  res.json({ received: true })
}