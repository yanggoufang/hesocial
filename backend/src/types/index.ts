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
  role?: 'user' | 'admin' | 'super_admin'
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

import { Request } from 'express'

export interface AuthenticatedRequest extends Request {
  user?: User;
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

// Sales Management System Types
export interface SalesLead {
  id: string
  firstName: string
  lastName: string
  email: string
  phone?: string
  company?: string
  jobTitle?: string
  annualIncome?: number
  netWorth?: number
  source: 'website' | 'referral' | 'event' | 'cold_call' | 'linkedin' | 'advertisement'
  referralCode?: string
  leadScore: number
  status: 'new' | 'qualified' | 'contacted' | 'nurturing' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  interestedMembershipTier?: 'Platinum' | 'Diamond' | 'Black Card'
  budgetRange?: string
  timeline?: string
  painPoints?: string
  interests: string[]
  notes?: string
  lastContactDate?: Date
  nextFollowUpDate?: Date
  assignedTo?: string
  createdAt: Date
  updatedAt: Date
}

export interface SalesOpportunity {
  id: string
  leadId: string
  name: string
  description?: string
  stage: 'qualification' | 'needs_analysis' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  probability: number
  value: number
  expectedCloseDate?: Date
  actualCloseDate?: Date
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  paymentTerms?: string
  closeReason?: string
  assignedTo: string
  createdAt: Date
  updatedAt: Date
}

export interface SalesActivity {
  id: string
  leadId?: string
  opportunityId?: string
  activityType: 'call' | 'email' | 'meeting' | 'demo' | 'proposal' | 'follow_up' | 'note'
  subject: string
  description?: string
  outcome?: string
  durationMinutes?: number
  scheduledAt?: Date
  completedAt?: Date
  createdBy: string
  createdAt: Date
  updatedAt: Date
}

export interface SalesPipelineStage {
  id: string
  name: string
  displayOrder: number
  defaultProbability: number
  isActive: boolean
  colorCode?: string
  createdAt: Date
  updatedAt: Date
}

export interface SalesTarget {
  id: string
  salesRepId: string
  periodType: 'monthly' | 'quarterly' | 'yearly'
  periodStart: Date
  periodEnd: Date
  targetRevenue: number
  targetLeads?: number
  targetConversions?: number
  actualRevenue: number
  actualLeads: number
  actualConversions: number
  createdAt: Date
  updatedAt: Date
}

export interface SalesTeamMember {
  id: string
  userId: string
  role: 'sales_rep' | 'senior_sales_rep' | 'sales_manager' | 'sales_director'
  territory?: string
  commissionRate: number
  isActive: boolean
  hireDate?: Date
  managerId?: string
  createdAt: Date
  updatedAt: Date
}

export interface SalesCommission {
  id: string
  salesRepId: string
  opportunityId: string
  commissionType: 'new_member' | 'renewal' | 'upsell' | 'referral'
  baseAmount: number
  commissionRate: number
  commissionAmount: number
  paymentStatus: 'pending' | 'approved' | 'paid' | 'disputed'
  periodMonth: number
  periodYear: number
  paidAt?: Date
  createdAt: Date
  updatedAt: Date
}

export interface SalesMetrics {
  totalLeads: number
  qualifiedLeads: number
  totalOpportunities: number
  totalPipelineValue: number
  conversionRate: number
  averageDealSize: number
  salesCycleLength: number
  winRate: number
  monthlyRevenue: number
  quarterlyRevenue: number
  yearlyRevenue: number
}

export interface SalesLeadFilters {
  status?: SalesLead['status']
  source?: SalesLead['source']
  assignedTo?: string
  membershipTier?: SalesLead['interestedMembershipTier']
  leadScore?: {
    min?: number
    max?: number
  }
  dateRange?: {
    start?: Date
    end?: Date
  }
}

export interface SalesOpportunityFilters {
  stage?: SalesOpportunity['stage']
  assignedTo?: string
  membershipTier?: SalesOpportunity['membershipTier']
  probability?: {
    min?: number
    max?: number
  }
  value?: {
    min?: number
    max?: number
  }
  dateRange?: {
    start?: Date
    end?: Date
  }
}

// Event Registration System Types
export interface RegistrationRequest {
  specialRequests?: string
}

export interface RegistrationDetails extends Registration {
  eventName: string
  eventDescription: string
  eventDateTime: Date
  registrationDeadline: Date
  exclusivityLevel: string
  dressCode: number
  capacity: number
  currentAttendees: number
  pricing: any
  amenities: string[]
  privacyGuarantees: string[]
  requirements: any[]
  eventImages: string[]
  venueName: string
  venueAddress: string
  latitude: number
  longitude: number
  venueAmenities: string[]
  venueImages: string[]
  categoryName: string
  categoryDescription: string
}

export interface RegistrationFilters {
  status?: Registration['status']
  paymentStatus?: Registration['paymentStatus']
  eventId?: string
  userId?: string
  search?: string
  dateRange?: {
    start?: Date
    end?: Date
  }
}

export interface RegistrationApproval {
  approved: boolean
  notes?: string
}

export interface PaymentStatusUpdate {
  paymentStatus: Registration['paymentStatus']
  paymentIntentId?: string
}

// Participant Access Control System Types
export interface ParticipantViewAccess {
  canViewParticipants: boolean
  maxPrivacyLevelVisible: number
  canSeeContactInfo: boolean
  canInitiateContact: boolean
  participantCountVisible: boolean
  accessLevel: number
}

export interface EventParticipantAccess {
  id: string
  eventId: string
  userId: string
  paymentStatus: 'pending' | 'paid' | 'refunded'
  accessLevel: number
  registrationId?: string
  grantedAt: Date
  expiresAt?: Date
  createdAt: Date
  updatedAt: Date
}

export interface ParticipantViewLog {
  id: string
  viewerId: string
  viewedParticipantId: string
  eventId: string
  viewTimestamp: Date
  viewType: 'list' | 'profile' | 'contact'
  ipAddress?: string
  userAgent?: string
}

export interface EventPrivacyOverride {
  id: string
  userId: string
  eventId: string
  privacyLevel: number
  allowContact: boolean
  showInList: boolean
  validUntil?: Date
  createdAt: Date
  updatedAt: Date
}

export interface FilteredParticipantInfo {
  id: string
  displayName: string
  profession?: string
  company?: string
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  interests?: string[]
  profilePicture?: string
  ageRange?: string
  city?: string
  bio?: string
  privacyLevel: number
  canContact: boolean
  contactInfo?: {
    email?: string
    phone?: string
    linkedIn?: string
    personalSocial?: Record<string, string>
  }
}

export interface ParticipantListResponse {
  participants: FilteredParticipantInfo[]
  totalCount: number
  paidParticipantCount: number
  unpaidParticipantCount: number
  viewerAccess: ParticipantViewAccess
  participantCountByTier: Record<string, number>
}

export interface ParticipantAccessCheckResponse {
  hasAccess: boolean
  accessLevel: ParticipantViewAccess
  paymentRequired: boolean
  paymentStatus: string
  registrationStatus?: string
}