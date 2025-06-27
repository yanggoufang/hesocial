import { Request, Response, NextFunction } from 'express'
import Joi from 'joi'
import logger from '../utils/logger.js'

export const validateRequest = (schema: {
  body?: Joi.ObjectSchema
  query?: Joi.ObjectSchema
  params?: Joi.ObjectSchema
}) => {
  return (req: Request, res: Response, next: NextFunction): void => {
    const errors: string[] = []

    if (schema.body) {
      const { error } = schema.body.validate(req.body)
      if (error) {
        errors.push(`Body: ${error.details.map(d => d.message).join(', ')}`)
      }
    }

    if (schema.query) {
      const { error } = schema.query.validate(req.query)
      if (error) {
        errors.push(`Query: ${error.details.map(d => d.message).join(', ')}`)
      }
    }

    if (schema.params) {
      const { error } = schema.params.validate(req.params)
      if (error) {
        errors.push(`Params: ${error.details.map(d => d.message).join(', ')}`)
      }
    }

    if (errors.length > 0) {
      logger.warn('Validation error:', { errors, url: req.url, method: req.method })
      res.status(400).json({
        success: false,
        error: 'Validation failed',
        details: errors
      })
      return
    }

    next()
  }
}

export const userRegistrationSchema = {
  body: Joi.object({
    email: Joi.string().email().required(),
    password: Joi.string().min(8).pattern(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/).required()
      .messages({
        'string.pattern.base': 'Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character'
      }),
    firstName: Joi.string().min(1).max(50).required(),
    lastName: Joi.string().min(1).max(50).required(),
    age: Joi.number().integer().min(18).max(100).required(),
    profession: Joi.string().min(2).max(100).required(),
    annualIncome: Joi.number().integer().min(5000000).required(), // NT$5M minimum
    netWorth: Joi.number().integer().min(30000000).required(), // NT$30M minimum
    membershipTier: Joi.string().valid('Platinum', 'Diamond', 'Black Card').required(),
    bio: Joi.string().max(500).optional(),
    interests: Joi.array().items(Joi.string().max(50)).min(1).max(10).required()
  })
}

export const userLoginSchema = {
  body: Joi.object({
    email: Joi.string().email().required(),
    password: Joi.string().required()
  })
}

export const eventCreationSchema = {
  body: Joi.object({
    name: Joi.string().min(5).max(200).required(),
    description: Joi.string().min(10).max(2000).required(),
    dateTime: Joi.date().greater('now').required(),
    registrationDeadline: Joi.date().greater('now').less(Joi.ref('dateTime')).required(),
    venueId: Joi.string().uuid().required(),
    categoryId: Joi.string().uuid().required(),
    pricing: Joi.object({
      vip: Joi.number().integer().min(1000).optional(),
      vvip: Joi.number().integer().min(5000).optional(),
      general: Joi.number().integer().min(500).optional(),
      currency: Joi.string().valid('TWD', 'USD').required(),
      installmentOptions: Joi.array().items(Joi.number().integer().valid(6, 12, 18, 24)).optional()
    }).required(),
    exclusivityLevel: Joi.string().valid('VIP', 'VVIP', 'Invitation Only').required(),
    dressCode: Joi.number().integer().min(1).max(5).required(),
    capacity: Joi.number().integer().min(2).max(500).required(),
    amenities: Joi.array().items(Joi.string().max(100)).max(20).optional(),
    privacyGuarantees: Joi.array().items(Joi.string().max(200)).max(10).optional(),
    requirements: Joi.array().items(
      Joi.object({
        type: Joi.string().valid('age', 'income', 'membership', 'verification').required(),
        value: Joi.alternatives().try(Joi.string(), Joi.number()).required(),
        description: Joi.string().max(200).required()
      })
    ).max(10).optional()
  })
}

export const eventRegistrationSchema = {
  body: Joi.object({
    eventId: Joi.string().uuid().required(),
    specialRequests: Joi.string().max(500).optional()
  })
}

export const profileUpdateSchema = {
  body: Joi.object({
    firstName: Joi.string().min(1).max(50).optional(),
    lastName: Joi.string().min(1).max(50).optional(),
    profession: Joi.string().min(2).max(100).optional(),
    bio: Joi.string().max(500).optional(),
    interests: Joi.array().items(Joi.string().max(50)).min(1).max(10).optional(),
    privacyLevel: Joi.number().integer().min(1).max(5).optional()
  })
}

export const queryPaginationSchema = {
  query: Joi.object({
    page: Joi.number().integer().min(1).default(1),
    limit: Joi.number().integer().min(1).max(100).default(20),
    sort: Joi.string().valid('created_at', 'updated_at', 'name', 'date_time').default('created_at'),
    order: Joi.string().valid('asc', 'desc').default('desc')
  })
}