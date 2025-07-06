import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { motion } from 'framer-motion'
import { Shield, User, Crown, AlertTriangle } from 'lucide-react'

interface ProtectedRouteProps {
  children: React.ReactNode
  requireAuth?: boolean
  requireRole?: 'user' | 'admin' | 'super_admin'
  requireMembership?: 'Platinum' | 'Diamond' | 'Black Card'
  requireVerification?: boolean
  fallbackPath?: string
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({
  children,
  requireAuth = true,
  requireRole,
  requireMembership,
  requireVerification = false,
  fallbackPath = '/login'
}) => {
  const { user, isAuthenticated, isLoading } = useAuth()
  const location = useLocation()

  // Show loading spinner while checking auth
  if (isLoading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="luxury-glass p-8 rounded-2xl text-center"
        >
          <div className="w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
          <p className="text-luxury-platinum">Verifying access...</p>
        </motion.div>
      </div>
    )
  }

  // Check basic authentication
  if (requireAuth && !isAuthenticated) {
    return <Navigate to={fallbackPath} state={{ from: location }} replace />
  }

  // If no user data available but auth is required
  if (requireAuth && !user) {
    return <Navigate to={fallbackPath} state={{ from: location }} replace />
  }

  // Check role-based access
  if (requireRole && user) {
    const hasRequiredRole = checkRoleAccess(user.role, requireRole)
    if (!hasRequiredRole) {
      return (
        <UnauthorizedAccess 
          requiredRole={requireRole}
          userRole={user.role}
          reason="insufficient_role"
        />
      )
    }
  }

  // Check membership tier access
  if (requireMembership && user) {
    const hasRequiredMembership = checkMembershipAccess(user.membershipTier, requireMembership)
    if (!hasRequiredMembership) {
      return (
        <UnauthorizedAccess 
          requiredMembership={requireMembership}
          userMembership={user.membershipTier}
          reason="insufficient_membership"
        />
      )
    }
  }

  // Check verification status
  if (requireVerification && user && !user.isVerified) {
    return (
      <UnauthorizedAccess 
        reason="unverified_account"
        verificationStatus={user.verificationStatus}
      />
    )
  }

  // All checks passed, render the protected content
  return <>{children}</>
}

// Helper function to check role hierarchy
function checkRoleAccess(userRole: string | undefined, requireRole: string): boolean {
  const roleHierarchy = {
    'user': 1,
    'admin': 2,
    'super_admin': 3
  }

  const userLevel = roleHierarchy[userRole as keyof typeof roleHierarchy] || 0
  const requiredLevel = roleHierarchy[requireRole as keyof typeof roleHierarchy] || 999

  return userLevel >= requiredLevel
}

// Helper function to check membership tier hierarchy
function checkMembershipAccess(userTier: string | undefined, requireTier: string): boolean {
  const tierHierarchy = {
    'Platinum': 1,
    'Diamond': 2,
    'Black Card': 3
  }

  const userLevel = tierHierarchy[userTier as keyof typeof tierHierarchy] || 0
  const requiredLevel = tierHierarchy[requireTier as keyof typeof tierHierarchy] || 999

  return userLevel >= requiredLevel
}

// Unauthorized access component
interface UnauthorizedAccessProps {
  requiredRole?: string
  userRole?: string
  requiredMembership?: string
  userMembership?: string
  reason: 'insufficient_role' | 'insufficient_membership' | 'unverified_account'
  verificationStatus?: string
}

const UnauthorizedAccess: React.FC<UnauthorizedAccessProps> = ({
  requiredRole,
  userRole,
  requiredMembership,
  userMembership,
  reason,
  verificationStatus
}) => {
  const getIcon = () => {
    switch (reason) {
      case 'insufficient_role':
        return <Shield className="h-16 w-16 text-red-400" />
      case 'insufficient_membership':
        return <Crown className="h-16 w-16 text-luxury-gold" />
      case 'unverified_account':
        return <AlertTriangle className="h-16 w-16 text-yellow-400" />
      default:
        return <User className="h-16 w-16 text-luxury-platinum" />
    }
  }

  const getMessage = () => {
    switch (reason) {
      case 'insufficient_role':
        return {
          title: 'Access Denied',
          description: `This area requires ${requiredRole} access. Your current role: ${userRole || 'user'}`,
          suggestion: 'Contact an administrator to request elevated permissions.'
        }
      case 'insufficient_membership':
        return {
          title: 'Membership Upgrade Required',
          description: `This feature is exclusive to ${requiredMembership} members. Your current tier: ${userMembership || 'None'}`,
          suggestion: 'Upgrade your membership to access premium features.'
        }
      case 'unverified_account':
        return {
          title: 'Account Verification Required',
          description: `Your account must be verified to access this feature. Status: ${verificationStatus || 'pending'}`,
          suggestion: 'Complete your profile verification or contact support for assistance.'
        }
      default:
        return {
          title: 'Access Restricted',
          description: 'You do not have permission to access this area.',
          suggestion: 'Please contact support if you believe this is an error.'
        }
    }
  }

  const message = getMessage()

  return (
    <div className="min-h-screen bg-luxury-midnight-black flex items-center justify-center p-4">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="luxury-glass p-8 rounded-2xl text-center max-w-md w-full"
      >
        <div className="mb-6">
          {getIcon()}
        </div>
        
        <h1 className="text-2xl font-luxury font-bold text-luxury-gold mb-4">
          {message.title}
        </h1>
        
        <p className="text-luxury-platinum/80 mb-6 leading-relaxed">
          {message.description}
        </p>
        
        <div className="space-y-4">
          <p className="text-luxury-platinum/60 text-sm">
            {message.suggestion}
          </p>
          
          <div className="flex space-x-3">
            <button
              onClick={() => window.history.back()}
              className="flex-1 px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors"
            >
              Go Back
            </button>
            <button
              onClick={() => window.location.href = '/'}
              className="flex-1 luxury-button py-2"
            >
              Home
            </button>
          </div>
        </div>
      </motion.div>
    </div>
  )
}

export default ProtectedRoute