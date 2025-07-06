import React from 'react'
import { motion } from 'framer-motion'
import { Shield, Crown, User, Settings } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { usePermissions } from '../hooks/useRoleAccess'

interface AdminIndicatorProps {
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right'
  showMembership?: boolean
  showRole?: boolean
  compact?: boolean
}

const AdminIndicator: React.FC<AdminIndicatorProps> = ({
  position = 'bottom-right',
  showMembership = true,
  showRole = true,
  compact = false
}) => {
  const { user, isAuthenticated } = useAuth()
  const { can } = usePermissions()

  if (!isAuthenticated || !user) return null

  const getPositionClasses = () => {
    switch (position) {
      case 'top-left':
        return 'top-4 left-4'
      case 'top-right':
        return 'top-4 right-4'
      case 'bottom-left':
        return 'bottom-4 left-4'
      case 'bottom-right':
        return 'bottom-4 right-4'
      default:
        return 'bottom-4 right-4'
    }
  }

  const getRoleIcon = () => {
    if (can.manageSuperAdmin) return <Shield className="h-4 w-4 text-red-400" />
    if (can.viewAdmin) return <Settings className="h-4 w-4 text-luxury-gold" />
    return <User className="h-4 w-4 text-luxury-platinum" />
  }

  const getMembershipIcon = () => {
    switch (user.membershipTier) {
      case 'Black Card':
        return <Crown className="h-4 w-4 text-purple-400" />
      case 'Diamond':
        return <Crown className="h-4 w-4 text-blue-400" />
      case 'Platinum':
        return <Crown className="h-4 w-4 text-gray-400" />
      default:
        return null
    }
  }

  const getRoleColor = () => {
    if (can.manageSuperAdmin) return 'border-red-400/30 bg-red-400/10'
    if (can.viewAdmin) return 'border-luxury-gold/30 bg-luxury-gold/10'
    return 'border-luxury-platinum/30 bg-luxury-platinum/10'
  }

  if (compact) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        className={`fixed ${getPositionClasses()} z-50`}
      >
        <div className={`flex items-center space-x-1 px-2 py-1 rounded-full border ${getRoleColor()} backdrop-blur-sm`}>
          {showRole && getRoleIcon()}
          {showMembership && getMembershipIcon()}
        </div>
      </motion.div>
    )
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className={`fixed ${getPositionClasses()} z-50`}
    >
      <div className={`luxury-glass px-3 py-2 rounded-lg border ${getRoleColor()}`}>
        <div className="flex items-center space-x-3 text-sm">
          {showRole && (
            <div className="flex items-center space-x-1">
              {getRoleIcon()}
              <span className="text-luxury-platinum font-medium">
                {user.role === 'super_admin' ? 'Super Admin' : 
                 user.role === 'admin' ? 'Admin' : 'User'}
              </span>
            </div>
          )}
          
          {showMembership && (
            <div className="flex items-center space-x-1">
              {getMembershipIcon()}
              <span className="text-luxury-platinum/80">
                {user.membershipTier}
              </span>
            </div>
          )}
        </div>
      </div>
    </motion.div>
  )
}

export default AdminIndicator