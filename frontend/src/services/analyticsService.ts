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

export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  message?: string
}

export interface AnalyticsOverview {
  total_events: number
  published_events: number
  completed_events: number
  cancelled_events: number
  avg_registrations: number
  total_registrations: number
  estimated_revenue: number
}

export interface EventTrend {
  month: string
  events_created: number
  total_registrations: number
  avg_fill_rate: number
}

export interface TopEvent {
  id: string
  title: string
  current_registrations: number
  capacity_max: number
  fill_rate: number
  revenue: number
  status: string
}

export interface CategoryPerformance {
  category_name: string
  event_count: number
  total_registrations: number
  avg_fill_rate: number
  total_revenue: number
}

export interface EventPerformanceData {
  event: {
    id: string
    title: string
    description: string
    start_datetime: string
    current_registrations: number
    capacity_max: number
    fill_rate: number
    current_revenue: number
    potential_revenue: number
    category_name: string
    venue_name: string
  }
  registrationTimeline: Array<{
    date: string
    registrations: number
    cumulative_registrations: number
  }>
  membershipBreakdown: Array<{
    membership_tier: string
    count: number
    percentage: number
  }>
  statusBreakdown: Array<{
    status: string
    count: number
  }>
}

export interface RevenueData {
  monthlyRevenue: Array<{
    month: string
    revenue: number
    event_count: number
    total_registrations: number
  }>
  categoryRevenue: Array<{
    category: string
    revenue: number
    event_count: number
    avg_revenue_per_event: number
  }>
  tierRevenue: Array<{
    membership_tier: string
    registration_count: number
    total_revenue: number
  }>
}

export interface EngagementData {
  engagement: Array<{
    membership_tier: string
    total_members: number
    active_members: number
    engagement_rate: number
    avg_events_per_member: number
  }>
  topMembers: Array<{
    first_name: string
    last_name: string
    membership_tier: string
    events_attended: number
    total_spent: number
  }>
  retention: Array<{
    cohort_month: string
    cohort_size: number
    active_this_month: number
    retention_rate: number
  }>
}

class AnalyticsService {
  // Get events overview analytics
  async getEventsOverview(): Promise<ApiResponse<{
    overview: AnalyticsOverview
    trends: EventTrend[]
    topEvents: TopEvent[]
    categoryPerformance: CategoryPerformance[]
  }>> {
    try {
      const response = await axios.get('/analytics/events/overview')
      return response.data
    } catch (error: any) {
      console.error('Analytics overview error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch analytics overview'
      }
    }
  }

  // Get individual event performance
  async getEventPerformance(eventId: string): Promise<ApiResponse<EventPerformanceData>> {
    try {
      const response = await axios.get(`/analytics/events/${eventId}/performance`)
      return response.data
    } catch (error: any) {
      console.error('Event performance error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch event performance data'
      }
    }
  }

  // Get revenue analytics
  async getRevenueAnalytics(): Promise<ApiResponse<RevenueData>> {
    try {
      const response = await axios.get('/analytics/revenue/events')
      return response.data
    } catch (error: any) {
      console.error('Revenue analytics error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch revenue analytics'
      }
    }
  }

  // Get member engagement analytics
  async getMemberEngagement(): Promise<ApiResponse<EngagementData>> {
    try {
      const response = await axios.get('/analytics/engagement/members')
      return response.data
    } catch (error: any) {
      console.error('Member engagement error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch member engagement data'
      }
    }
  }

  // Export analytics data
  async exportAnalyticsData(format: 'csv' | 'excel' = 'csv'): Promise<ApiResponse<{ downloadUrl: string }>> {
    try {
      const response = await axios.post('/analytics/export', { format })
      return response.data
    } catch (error: any) {
      console.error('Export analytics error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to export analytics data'
      }
    }
  }

  // Get real-time analytics (for dashboard auto-refresh)
  async getRealTimeMetrics(): Promise<ApiResponse<{
    activeUsers: number
    ongoingEvents: number
    todayRegistrations: number
    systemHealth: string
  }>> {
    try {
      const response = await axios.get('/analytics/realtime')
      return response.data
    } catch (error: any) {
      console.error('Real-time metrics error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch real-time metrics'
      }
    }
  }

  // Get custom date range analytics
  async getCustomRangeAnalytics(
    startDate: string,
    endDate: string,
    metrics: string[] = ['events', 'revenue', 'registrations']
  ): Promise<ApiResponse<any>> {
    try {
      const response = await axios.post('/analytics/custom-range', {
        startDate,
        endDate,
        metrics
      })
      return response.data
    } catch (error: any) {
      console.error('Custom range analytics error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch custom range analytics'
      }
    }
  }

  // Get comparative analytics (month-over-month, year-over-year)
  async getComparativeAnalytics(
    period: 'month' | 'quarter' | 'year',
    comparison: 'previous' | 'year-ago'
  ): Promise<ApiResponse<{
    current: any
    previous: any
    growth: {
      events: number
      revenue: number
      registrations: number
      engagement: number
    }
  }>> {
    try {
      const response = await axios.get(`/analytics/comparative`, {
        params: { period, comparison }
      })
      return response.data
    } catch (error: any) {
      console.error('Comparative analytics error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch comparative analytics'
      }
    }
  }
}

export const analyticsService = new AnalyticsService()
export default analyticsService
