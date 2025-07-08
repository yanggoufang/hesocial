import { 
  ParticipantListResponse, 
  ParticipantAccessCheckResponse, 
  ParticipantFilters,
  FilteredParticipantInfo
} from '../types/participant'
import { ApiResponse } from '../types/index'

const API_BASE = '/api'

class ParticipantService {
  private async request<T>(endpoint: string, options?: RequestInit): Promise<ApiResponse<T>> {
    const token = localStorage.getItem('token')
    
    const response = await fetch(`${API_BASE}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
        ...options?.headers,
      },
      ...options,
    })

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`)
    }

    return response.json()
  }

  /**
   * Get filtered list of event participants
   */
  async getEventParticipants(
    eventId: string, 
    page: number = 1, 
    limit: number = 20,
    filters: ParticipantFilters = {}
  ): Promise<ApiResponse<ParticipantListResponse>> {
    const params = new URLSearchParams({
      page: page.toString(),
      limit: limit.toString(),
      ...(filters.membershipTier && { membershipTier: filters.membershipTier }),
      ...(filters.profession && { profession: filters.profession }),
      ...(filters.minPrivacyLevel && { minPrivacyLevel: filters.minPrivacyLevel.toString() }),
      ...(filters.search && { search: filters.search })
    })

    return this.request<ParticipantListResponse>(`/events/${eventId}/participants?${params}`)
  }

  /**
   * Check participant access for an event
   */
  async checkParticipantAccess(eventId: string): Promise<ApiResponse<ParticipantAccessCheckResponse>> {
    return this.request<ParticipantAccessCheckResponse>(`/events/${eventId}/participant-access`)
  }

  /**
   * Get detailed information about a specific participant
   */
  async getParticipantDetails(
    eventId: string, 
    participantId: string
  ): Promise<ApiResponse<{ participant: FilteredParticipantInfo; viewerAccess: any }>> {
    return this.request(`/events/${eventId}/participants/${participantId}`)
  }

  /**
   * Initiate contact with a participant
   */
  async initiateContact(
    eventId: string, 
    participantId: string, 
    message: string
  ): Promise<ApiResponse<any>> {
    return this.request(`/events/${eventId}/participants/${participantId}/contact`, {
      method: 'POST',
      body: JSON.stringify({ message })
    })
  }

  /**
   * Update privacy settings for an event
   */
  async updatePrivacySettings(
    eventId: string, 
    settings: {
      privacyLevel?: number
      allowContact?: boolean
      showInList?: boolean
    }
  ): Promise<ApiResponse<any>> {
    return this.request(`/events/${eventId}/privacy-settings`, {
      method: 'PUT',
      body: JSON.stringify(settings)
    })
  }

  /**
   * Get privacy settings for an event
   */
  async getPrivacySettings(eventId: string): Promise<ApiResponse<any>> {
    return this.request(`/events/${eventId}/privacy-settings`)
  }
}

export const participantService = new ParticipantService()
export default participantService