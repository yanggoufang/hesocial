import { Request } from 'express'

export interface User {
  id: string
  email: string
  firstName: string
  lastName: string
  age: number
  profession: string
  annualIncome: number
  netWorth: number
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  privacyLevel: 1 | 2 | 3 | 4 | 5
  isVerified: boolean
  verificationStatus: 'pending' | 'approved' | 'rejected'
  profilePicture?: string
  bio?: string
  interests: string[]
  createdAt: Date
  updatedAt: Date
  deletedAt?: Date
}

export interface Event {
  id: string
  name: string
  description: string
  dateTime: Date
  registrationDeadline: Date
  venueId: string
  categoryId: string
  organizerId: string
  pricing: EventPricing
  exclusivityLevel: 'VIP' | 'VVIP' | 'Invitation Only'
  dressCode: 1 | 2 | 3 | 4 | 5
  capacity: number
  currentAttendees: number
  amenities: string[]
  privacyGuarantees: string[]
  images: string[]
  videoUrl?: string
  requirements: EventRequirement[]
  isActive: boolean
  createdAt: Date
  updatedAt: Date
}

export interface Venue {
  id: string
  name: string
  address: string
  city: string
  coordinates: {
    lat: number
    lng: number
  }
  rating: number
  amenities: string[]
  images: string[]
  createdAt: Date
  updatedAt: Date
}

export interface EventCategory {
  id: string
  name: string
  description: string
  icon: string
  createdAt: Date
  updatedAt: Date
}

export interface EventPricing {
  vip?: number
  vvip?: number
  general?: number
  currency: 'TWD' | 'USD'
  installmentOptions?: number[]
}

export interface EventRequirement {
  type: 'age' | 'income' | 'membership' | 'verification'
  value: string | number
  description: string
}

export interface Registration {
  id: string
  userId: string
  eventId: string
  status: 'pending' | 'approved' | 'rejected' | 'cancelled'
  paymentStatus: 'pending' | 'paid' | 'refunded'
  paymentIntentId?: string
  specialRequests?: string
  createdAt: Date
  updatedAt: Date
}

export interface FinancialVerification {
  id: string
  userId: string
  incomeProofUrl: string
  assetDocuments: Record<string, any>
  verificationDate?: Date
  verifiedBy?: string
  status: 'pending' | 'approved' | 'rejected'
  notes?: string
  createdAt: Date
  updatedAt: Date
}

export interface AuthenticatedRequest extends Request {
  user?: User
}

export interface JwtPayload {
  userId: string
  email: string
  membershipTier: string
  iat?: number
  exp?: number
}

export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  message?: string
  error?: string
  pagination?: {
    page: number
    limit: number
    total: number
    totalPages: number
  }
}

export interface DatabaseConfig {
  host: string
  port: number
  database: string
  username: string
  password: string
  ssl?: boolean
}

export interface RedisConfig {
  host: string
  port: number
  password?: string
}

export interface EmailConfig {
  host: string
  port: number
  secure: boolean
  auth: {
    user: string
    pass: string
  }
}

export interface StripeConfig {
  publicKey: string
  secretKey: string
  webhookSecret: string
}

export interface OAuthConfig {
  google: {
    clientId: string
    clientSecret: string
  }
  linkedin: {
    clientId: string
    clientSecret: string
  }
}