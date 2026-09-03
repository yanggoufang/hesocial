import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { authService, User, LoginCredentials, RegisterData } from '../services/authService'

interface AuthContextType {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (credentials: LoginCredentials) => Promise<{ success: boolean; error?: string }>
  register: (userData: RegisterData) => Promise<{ success: boolean; error?: string }>
  logout: () => void
  updateProfile: (updates: Partial<User>) => Promise<{ success: boolean; error?: string }>
  refreshToken: () => Promise<void>
  loginWithGoogle: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

interface AuthProviderProps {
  children: ReactNode
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  const isAuthenticated = !!user

  // Initialize auth state on app load
  useEffect(() => {
    initializeAuth()
  }, [])

  // Handle OAuth callback tokens
  useEffect(() => {
    const token = authService.handleOAuthCallback()
    if (token) {
      // OAuth login successful, fetch user profile
      loadUserProfile()
    }
  }, [])

  const initializeAuth = async () => {
    try {
      setIsLoading(true)
      
      if (authService.isAuthenticated()) {
        // Validate existing token and load user data
        const result = await authService.validateToken()
        
        if (result.success && result.data) {
          setUser(result.data.user)
        } else {
          // Invalid token, clear it
          authService.logout()
          setUser(null)
        }
      }
    } catch (error) {
      console.error('Auth initialization error:', error)
      authService.logout()
      setUser(null)
    } finally {
      setIsLoading(false)
    }
  }

  const loadUserProfile = async () => {
    try {
      const result = await authService.getProfile()
      
      if (result.success && result.data) {
        setUser(result.data.user)
      }
    } catch (error) {
      console.error('Load user profile error:', error)
    }
  }

  const login = async (credentials: LoginCredentials): Promise<{ success: boolean; error?: string }> => {
    try {
      setIsLoading(true)
      
      const result = await authService.login(credentials)
      
      if (result.success && result.data) {
        setUser(result.data.user)
        return { success: true }
      } else {
        return { success: false, error: result.error }
      }
    } catch (error) {
      console.error('Login error:', error)
      return { success: false, error: 'Login failed' }
    } finally {
      setIsLoading(false)
    }
  }

  const register = async (userData: RegisterData): Promise<{ success: boolean; error?: string }> => {
    try {
      setIsLoading(true)
      
      const result = await authService.register(userData)
      
      if (result.success && result.data) {
        setUser(result.data.user)
        return { success: true }
      } else {
        return { success: false, error: result.error }
      }
    } catch (error) {
      console.error('Registration error:', error)
      return { success: false, error: 'Registration failed' }
    } finally {
      setIsLoading(false)
    }
  }

  const logout = () => {
    authService.logout()
    setUser(null)
  }

  const updateProfile = async (updates: Partial<User>): Promise<{ success: boolean; error?: string }> => {
    try {
      const result = await authService.updateProfile(updates)
      
      if (result.success && result.data) {
        setUser(result.data.user)
        return { success: true }
      } else {
        return { success: false, error: result.error }
      }
    } catch (error) {
      console.error('Update profile error:', error)
      return { success: false, error: 'Failed to update profile' }
    }
  }

  const refreshToken = async () => {
    try {
      const result = await authService.refreshToken()
      
      if (!result.success) {
        // Refresh failed, logout user
        logout()
      }
    } catch (error) {
      console.error('Refresh token error:', error)
      logout()
    }
  }

  const loginWithGoogle = () => {
    authService.initiateGoogleLogin()
  }

  const value: AuthContextType = {
    user,
    isLoading,
    isAuthenticated,
    login,
    register,
    logout,
    updateProfile,
    refreshToken,
    loginWithGoogle
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}

export const useAuth = (): AuthContextType => {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}