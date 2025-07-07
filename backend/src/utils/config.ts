import dotenv from 'dotenv'
import { EmailConfig, StripeConfig, OAuthConfig } from '../types/index.js'

dotenv.config()

interface R2Config {
  endpoint: string
  accessKeyId: string
  secretAccessKey: string
  bucketName: string
  publicUrl: string
}

interface Config {
  port: number
  nodeEnv: string
  jwtSecret: string
  jwtExpiresIn: string
  email: EmailConfig
  stripe: StripeConfig
  oauth: OAuthConfig
  r2: R2Config
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

  r2: {
    endpoint: process.env.R2_ENDPOINT || 'https://hesocial.r2.cloudflarestorage.com',
    accessKeyId: process.env.R2_ACCESS_KEY_ID || '',
    secretAccessKey: process.env.R2_SECRET_ACCESS_KEY || '',
    bucketName: process.env.R2_BUCKET_NAME || 'hesocial-media',
    publicUrl: process.env.R2_PUBLIC_URL || 'https://media.hesocial.com'
  },

  corsOrigins: process.env.CORS_ORIGINS?.split(',') || ['http://localhost:3000'],
  rateLimitWindowMinutes: parseInt(process.env.RATE_LIMIT_WINDOW_MINUTES || '15', 10),
  rateLimitMaxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '1000', 10),
  fileUploadMaxSize: parseInt(process.env.FILE_UPLOAD_MAX_SIZE || '10485760', 10), // 10MB
  fileUploadPath: process.env.FILE_UPLOAD_PATH || './uploads'
}

export default config