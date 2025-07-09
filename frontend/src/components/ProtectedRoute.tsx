import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'
import { motion } from 'framer-motion'
import { Shield, User, Crown, MessageCircle } from 'lucide-react'

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
          <p className="text-luxury-platinum">驗證存取權限中...</p>
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
        return <MessageCircle className="h-16 w-16 text-luxury-gold" />
      default:
        return <User className="h-16 w-16 text-luxury-platinum" />
    }
  }

  const getMessage = () => {
    switch (reason) {
      case 'insufficient_role':
        return {
          title: '存取被拒',
          description: `此區域需要 ${requiredRole === 'admin' ? '管理員' : requiredRole === 'super_admin' ? '超級管理員' : requiredRole} 權限。您目前的角色：${userRole === 'admin' ? '管理員' : userRole === 'super_admin' ? '超級管理員' : userRole === 'user' ? '使用者' : userRole || '使用者'}`,
          suggestion: '請聯繫管理員申請更高權限。'
        }
      case 'insufficient_membership':
        return {
          title: '需要升級會員',
          description: `此功能僅限 ${requiredMembership === 'Platinum' ? '白金卡' : requiredMembership === 'Diamond' ? '鑽石卡' : requiredMembership === 'Black Card' ? '黑卡' : requiredMembership} 會員使用。您目前的等級：${userMembership === 'Platinum' ? '白金卡' : userMembership === 'Diamond' ? '鑽石卡' : userMembership === 'Black Card' ? '黑卡' : userMembership || '無'}`,
          suggestion: '請升級您的會員資格以使用高級功能。'
        }
      case 'unverified_account':
        return {
          title: '需要帳戶驗證',
          description: `您的帳戶必須經過驗證才能使用此功能。狀態：${verificationStatus === 'pending' ? '待審核' : verificationStatus === 'approved' ? '已通過' : verificationStatus === 'rejected' ? '已拒絕' : verificationStatus || '待審核'}`,
          suggestion: '請完成您的個人資料驗證，或聯繫客服尋求協助。'
        }
      default:
        return {
          title: '存取受限',
          description: '您沒有權限存取此區域。',
          suggestion: '如果您認為這是錯誤，請聯繫客服。'
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
              返回
            </button>
            <button
              onClick={() => window.location.href = '/'}
              className="flex-1 luxury-button py-2"
            >
              首頁
            </button>
          </div>
        </div>
      </motion.div>
    </div>
  )
}

export default ProtectedRoute