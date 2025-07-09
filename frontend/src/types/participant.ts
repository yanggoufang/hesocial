export interface ParticipantViewAccess {
  canViewParticipants: boolean
  maxPrivacyLevelVisible: number
  canSeeContactInfo: boolean
  canInitiateContact: boolean
  participantCountVisible: boolean
  accessLevel: number
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

export interface ParticipantFilters {
  membershipTier?: string
  profession?: string
  minPrivacyLevel?: number
  search?: string
}