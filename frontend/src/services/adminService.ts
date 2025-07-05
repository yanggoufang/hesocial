import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

export interface BackupInfo {
  id: string
  type: 'shutdown' | 'manual' | 'periodic'
  timestamp: string
  size: string
  schemaVersion?: string
  status?: 'latest_restored'
}

export interface SystemHealth {
  database: {
    connected: boolean
    queryTime?: number
    status: 'healthy' | 'unhealthy'
  }
  r2Sync: {
    enabled: boolean
    connectionHealthy: boolean
    lastBackup?: string
    backupCount: number
    periodicBackupsEnabled?: boolean
    periodicBackupInterval?: number
    recentBackups: BackupInfo[]
    status: 'healthy' | 'unhealthy' | 'disabled'
  }
}

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
  role: 'user' | 'admin' | 'super_admin'
  profilePicture?: string
  bio?: string
  interests: string[]
  createdAt: string
  updatedAt: string
}

export interface UserStats {
  totalUsers: number
  usersByRole: Array<{ role: string; count: number }>
  usersByMembershipTier: Array<{ membership_tier: string; count: number }>
  usersByVerificationStatus: Array<{ verification_status: string; count: number }>
  recentRegistrations: number
}

export interface AdminResponse<T = any> {
  success: boolean
  data?: T
  message?: string
  error?: string
  meta?: any
  pagination?: {
    page: number
    limit: number
    total: number
    totalPages: number
  }
}

class AdminService {
  private baseURL: string

  constructor() {
    this.baseURL = `${API_BASE_URL}/admin`
  }

  /**
   * Create a manual backup
   */
  async createBackup(): Promise<AdminResponse<{ backupId: string; type: string; timestamp: string }>> {
    try {
      const response = await axios.post(`${this.baseURL}/backup`)
      return response.data
    } catch (error: any) {
      console.error('Create backup error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to create backup'
      }
    }
  }

  /**
   * List available backups
   */
  async listBackups(limit: number = 20): Promise<AdminResponse<BackupInfo[]>> {
    try {
      const response = await axios.get(`${this.baseURL}/backups`, {
        params: { limit }
      })
      return response.data
    } catch (error: any) {
      console.error('List backups error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to list backups'
      }
    }
  }

  /**
   * Restore database from backup
   */
  async restoreBackup(backupId: string, force: boolean = false): Promise<AdminResponse> {
    try {
      const response = await axios.post(`${this.baseURL}/restore`, {
        backupId,
        force
      })
      return response.data
    } catch (error: any) {
      console.error('Restore backup error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to restore backup'
      }
    }
  }

  /**
   * Clean up old backups
   */
  async cleanupBackups(): Promise<AdminResponse> {
    try {
      const response = await axios.post(`${this.baseURL}/cleanup`)
      return response.data
    } catch (error: any) {
      console.error('Cleanup backups error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to cleanup backups'
      }
    }
  }

  /**
   * Get system health status
   */
  async getSystemHealth(): Promise<AdminResponse<SystemHealth>> {
    try {
      const [databaseHealth, r2Health] = await Promise.all([
        axios.get(`${API_BASE_URL}/health/database`),
        axios.get(`${API_BASE_URL}/health/r2-sync`)
      ])

      const systemHealth: SystemHealth = {
        database: databaseHealth.data.database || {
          connected: false,
          status: 'unhealthy'
        },
        r2Sync: r2Health.data.r2Sync || {
          enabled: false,
          connectionHealthy: false,
          backupCount: 0,
          recentBackups: [],
          status: 'disabled'
        }
      }

      return {
        success: true,
        data: systemHealth
      }
    } catch (error: any) {
      console.error('Get system health error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get system health'
      }
    }
  }

  /**
   * Get comprehensive system status
   */
  async getFullSystemStatus(): Promise<AdminResponse> {
    try {
      const response = await axios.get(`${API_BASE_URL}/health/full`)
      return response.data
    } catch (error: any) {
      console.error('Get full system status error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get system status'
      }
    }
  }

  // User Management Methods

  /**
   * Get all users with pagination and filtering
   */
  async getUsers(params: {
    page?: number
    limit?: number
    search?: string
    role?: string
    membershipTier?: string
    verificationStatus?: string
  } = {}): Promise<AdminResponse<User[]>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/users`, { params })
      return response.data
    } catch (error: any) {
      console.error('Get users error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get users'
      }
    }
  }

  /**
   * Get user by ID
   */
  async getUser(id: string): Promise<AdminResponse<User>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/users/${id}`)
      return response.data
    } catch (error: any) {
      console.error('Get user error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get user'
      }
    }
  }

  /**
   * Update user information
   */
  async updateUser(id: string, userData: Partial<User>): Promise<AdminResponse> {
    try {
      const response = await axios.put(`${API_BASE_URL}/users/${id}`, userData)
      return response.data
    } catch (error: any) {
      console.error('Update user error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update user'
      }
    }
  }

  /**
   * Delete user (super admin only)
   */
  async deleteUser(id: string): Promise<AdminResponse> {
    try {
      const response = await axios.delete(`${API_BASE_URL}/users/${id}`)
      return response.data
    } catch (error: any) {
      console.error('Delete user error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to delete user'
      }
    }
  }

  /**
   * Verify user account
   */
  async verifyUser(id: string, status: 'approved' | 'rejected'): Promise<AdminResponse> {
    try {
      const response = await axios.post(`${API_BASE_URL}/users/${id}/verify`, { status })
      return response.data
    } catch (error: any) {
      console.error('Verify user error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to verify user'
      }
    }
  }

  /**
   * Update user role (super admin only)
   */
  async updateUserRole(id: string, role: 'user' | 'admin' | 'super_admin'): Promise<AdminResponse> {
    try {
      const response = await axios.post(`${API_BASE_URL}/users/${id}/role`, { role })
      return response.data
    } catch (error: any) {
      console.error('Update user role error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update user role'
      }
    }
  }

  /**
   * Get user statistics
   */
  async getUserStats(): Promise<AdminResponse<UserStats>> {
    try {
      const response = await axios.get(`${API_BASE_URL}/users/stats/overview`)
      return response.data
    } catch (error: any) {
      console.error('Get user stats error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get user statistics'
      }
    }
  }
}

export const adminService = new AdminService()