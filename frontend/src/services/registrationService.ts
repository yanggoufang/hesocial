import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

export interface Registration {
  id: string
  userId: string
  eventId: string
  status: 'pending' | 'approved' | 'rejected' | 'cancelled'
  paymentStatus: 'pending' | 'paid' | 'refunded'
  paymentIntentId?: string
  specialRequests?: string
  createdAt: string
  updatedAt: string
}

export interface RegistrationDetails extends Registration {
  eventName: string
  eventDescription: string
  eventDateTime: string
  registrationDeadline: string
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

export interface RegistrationRequest {
  specialRequests?: string
}

export interface RegistrationFilters {
  status?: Registration['status']
  paymentStatus?: Registration['paymentStatus']
  eventId?: string
  userId?: string
  search?: string
  page?: number
  limit?: number
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

class RegistrationService {
  private getAuthHeaders() {
    const token = localStorage.getItem('token')
    return {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    }
  }

  /**
   * Register for an event
   */
  async registerForEvent(eventId: string, data: RegistrationRequest): Promise<ApiResponse> {
    try {
      const response = await axios.post(
        `${API_BASE_URL}/registrations/events/${eventId}`,
        data,
        { headers: this.getAuthHeaders() }
      )
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to register for event'
      }
    }
  }

  /**
   * Get current user's registrations
   */
  async getUserRegistrations(filters: RegistrationFilters = {}): Promise<ApiResponse<Registration[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/registrations/user`, {
        headers: this.getAuthHeaders(),
        params: filters
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch registrations'
      }
    }
  }

  /**
   * Get specific registration details
   */
  async getRegistrationDetails(registrationId: string): Promise<ApiResponse<RegistrationDetails>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/registrations/${registrationId}`, {
        headers: this.getAuthHeaders()
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch registration details'
      }
    }
  }

  /**
   * Update registration (special requests)
   */
  async updateRegistration(registrationId: string, data: RegistrationRequest): Promise<ApiResponse> {
    try {
      const response = await axios.put(
        `${API_BASE_URL}/registrations/${registrationId}`,
        data,
        { headers: this.getAuthHeaders() }
      )
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update registration'
      }
    }
  }

  /**
   * Cancel registration
   */
  async cancelRegistration(registrationId: string): Promise<ApiResponse> {
    try {
      const response = await axios.delete(`${API_BASE_URL}/registrations/${registrationId}`, {
        headers: this.getAuthHeaders()
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to cancel registration'
      }
    }
  }

  // Admin functions

  /**
   * Get all registrations (admin only)
   */
  async getAllRegistrations(filters: RegistrationFilters = {}): Promise<ApiResponse<Registration[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/registrations/admin/all`, {
        headers: this.getAuthHeaders(),
        params: filters
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch all registrations'
      }
    }
  }

  /**
   * Approve or reject registration (admin only)
   */
  async approveRegistration(registrationId: string, approved: boolean, notes?: string): Promise<ApiResponse> {
    try {
      const response = await axios.post(
        `${API_BASE_URL}/registrations/${registrationId}/approve`,
        { approved, notes },
        { headers: this.getAuthHeaders() }
      )
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update registration status'
      }
    }
  }

  /**
   * Update payment status (admin only)
   */
  async updatePaymentStatus(
    registrationId: string, 
    paymentStatus: Registration['paymentStatus'], 
    paymentIntentId?: string
  ): Promise<ApiResponse> {
    try {
      const response = await axios.post(
        `${API_BASE_URL}/registrations/${registrationId}/payment`,
        { paymentStatus, paymentIntentId },
        { headers: this.getAuthHeaders() }
      )
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update payment status'
      }
    }
  }

  /**
   * Get registration statistics
   */
  async getRegistrationStats(eventId?: string): Promise<ApiResponse<any>> {
    try {
      const params = eventId ? { eventId } : {}
      const response = await axios.get(`${API_BASE_URL}/registrations/admin/stats`, {
        headers: this.getAuthHeaders(),
        params
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch registration statistics'
      }
    }
  }

  /**
   * Check if user can register for event
   */
  async checkRegistrationEligibility(eventId: string): Promise<ApiResponse<any>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/registrations/check/${eventId}`, {
        headers: this.getAuthHeaders()
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to check registration eligibility'
      }
    }
  }

  /**
   * Get registration waitlist for event (admin only)
   */
  async getEventWaitlist(eventId: string): Promise<ApiResponse<Registration[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/registrations/admin/waitlist/${eventId}`, {
        headers: this.getAuthHeaders()
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch event waitlist'
      }
    }
  }

  /**
   * Export registrations to CSV (admin only)
   */
  async exportRegistrations(eventId?: string): Promise<ApiResponse<any>> {
    try {
      const params = eventId ? { eventId } : {}
      const response = await axios.get(`${API_BASE_URL}/registrations/admin/export`, {
        headers: this.getAuthHeaders(),
        params,
        responseType: 'blob'
      })
      
      // Create download link
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', `registrations-${eventId || 'all'}-${new Date().toISOString().split('T')[0]}.csv`)
      document.body.appendChild(link)
      link.click()
      link.remove()
      
      return { success: true, message: 'Export downloaded successfully' }
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to export registrations'
      }
    }
  }
}

export const registrationService = new RegistrationService()
export default registrationService