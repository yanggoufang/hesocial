import { useAuth } from './useAuth'

interface RoleAccessResult {
  hasAccess: boolean
  userRole: string | undefined
  isAdmin: boolean
  isSuperAdmin: boolean
  isUser: boolean
}

interface MembershipAccessResult {
  hasAccess: boolean
  userTier: string | undefined
  isPlatinum: boolean
  isDiamond: boolean
  isBlackCard: boolean
}

export const useRoleAccess = (requiredRole?: 'user' | 'admin' | 'super_admin'): RoleAccessResult => {
  const { user } = useAuth()
  
  const roleHierarchy = {
    'user': 1,
    'admin': 2,
    'super_admin': 3
  }

  const userLevel = roleHierarchy[user?.role as keyof typeof roleHierarchy] || 0
  const requiredLevel = requiredRole ? roleHierarchy[requiredRole] : 1

  return {
    hasAccess: userLevel >= requiredLevel,
    userRole: user?.role,
    isAdmin: userLevel >= 2,
    isSuperAdmin: userLevel >= 3,
    isUser: userLevel >= 1
  }
}

export const useMembershipAccess = (requiredTier?: 'Platinum' | 'Diamond' | 'Black Card'): MembershipAccessResult => {
  const { user } = useAuth()
  
  const tierHierarchy = {
    'Platinum': 1,
    'Diamond': 2,
    'Black Card': 3
  }

  const userLevel = tierHierarchy[user?.membershipTier as keyof typeof tierHierarchy] || 0
  const requiredLevel = requiredTier ? tierHierarchy[requiredTier] : 1

  return {
    hasAccess: userLevel >= requiredLevel,
    userTier: user?.membershipTier,
    isPlatinum: userLevel >= 1,
    isDiamond: userLevel >= 2,
    isBlackCard: userLevel >= 3
  }
}

export const useVerificationAccess = () => {
  const { user } = useAuth()
  
  return {
    isVerified: user?.isVerified || false,
    verificationStatus: user?.verificationStatus,
    needsVerification: user?.verificationStatus === 'pending',
    isRejected: user?.verificationStatus === 'rejected'
  }
}

// Combined access checker
export const usePermissions = () => {
  const { user, isAuthenticated } = useAuth()
  const roleAccess = useRoleAccess()
  const membershipAccess = useMembershipAccess()
  const verificationAccess = useVerificationAccess()

  const can = {
    // Authentication
    access: isAuthenticated,
    
    // Role-based permissions
    viewAdmin: roleAccess.isAdmin,
    manageSuperAdmin: roleAccess.isSuperAdmin,
    manageUsers: roleAccess.isAdmin,
    manageEvents: roleAccess.isAdmin,
    manageVenues: roleAccess.isAdmin,
    manageCategories: roleAccess.isAdmin,
    manageBackups: roleAccess.isSuperAdmin,
    viewSalesData: roleAccess.isAdmin,
    
    // Membership-based permissions
    accessVVIP: membershipAccess.isDiamond && verificationAccess.isVerified,
    accessPremiumEvents: membershipAccess.isDiamond,
    accessExclusiveEvents: membershipAccess.isBlackCard,
    uploadMedia: isAuthenticated,
    
    // Verification-based permissions
    registerForEvents: verificationAccess.isVerified,
    accessPrivateContent: verificationAccess.isVerified,
    
    // Combined permissions
    fullAdminAccess: roleAccess.isSuperAdmin,
    eventManagement: roleAccess.isAdmin,
    memberFeatures: isAuthenticated && verificationAccess.isVerified
  }

  const cannot = {
    access: !isAuthenticated,
    viewAdmin: !roleAccess.isAdmin,
    manageSuperAdmin: !roleAccess.isSuperAdmin,
    accessVVIP: !membershipAccess.isDiamond || !verificationAccess.isVerified,
    registerForEvents: !verificationAccess.isVerified
  }

  return {
    user,
    can,
    cannot,
    roleAccess,
    membershipAccess,
    verificationAccess
  }
}