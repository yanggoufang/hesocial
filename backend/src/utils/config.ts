import dotenv from 'dotenv'
import { DatabaseConfig, RedisConfig, EmailConfig, StripeConfig, OAuthConfig } from '@/types/index.js'

dotenv.config()

interface Config {
  port: number
  nodeEnv: string
  jwtSecret: string
  jwtExpiresIn: string
  database: DatabaseConfig
  redis: RedisConfig
  email: EmailConfig
  stripe: StripeConfig
  oauth: OAuthConfig
  corsOrigins: string[]
  rateLimitWindowMinutes: number
  rateLimitMaxRequests: number
  fileUploadMaxSize: number
  fileUploadPath: string
}

const config: Config = {
  port: parseInt(process.env.PORT || '5000', 10),
  nodeEnv: process.env.NODE_ENV || 'development',
  jwtSecret: process.env.JWT_SECRET || 'fallback-secret-key-change-in-production',
  jwtExpiresIn: process.env.JWT_EXPIRES_IN || '7d',

  database: {
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT || '5432', 10),
    database: process.env.DB_NAME || 'hesocial',
    username: process.env.DB_USER || 'postgres',
    password: process.env.DB_PASSWORD || 'password',
    ssl: process.env.DB_SSL === 'true'
  },

  redis: {
    host: process.env.REDIS_HOST || 'localhost',
    port: parseInt(process.env.REDIS_PORT || '6379', 10),
    password: process.env.REDIS_PASSWORD
  },

  email: {
    host: process.env.EMAIL_HOST || 'smtp.ethereal.email',
    port: parseInt(process.env.EMAIL_PORT || '587', 10),
    secure: process.env.EMAIL_SECURE === 'true',
    auth: {
      user: process.env.EMAIL_USER || '',
      pass: process.env.EMAIL_PASS || ''
    }
  },

  stripe: {
    publicKey: process.env.STRIPE_PUBLIC_KEY || '',
    secretKey: process.env.STRIPE_SECRET_KEY || '',
    webhookSecret: process.env.STRIPE_WEBHOOK_SECRET || ''
  },

  oauth: {
    google: {
      clientId: process.env.GOOGLE_CLIENT_ID || '',
      clientSecret: process.env.GOOGLE_CLIENT_SECRET || ''
    },
    linkedin: {
      clientId: process.env.LINKEDIN_CLIENT_ID || '',
      clientSecret: process.env.LINKEDIN_CLIENT_SECRET || ''
    }
  },

  corsOrigins: process.env.CORS_ORIGINS?.split(',') || ['http://localhost:3000'],
  rateLimitWindowMinutes: parseInt(process.env.RATE_LIMIT_WINDOW_MINUTES || '15', 10),
  rateLimitMaxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10),
  fileUploadMaxSize: parseInt(process.env.FILE_UPLOAD_MAX_SIZE || '10485760', 10), // 10MB
  fileUploadPath: process.env.FILE_UPLOAD_PATH || './uploads'
}

export default config