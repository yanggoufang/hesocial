import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

export interface SalesLead {
  id: string
  firstName: string
  lastName: string
  email: string
  phone?: string
  company?: string
  position?: string
  leadScore: number
  annualIncome: number
  netWorth: number
  source: 'website' | 'referral' | 'event' | 'cold_call' | 'linkedin' | 'advertisement'
  status: 'new' | 'qualified' | 'contacted' | 'nurturing' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  assignedTo?: string
  interests: string[]
  lastContactDate?: string
  nextFollowUpDate?: string
  notes?: string
  createdAt: string
  updatedAt: string
}

export interface SalesOpportunity {
  id: string
  leadId: string
  name: string
  description?: string
  stage: 'qualification' | 'needs_analysis' | 'proposal' | 'negotiation' | 'closed_won' | 'closed_lost'
  probability: number
  value: number
  membershipTier: 'Platinum' | 'Diamond' | 'Black Card'
  expectedCloseDate: string
  actualCloseDate?: string
  assignedTo?: string
  notes?: string
  createdAt: string
  updatedAt: string
  lead?: {
    firstName: string
    lastName: string
    email: string
  }
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

export interface SalesFilters {
  page?: number
  limit?: number
  search?: string
  status?: string
  source?: string
  stage?: string
  assignedTo?: string
}

interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

const authHeaders = () => {
  const token = localStorage.getItem('hesocial_token')
  return {
    'Content-Type': 'application/json',
    ...(token ? { Authorization: `Bearer ${token}` } : {})
  }
}

const parseInterests = (value: unknown): string[] => {
  if (Array.isArray(value)) return value.map(String)
  if (typeof value !== 'string') return []
  try {
    const parsed = JSON.parse(value)
    return Array.isArray(parsed) ? parsed.map(String) : []
  } catch {
    return []
  }
}

const mapLead = (row: any): SalesLead => ({
  id: String(row.id),
  firstName: row.firstName ?? row.first_name ?? '',
  lastName: row.lastName ?? row.last_name ?? '',
  email: row.email ?? '',
  phone: row.phone ?? undefined,
  company: row.company ?? undefined,
  position: row.position ?? row.job_title ?? undefined,
  leadScore: Number(row.leadScore ?? row.lead_score) || 0,
  annualIncome: Number(row.annualIncome ?? row.annual_income) || 0,
  netWorth: Number(row.netWorth ?? row.net_worth) || 0,
  source: row.source ?? 'website',
  status: row.status ?? 'new',
  assignedTo: row.assignedTo ?? row.assigned_to ?? undefined,
  interests: parseInterests(row.interests),
  lastContactDate: row.lastContactDate ?? row.last_contact_date ?? undefined,
  nextFollowUpDate: row.nextFollowUpDate ?? row.next_follow_up_date ?? undefined,
  notes: row.notes ?? undefined,
  createdAt: row.createdAt ?? row.created_at ?? '',
  updatedAt: row.updatedAt ?? row.updated_at ?? ''
})

const mapOpportunity = (row: any): SalesOpportunity => ({
  id: String(row.id),
  leadId: String(row.leadId ?? row.lead_id ?? ''),
  name: row.name ?? '',
  description: row.description ?? undefined,
  stage: row.stage ?? 'qualification',
  probability: Number(row.probability) || 0,
  value: Number(row.value) || 0,
  membershipTier: row.membershipTier ?? row.membership_tier ?? 'Platinum',
  expectedCloseDate: row.expectedCloseDate ?? row.expected_close_date ?? '',
  actualCloseDate: row.actualCloseDate ?? row.actual_close_date ?? undefined,
  assignedTo: row.assignedTo ?? row.assigned_to ?? undefined,
  notes: row.notes ?? undefined,
  createdAt: row.createdAt ?? row.created_at ?? '',
  updatedAt: row.updatedAt ?? row.updated_at ?? '',
  lead: row.lead ?? {
    firstName: row.lead_first_name ?? '',
    lastName: row.lead_last_name ?? '',
    email: row.lead_email ?? ''
  }
})

class SalesService {
  async getLeads(filters: SalesFilters = {}): Promise<ApiResponse<SalesLead[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/sales/leads`, {
        headers: authHeaders(),
        params: filters
      })

      return {
        ...response.data,
        data: (response.data.data || []).map(mapLead)
      }
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch sales leads'
      }
    }
  }

  async getOpportunities(filters: SalesFilters = {}): Promise<ApiResponse<SalesOpportunity[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/sales/opportunities`, {
        headers: authHeaders(),
        params: filters
      })

      return {
        ...response.data,
        data: (response.data.data || []).map(mapOpportunity)
      }
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch sales opportunities'
      }
    }
  }

  async getMetrics(): Promise<ApiResponse<SalesMetrics>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/sales/metrics`, {
        headers: authHeaders()
      })
      return response.data
    } catch (error: any) {
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to fetch sales metrics'
      }
    }
  }
}

export const salesService = new SalesService()
export default salesService
