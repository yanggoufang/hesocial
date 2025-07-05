import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:5000/api'

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
}

export interface AuthResponse {
  success: boolean
  data?: {
    user: User
    token: string
  }
  message?: string
  error?: string
}

export interface LoginCredentials {
  email: string
  password: string
}

export interface RegisterData {
  email: string
  password: string
  firstName: string
  lastName: string
  age: number
  profession: string
  annualIncome: number
  netWorth: number
  bio?: string
  interests?: string[]
}

class AuthService {
  private baseURL: string
  private token: string | null = null

  constructor() {
    this.baseURL = `${API_BASE_URL}/auth`
    this.token = localStorage.getItem('hesocial_token')
    
    // Set up axios interceptor to include token in all requests
    axios.interceptors.request.use(
      (config) => {
        if (this.token) {
          config.headers.Authorization = `Bearer ${this.token}`
        }
        return config
      },
      (error) => {
        return Promise.reject(error)
      }
    )

    // Set up response interceptor for token expiration
    axios.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          this.logout()
        }
        return Promise.reject(error)
      }
    )
  }

  async login(credentials: LoginCredentials): Promise<AuthResponse> {
    try {
      const response = await axios.post(`${this.baseURL}/login`, credentials)
      
      if (response.data.success && response.data.data) {
        this.setToken(response.data.data.token)
      }
      
      return response.data
    } catch (error: any) {
      console.error('Login error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Login failed'
      }
    }
  }

  async register(userData: RegisterData): Promise<AuthResponse> {
    try {
      const response = await axios.post(`${this.baseURL}/register`, userData)
      
      if (response.data.success && response.data.data) {
        this.setToken(response.data.data.token)
      }
      
      return response.data
    } catch (error: any) {
      console.error('Registration error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Registration failed'
      }
    }
  }

  async getProfile(): Promise<{ success: boolean; data?: { user: User }; error?: string }> {
    try {
      if (!this.token) {
        return { success: false, error: 'No token available' }
      }

      const response = await axios.get(`${this.baseURL}/profile`)
      return response.data
    } catch (error: any) {
      console.error('Get profile error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to get profile'
      }
    }
  }

  async updateProfile(updates: Partial<User>): Promise<{ success: boolean; data?: { user: User }; error?: string }> {
    try {
      const response = await axios.put(`${this.baseURL}/profile`, updates)
      return response.data
    } catch (error: any) {
      console.error('Update profile error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to update profile'
      }
    }
  }

  async refreshToken(): Promise<{ success: boolean; data?: { token: string }; error?: string }> {
    try {
      const response = await axios.post(`${this.baseURL}/refresh`)
      
      if (response.data.success && response.data.data) {
        this.setToken(response.data.data.token)
      }
      
      return response.data
    } catch (error: any) {
      console.error('Refresh token error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Failed to refresh token'
      }
    }
  }

  async validateToken(): Promise<{ success: boolean; data?: { user: User; valid: boolean }; error?: string }> {
    try {
      if (!this.token) {
        return { success: false, error: 'No token available' }
      }

      const response = await axios.get(`${this.baseURL}/validate`)
      return response.data
    } catch (error: any) {
      console.error('Validate token error:', error)
      return {
        success: false,
        error: error.response?.data?.error || 'Token validation failed'
      }
    }
  }

  // Google OAuth
  initiateGoogleLogin(): void {
    window.location.href = `${this.baseURL}/google`
  }

  // Handle OAuth callback token (from URL parameters)
  handleOAuthCallback(): string | null {
    const urlParams = new URLSearchParams(window.location.search)
    const token = urlParams.get('token')
    
    if (token) {
      this.setToken(token)
      // Clean up URL
      window.history.replaceState({}, document.title, window.location.pathname)
      return token
    }
    
    return null
  }

  logout(): void {
    this.token = null
    localStorage.removeItem('hesocial_token')
    
    // Make logout API call (fire and forget)
    axios.post(`${this.baseURL}/logout`).catch(() => {
      // Silent fail - token is already removed locally
    })
  }

  getToken(): string | null {
    return this.token
  }

  isAuthenticated(): boolean {
    return !!this.token
  }

  private setToken(token: string): void {
    this.token = token
    localStorage.setItem('hesocial_token', token)
  }
}

export const authService = new AuthService()