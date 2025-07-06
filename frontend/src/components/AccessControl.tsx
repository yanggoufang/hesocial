import React from 'react'
import { useAuth } from '../hooks/useAuth'
import { usePermissions } from '../hooks/useRoleAccess'

interface AccessControlProps {
  children: React.ReactNode
  // Authentication requirements
  requireAuth?: boolean
  
  // Role requirements
  requireRole?: 'user' | 'admin' | 'super_admin'
  allowRoles?: ('user' | 'admin' | 'super_admin')[]
  
  // Membership requirements
  requireMembership?: 'Platinum' | 'Diamond' | 'Black Card'
  allowMemberships?: ('Platinum' | 'Diamond' | 'Black Card')[]
  
  // Verification requirements
  requireVerification?: boolean
  
  // Custom condition
  condition?: boolean
  
  // Fallback content when access is denied
  fallback?: React.ReactNode
  
  // Show/hide instead of render/not render
  mode?: 'render' | 'visibility'
}

const AccessControl: React.FC<AccessControlProps> = ({
  children,
  requireAuth = false,
  requireRole,
  allowRoles,
  requireMembership,
  allowMemberships,
  requireVerification = false,
  condition = true,
  fallback = null,
  mode = 'render'
}) => {
  const { isAuthenticated, user } = useAuth()
  const { can, roleAccess, membershipAccess, verificationAccess } = usePermissions()

  // Check all access conditions
  const hasAccess = (() => {
    // Custom condition check
    if (!condition) return false

    // Authentication check
    if (requireAuth && !isAuthenticated) return false

    // Role-based check
    if (requireRole) {
      if (!roleAccess.hasAccess || !checkRoleRequirement(user?.role, requireRole)) {
        return false
      }
    }

    // Allow roles check (any of the specified roles)
    if (allowRoles && allowRoles.length > 0) {
      if (!user?.role || !allowRoles.includes(user.role as any)) {
        return false
      }
    }

    // Membership-based check
    if (requireMembership) {
      if (!membershipAccess.hasAccess || !checkMembershipRequirement(user?.membershipTier, requireMembership)) {
        return false
      }
    }

    // Allow memberships check (any of the specified tiers)
    if (allowMemberships && allowMemberships.length > 0) {
      if (!user?.membershipTier || !allowMemberships.includes(user.membershipTier as any)) {
        return false
      }
    }

    // Verification check
    if (requireVerification && !verificationAccess.isVerified) {
      return false
    }

    return true
  })()

  // Handle different display modes
  if (mode === 'visibility') {
    return (
      <div style={{ display: hasAccess ? 'block' : 'none' }}>
        {children}
      </div>
    )
  }

  // Default render mode
  if (!hasAccess) {
    return <>{fallback}</>
  }

  return <>{children}</>
}

// Helper functions
function checkRoleRequirement(userRole: string | undefined, requiredRole: string): boolean {
  const roleHierarchy = {
    'user': 1,
    'admin': 2,
    'super_admin': 3
  }

  const userLevel = roleHierarchy[userRole as keyof typeof roleHierarchy] || 0
  const requiredLevel = roleHierarchy[requiredRole as keyof typeof roleHierarchy] || 999

  return userLevel >= requiredLevel
}

function checkMembershipRequirement(userTier: string | undefined, requiredTier: string): boolean {
  const tierHierarchy = {
    'Platinum': 1,
    'Diamond': 2,
    'Black Card': 3
  }

  const userLevel = tierHierarchy[userTier as keyof typeof tierHierarchy] || 0
  const requiredLevel = tierHierarchy[requiredTier as keyof typeof tierHierarchy] || 999

  return userLevel >= requiredLevel
}

// Convenient component variants
export const AdminOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireRole="admin" fallback={fallback}>
    {children}
  </AccessControl>
)

export const SuperAdminOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireRole="super_admin" fallback={fallback}>
    {children}
  </AccessControl>
)

export const AuthenticatedOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireAuth={true} fallback={fallback}>
    {children}
  </AccessControl>
)

export const VerifiedOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireAuth={true} requireVerification={true} fallback={fallback}>
    {children}
  </AccessControl>
)

export const DiamondOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireAuth={true} requireMembership="Diamond" fallback={fallback}>
    {children}
  </AccessControl>
)

export const BlackCardOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl requireAuth={true} requireMembership="Black Card" fallback={fallback}>
    {children}
  </AccessControl>
)

export const VVIPOnly: React.FC<{ children: React.ReactNode; fallback?: React.ReactNode }> = ({ 
  children, 
  fallback 
}) => (
  <AccessControl 
    requireAuth={true} 
    requireMembership="Diamond" 
    requireVerification={true} 
    fallback={fallback}
  >
    {children}
  </AccessControl>
)

export default AccessControl