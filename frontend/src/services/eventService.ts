import axios from 'axios'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

// Configure axios defaults
axios.defaults.baseURL = API_URL
axios.defaults.timeout = 10000

// Add request interceptor to include auth token
axios.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor for error handling
axios.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

export interface Event {
  id: string
  title: string
  slug: string
  description: string
  detailed_description?: string
  category_id: string
  venue_id: string
  organizer_id: string
  start_datetime: string
  end_datetime: string
  timezone: string
  capacity_min: number
  capacity_max: number
  current_registrations: number
  price_platinum: number
  price_diamond: number
  price_black_card: number
  currency: string
  status: 'draft' | 'pending_review' | 'approved' | 'published' | 'full' | 'completed' | 'cancelled' | 'archived'
  approval_status: 'pending' | 'approved' | 'rejected'
  approved_by?: string
  approved_at?: string
  required_membership_tiers: string[]
  required_verification: boolean
  age_restriction?: { min?: number; max?: number }
  dress_code?: string
  language: string
  special_requirements?: string
  inclusions: string[]
  exclusions: string[]
  registration_opens_at?: string
  registration_closes_at?: string
  cancellation_deadline?: string
  waitlist_enabled: boolean
  auto_approval: boolean
  meta_title?: string
  meta_description?: string
  featured_image?: string
  gallery_images: string[]
  internal_notes?: string
  cost_breakdown?: Record<string, any>
  profit_margin?: number
  created_at: string
  updated_at: string
  published_at?: string
  archived_at?: string
  // Joined data
  category_name?: string
  category_slug?: string
  category_icon?: string
  category_color?: string
  venue_name?: string
  venue_address?: string
  venue_city?: string
  venue_type?: string
  venue_capacity_max?: number
  organizer_name?: string
  approved_by_name?: string
  registration_stats?: {
    total_registrations: number
    confirmed_registrations: number
    waitlisted_registrations: number
    pending_registrations: number
  }
  waitlist_count?: number
}

export interface Venue {
  id: string
  name: string
  description?: string
  address: string
  city: string
  district?: string
  postal_code?: string
  country: string
  venue_type: 'restaurant' | 'yacht' | 'gallery' | 'private_residence' | 'hotel' | 'outdoor'
  capacity_min: number
  capacity_max: number
  price_tier: 'premium' | 'luxury' | 'ultra_luxury'
  amenities: string[]
  contact_name?: string
  contact_phone?: string
  contact_email?: string
  booking_requirements?: string
  cancellation_policy?: string
  is_active: boolean
  images: string[]
  location_coordinates?: { lat: number; lng: number }
  created_at: string
  updated_at: string
}

export interface EventCategory {
  id: string
  name: string
  slug: string
  description?: string
  icon?: string
  color?: string
  target_membership_tiers: string[]
  typical_duration_hours: number
  typical_capacity: { min: number; max: number }
  is_active: boolean
  sort_order: number
  created_at: string
  updated_at: string
}

export interface EventFilters {
  page?: number
  limit?: number
  search?: string
  category?: string
  status?: string
  venue?: string
  startDate?: string
  endDate?: string
  membershipTier?: string
}

export interface VenueFilters {
  page?: number
  limit?: number
  search?: string
  venueType?: string
  city?: string
  isActive?: string
}

export interface EventResponse<T> {
  success: boolean
  data: T
  pagination?: {
    page: number
    limit: number
    total: number
    totalPages: number
  }
  error?: string
}

export interface CreateEventData {
  title: string
  description: string
  detailedDescription?: string
  categoryId: string
  venueId: string
  startDatetime: string
  endDatetime: string
  timezone?: string
  capacityMin?: number
  capacityMax: number
  pricePlatinum: number
  priceDiamond: number
  priceBlackCard: number
  currency?: string
  requiredMembershipTiers: string[]
  requiredVerification?: boolean
  ageRestriction?: { min?: number; max?: number }
  dressCode?: string
  language?: string
  specialRequirements?: string
  inclusions: string[]
  exclusions: string[]
  registrationOpensAt?: string
  registrationClosesAt?: string
  cancellationDeadline?: string
  waitlistEnabled?: boolean
  autoApproval?: boolean
  metaTitle?: string
  metaDescription?: string
  featuredImage?: string
  internalNotes?: string
  costBreakdown?: Record<string, any>
  profitMargin?: number
}

export interface CreateVenueData {
  name: string
  description?: string
  address: string
  city: string
  district?: string
  postalCode?: string
  country?: string
  venueType: string
  capacityMin?: number
  capacityMax: number
  priceTier?: string
  amenities: string[]
  contactName?: string
  contactPhone?: string
  contactEmail?: string
  bookingRequirements?: string
  cancellationPolicy?: string
  images: string[]
  locationCoordinates?: { lat: number; lng: number }
}

export interface CreateCategoryData {
  name: string
  slug?: string
  description?: string
  icon?: string
  color?: string
  targetMembershipTiers: string[]
  typicalDurationHours?: number
  typicalCapacity?: { min: number; max: number }
  sortOrder?: number
}

class EventService {
  // Event Management
  async getEvents(filters: EventFilters = {}): Promise<EventResponse<Event[]>> {
    try {
      const params = new URLSearchParams()
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== '') {
          params.append(key, value.toString())
        }
      })
      
      const response = await axios.get(`/events?${params.toString()}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch events')
    }
  }

  async getEvent(id: string): Promise<EventResponse<Event>> {
    try {
      const response = await axios.get(`/events/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch event')
    }
  }

  async createEvent(eventData: CreateEventData): Promise<EventResponse<{ eventId: string; slug: string }>> {
    try {
      const response = await axios.post('/events', eventData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to create event')
    }
  }

  async updateEvent(id: string, eventData: Partial<CreateEventData>): Promise<EventResponse<void>> {
    try {
      const response = await axios.put(`/events/${id}`, eventData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to update event')
    }
  }

  async deleteEvent(id: string): Promise<EventResponse<void>> {
    try {
      const response = await axios.delete(`/events/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to delete event')
    }
  }

  async publishEvent(id: string): Promise<EventResponse<void>> {
    try {
      const response = await axios.post(`/events/${id}/publish`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to publish event')
    }
  }

  async approveEvent(id: string, approved: boolean): Promise<EventResponse<void>> {
    try {
      const response = await axios.post(`/events/${id}/approve`, { approved })
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to approve/reject event')
    }
  }

  // Venue Management
  async getVenues(filters: VenueFilters = {}): Promise<EventResponse<Venue[]>> {
    try {
      const params = new URLSearchParams()
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== '') {
          params.append(key, value.toString())
        }
      })
      
      const response = await axios.get(`/venues?${params.toString()}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch venues')
    }
  }

  async getVenue(id: string): Promise<EventResponse<Venue>> {
    try {
      const response = await axios.get(`/venues/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch venue')
    }
  }

  async createVenue(venueData: CreateVenueData): Promise<EventResponse<{ venueId: string }>> {
    try {
      const response = await axios.post('/venues', venueData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to create venue')
    }
  }

  async updateVenue(id: string, venueData: Partial<CreateVenueData>): Promise<EventResponse<void>> {
    try {
      const response = await axios.put(`/venues/${id}`, venueData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to update venue')
    }
  }

  async deleteVenue(id: string): Promise<EventResponse<void>> {
    try {
      const response = await axios.delete(`/venues/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to delete venue')
    }
  }

  // Category Management
  async getCategories(): Promise<EventResponse<EventCategory[]>> {
    try {
      const response = await axios.get('/categories')
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch categories')
    }
  }

  async getCategory(id: string): Promise<EventResponse<EventCategory>> {
    try {
      const response = await axios.get(`/categories/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch category')
    }
  }

  async createCategory(categoryData: CreateCategoryData): Promise<EventResponse<{ categoryId: string; slug: string }>> {
    try {
      const response = await axios.post('/categories', categoryData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to create category')
    }
  }

  async updateCategory(id: string, categoryData: Partial<CreateCategoryData>): Promise<EventResponse<void>> {
    try {
      const response = await axios.put(`/categories/${id}`, categoryData)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to update category')
    }
  }

  async deleteCategory(id: string): Promise<EventResponse<void>> {
    try {
      const response = await axios.delete(`/categories/${id}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to delete category')
    }
  }

  // Event Registration
  async registerForEvent(eventId: string, registrationData?: any): Promise<EventResponse<{ registrationId: string }>> {
    try {
      const response = await axios.post(`/registrations/events/${eventId}`, registrationData || {})
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to register for event')
    }
  }

  async getUserRegistrations(): Promise<EventResponse<any[]>> {
    try {
      const response = await axios.get('/registrations/user')
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to fetch user registrations')
    }
  }

  async cancelRegistration(registrationId: string): Promise<EventResponse<void>> {
    try {
      const response = await axios.delete(`/registrations/${registrationId}`)
      return response.data
    } catch (error: any) {
      throw new Error(error.response?.data?.error || 'Failed to cancel registration')
    }
  }
}

export const eventService = new EventService()
export default eventService