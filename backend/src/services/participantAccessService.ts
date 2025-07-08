import { getDuckDBConnection } from '../database/duckdb-connection.js'
import { 
  ParticipantViewAccess, 
  FilteredParticipantInfo, 
  ParticipantListResponse,
  ParticipantAccessCheckResponse,
  User,
  Registration 
} from '../types/index.js'
import logger from '../utils/logger.js'

export class ParticipantAccessService {
  constructor(private db: any) {}

  /**
   * Get participant view access level for a user
   */
  async getParticipantViewAccess(
    userId: string,
    eventId: string,
    membershipTier: string
  ): Promise<ParticipantViewAccess> {
    try {
      // Check if user has paid access to this event
      const result = await this.db.query(`
        SELECT payment_status, access_level, created_at
        FROM event_participant_access 
        WHERE user_id = ? AND event_id = ?
      `, [userId, eventId])
      const accessRecord = result.rows[0]

      // Default no access
      if (!accessRecord || accessRecord.payment_status !== 'paid') {
        return {
          canViewParticipants: false,
          maxPrivacyLevelVisible: 0,
          canSeeContactInfo: false,
          canInitiateContact: false,
          participantCountVisible: true,
          accessLevel: 0
        }
      }

      // Base access for paid participants
      const baseAccess: ParticipantViewAccess = {
        canViewParticipants: true,
        maxPrivacyLevelVisible: 3,
        canSeeContactInfo: false,
        canInitiateContact: true,
        participantCountVisible: true,
        accessLevel: 3
      }

      // Enhanced access for premium tiers
      if (membershipTier === 'Diamond' || membershipTier === 'Black Card') {
        baseAccess.maxPrivacyLevelVisible = 5
        baseAccess.canSeeContactInfo = true
        baseAccess.accessLevel = 4
      }

      return baseAccess
    } catch (error) {
      logger.error('Error getting participant view access:', error)
      return {
        canViewParticipants: false,
        maxPrivacyLevelVisible: 0,
        canSeeContactInfo: false,
        canInitiateContact: false,
        participantCountVisible: true,
        accessLevel: 0
      }
    }
  }

  /**
   * Check if user has access to view participants for an event
   */
  async checkParticipantAccess(userId: string, eventId: string): Promise<ParticipantAccessCheckResponse> {
    try {
      // Get user's membership tier
      const userResult = await this.db.query('SELECT membership_tier FROM users WHERE id = ?', [userId])
      const user = userResult.rows[0]
      if (!user) {
        throw new Error('User not found')
      }

      // Get registration and access status
      const registrationResult = await this.db.query(`
        SELECT r.status, r.payment_status, epa.access_level
        FROM registrations r
        LEFT JOIN event_participant_access epa ON r.id = epa.registration_id
        WHERE r.user_id = ? AND r.event_id = ?
      `, [userId, eventId])
      const registration = registrationResult.rows[0]

      const accessLevel = await this.getParticipantViewAccess(userId, eventId, user.membership_tier)

      return {
        hasAccess: accessLevel.canViewParticipants,
        accessLevel,
        paymentRequired: !accessLevel.canViewParticipants,
        paymentStatus: registration?.payment_status || 'none',
        registrationStatus: registration?.status || 'none'
      }
    } catch (error) {
      logger.error('Error checking participant access:', error)
      return {
        hasAccess: false,
        accessLevel: {
          canViewParticipants: false,
          maxPrivacyLevelVisible: 0,
          canSeeContactInfo: false,
          canInitiateContact: false,
          participantCountVisible: true,
          accessLevel: 0
        },
        paymentRequired: true,
        paymentStatus: 'unknown'
      }
    }
  }

  /**
   * Get filtered participant information based on privacy levels and viewer access
   */
  async getEventParticipants(
    eventId: string,
    viewerId: string,
    page: number = 1,
    limit: number = 20,
    filters: {
      membershipTier?: string
      profession?: string
      minPrivacyLevel?: number
    } = {}
  ): Promise<ParticipantListResponse> {
    try {
      // Get viewer's access level
      const viewerResult = await this.db.query('SELECT membership_tier FROM users WHERE id = ?', [viewerId])
      const viewer = viewerResult.rows[0]
      if (!viewer) {
        throw new Error('Viewer not found')
      }

      const viewerAccess = await this.getParticipantViewAccess(viewerId, eventId, viewer.membership_tier)

      // If no access, return limited information
      if (!viewerAccess.canViewParticipants) {
        const totalCount = await this.getParticipantCounts(eventId)
        return {
          participants: [],
          totalCount: totalCount.total,
          paidParticipantCount: totalCount.paid,
          unpaidParticipantCount: totalCount.unpaid,
          viewerAccess,
          participantCountByTier: totalCount.byTier
        }
      }

      // Build query for participants with access control
      let whereConditions = [
        'epa.event_id = ?',
        'epa.payment_status = "paid"',
        'u.show_in_participant_lists = true'
      ]
      
      let queryParams: any[] = [eventId]

      // Add privacy level filter
      const effectivePrivacyLevel = `
        COALESCE(epo.privacy_level, u.default_privacy_level, 2)
      `
      
      whereConditions.push(`${effectivePrivacyLevel} <= ?`)
      queryParams.push(viewerAccess.maxPrivacyLevelVisible)

      // Add optional filters
      if (filters.membershipTier) {
        whereConditions.push('u.membership_tier = ?')
        queryParams.push(filters.membershipTier)
      }

      if (filters.profession) {
        whereConditions.push('u.profession LIKE ?')
        queryParams.push(`%${filters.profession}%`)
      }

      if (filters.minPrivacyLevel) {
        whereConditions.push(`${effectivePrivacyLevel} >= ?`)
        queryParams.push(filters.minPrivacyLevel)
      }

      // Exclude the viewer from results
      whereConditions.push('u.id != ?')
      queryParams.push(viewerId)

      const whereClause = whereConditions.join(' AND ')

      // Get total count
      const countQuery = `
        SELECT COUNT(*) as count
        FROM users u
        JOIN event_participant_access epa ON u.id = epa.user_id
        LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ?
        WHERE ${whereClause}
      `
      
      const totalCountResult = await this.db.query(countQuery, [eventId, ...queryParams])
      const totalCount = totalCountResult.rows[0]

      // Get participants with pagination
      const offset = (page - 1) * limit
      const participantsQuery = `
        SELECT 
          u.id,
          u.first_name,
          u.last_name,
          u.email,
          u.phone,
          u.age,
          u.profession,
          u.company,
          u.city,
          u.membership_tier,
          u.interests,
          u.profile_picture,
          u.bio,
          u.default_privacy_level,
          u.allow_contact_requests,
          COALESCE(epo.privacy_level, u.default_privacy_level, 2) as effective_privacy_level,
          COALESCE(epo.allow_contact, u.allow_contact_requests, true) as can_contact,
          COALESCE(epo.show_in_list, u.show_in_participant_lists, true) as show_in_list
        FROM users u
        JOIN event_participant_access epa ON u.id = epa.user_id
        LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ?
        WHERE ${whereClause}
        ORDER BY u.membership_tier DESC, u.first_name ASC
        LIMIT ? OFFSET ?
      `

      const participants = this.db.prepare(participantsQuery).all(
        eventId, ...queryParams, limit, offset
      ) as any[]

      // Filter and format participant information
      const filteredParticipants = participants
        .map(participant => this.filterParticipantInfo(participant, viewerAccess))
        .filter(p => p !== null) as FilteredParticipantInfo[]

      // Log the view activity
      this.logParticipantView(
        viewerId, 
        filteredParticipants.map(p => p.id), 
        eventId, 
        'list'
      )

      // Get additional counts
      const counts = this.getParticipantCounts(eventId)

      return {
        participants: filteredParticipants,
        totalCount: totalCount.count,
        paidParticipantCount: counts.paid,
        unpaidParticipantCount: counts.unpaid,
        viewerAccess,
        participantCountByTier: counts.byTier
      }
    } catch (error) {
      logger.error('Error getting event participants:', error)
      throw error
    }
  }

  /**
   * Filter participant information based on privacy level and viewer access
   */
  private filterParticipantInfo(
    participant: any,
    viewerAccess: ParticipantViewAccess
  ): FilteredParticipantInfo | null {
    const privacyLevel = participant.effective_privacy_level

    // If participant's privacy level exceeds viewer's access, hide completely
    if (privacyLevel > viewerAccess.maxPrivacyLevelVisible) {
      return null
    }

    const filtered: FilteredParticipantInfo = {
      id: participant.id,
      displayName: this.getDisplayName(participant, privacyLevel),
      membershipTier: participant.membership_tier,
      privacyLevel,
      canContact: participant.can_contact && viewerAccess.canInitiateContact
    }

    // Add information based on privacy level
    if (privacyLevel >= 1) {
      filtered.profession = this.getProfessionCategory(participant.profession)
      filtered.interests = this.parseInterests(participant.interests).slice(0, 3)
      filtered.profilePicture = participant.profile_picture
      filtered.ageRange = this.getAgeRange(participant.age)
    }

    if (privacyLevel >= 2) {
      filtered.city = participant.city
      filtered.company = this.getCompanyCategory(participant.company)
    }

    if (privacyLevel >= 3) {
      filtered.displayName = `${participant.first_name} ${participant.last_name}`
      filtered.company = participant.company
      filtered.bio = participant.bio?.substring(0, 200)
    }

    if (privacyLevel >= 4 && viewerAccess.canSeeContactInfo) {
      filtered.bio = participant.bio
      filtered.contactInfo = {
        email: participant.email
      }
    }

    if (privacyLevel >= 5 && viewerAccess.canSeeContactInfo) {
      filtered.contactInfo = {
        ...filtered.contactInfo,
        phone: participant.phone
      }
    }

    return filtered
  }

  /**
   * Get display name based on privacy level
   */
  private getDisplayName(participant: any, privacyLevel: number): string {
    if (privacyLevel >= 3) {
      return `${participant.first_name} ${participant.last_name}`
    } else if (privacyLevel >= 2) {
      return `${participant.first_name} ${participant.last_name.charAt(0)}.`
    } else {
      return `${participant.first_name} ${participant.last_name.charAt(0)}.`
    }
  }

  /**
   * Get profession category (generalized profession)
   */
  private getProfessionCategory(profession: string): string {
    if (!profession) return 'Professional'
    
    const categories: Record<string, string[]> = {
      'Technology': ['software', 'engineer', 'developer', 'tech', 'it', 'data', 'ai', 'machine learning'],
      'Finance': ['finance', 'banking', 'investment', 'fund', 'trading', 'analyst', 'wealth'],
      'Healthcare': ['doctor', 'physician', 'medical', 'health', 'nurse', 'surgeon'],
      'Business': ['ceo', 'manager', 'director', 'executive', 'business', 'entrepreneur'],
      'Legal': ['lawyer', 'attorney', 'legal', 'counsel', 'judge'],
      'Real Estate': ['real estate', 'property', 'development', 'construction'],
      'Consulting': ['consultant', 'consulting', 'advisory'],
      'Education': ['professor', 'teacher', 'education', 'academic'],
      'Media': ['media', 'journalist', 'marketing', 'advertising', 'pr']
    }

    const lowerProfession = profession.toLowerCase()
    
    for (const [category, keywords] of Object.entries(categories)) {
      if (keywords.some(keyword => lowerProfession.includes(keyword))) {
        return category
      }
    }
    
    return 'Professional'
  }

  /**
   * Get company category (for privacy level 2)
   */
  private getCompanyCategory(company: string): string {
    if (!company) return undefined
    
    // For privacy level 2, show industry rather than specific company
    // This would ideally be enhanced with a proper company database
    if (company.toLowerCase().includes('tech') || 
        company.toLowerCase().includes('software') ||
        company.toLowerCase().includes('microsoft') ||
        company.toLowerCase().includes('google') ||
        company.toLowerCase().includes('apple')) {
      return 'Technology Company'
    }
    
    if (company.toLowerCase().includes('bank') ||
        company.toLowerCase().includes('financial') ||
        company.toLowerCase().includes('investment')) {
      return 'Financial Services'
    }
    
    return 'Private Company'
  }

  /**
   * Get age range from specific age
   */
  private getAgeRange(age: number): string {
    if (age < 25) return '18-24'
    if (age < 30) return '25-29'
    if (age < 35) return '30-34'
    if (age < 40) return '35-39'
    if (age < 45) return '40-44'
    if (age < 50) return '45-49'
    if (age < 55) return '50-54'
    if (age < 60) return '55-59'
    if (age < 65) return '60-64'
    return '65+'
  }

  /**
   * Parse interests from JSON string
   */
  private parseInterests(interests: string): string[] {
    try {
      return JSON.parse(interests || '[]')
    } catch {
      return []
    }
  }

  /**
   * Get participant counts and statistics
   */
  private async getParticipantCounts(eventId: string) {
    const countsResult = await this.db.query(`
      SELECT 
        COUNT(*) as total,
        SUM(CASE WHEN epa.payment_status = 'paid' THEN 1 ELSE 0 END) as paid,
        SUM(CASE WHEN epa.payment_status != 'paid' THEN 1 ELSE 0 END) as unpaid
      FROM event_participant_access epa
      WHERE epa.event_id = ?
    `, [eventId])
    const counts = countsResult.rows[0]

    const byTierResult = await this.db.query(`
      SELECT 
        u.membership_tier,
        COUNT(*) as count
      FROM users u
      JOIN event_participant_access epa ON u.id = epa.user_id
      WHERE epa.event_id = ? AND epa.payment_status = 'paid'
      GROUP BY u.membership_tier
    `, [eventId])
    const byTier = byTierResult.rows

    const tierCounts: Record<string, number> = {}
    byTier.forEach(tier => {
      tierCounts[tier.membership_tier] = tier.count
    })

    return {
      total: counts.total || 0,
      paid: counts.paid || 0,
      unpaid: counts.unpaid || 0,
      byTier: tierCounts
    }
  }

  /**
   * Log participant viewing activity
   */
  private logParticipantView(
    viewerId: string,
    viewedParticipantIds: string[],
    eventId: string,
    viewType: 'list' | 'profile' | 'contact',
    ipAddress?: string,
    userAgent?: string
  ) {
    try {
      const stmt = this.db.prepare(`
        INSERT INTO participant_view_logs (viewer_id, viewed_participant_id, event_id, view_type, ip_address, user_agent)
        VALUES (?, ?, ?, ?, ?, ?)
      `)

      for (const participantId of viewedParticipantIds) {
        stmt.run(viewerId, participantId, eventId, viewType, ipAddress, userAgent)
      }
    } catch (error) {
      logger.error('Error logging participant view:', error)
      // Don't throw - logging shouldn't break the main functionality
    }
  }

  /**
   * Update participant access when payment status changes
   */
  updateParticipantAccess(
    userId: string,
    eventId: string,
    paymentStatus: 'pending' | 'paid' | 'refunded',
    registrationId?: string
  ) {
    try {
      const accessLevel = paymentStatus === 'paid' ? 3 : paymentStatus === 'pending' ? 1 : 0

      const stmt = this.db.prepare(`
        INSERT INTO event_participant_access (user_id, event_id, payment_status, access_level, registration_id)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(event_id, user_id) DO UPDATE SET
        payment_status = excluded.payment_status,
        access_level = excluded.access_level,
        registration_id = excluded.registration_id,
        updated_at = CURRENT_TIMESTAMP
      `)

      stmt.run(userId, eventId, paymentStatus, accessLevel, registrationId)
      
      logger.info(`Updated participant access for user ${userId} on event ${eventId}: ${paymentStatus}`)
    } catch (error) {
      logger.error('Error updating participant access:', error)
      throw error
    }
  }
}

export default ParticipantAccessService