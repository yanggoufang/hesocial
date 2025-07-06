import React from 'react'
import { motion } from 'framer-motion'
import { Shield, Crown, User, Check, X } from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { usePermissions } from '../hooks/useRoleAccess'
import AccessControl, { AdminOnly, SuperAdminOnly, DiamondOnly, VVIPOnly } from '../components/AccessControl'

const AccessTestPage: React.FC = () => {
  const { user, isAuthenticated } = useAuth()
  const { can, cannot, roleAccess, membershipAccess, verificationAccess } = usePermissions()

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-12">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-center mb-12"
        >
          <h1 className="text-4xl font-luxury font-bold text-luxury-gold mb-4">
            Access Control Test Page
          </h1>
          <p className="text-luxury-platinum/80 text-lg">
            Testing role-based and membership-based access controls
          </p>
        </motion.div>

        {/* User Status */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="luxury-glass p-6 rounded-2xl mb-8"
        >
          <h2 className="text-2xl font-luxury font-semibold text-luxury-gold mb-4 flex items-center">
            <User className="h-6 w-6 mr-2" />
            Current User Status
          </h2>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="text-center">
              <div className={`text-lg font-medium ${isAuthenticated ? 'text-green-400' : 'text-red-400'}`}>
                {isAuthenticated ? <Check className="h-5 w-5 mx-auto mb-1" /> : <X className="h-5 w-5 mx-auto mb-1" />}
              </div>
              <p className="text-luxury-platinum/80 text-sm">Authenticated</p>
            </div>
            
            <div className="text-center">
              <div className="text-luxury-gold text-lg font-medium mb-1">
                {user?.role || 'None'}
              </div>
              <p className="text-luxury-platinum/80 text-sm">Role</p>
            </div>
            
            <div className="text-center">
              <div className="text-luxury-gold text-lg font-medium mb-1">
                {user?.membershipTier || 'None'}
              </div>
              <p className="text-luxury-platinum/80 text-sm">Membership</p>
            </div>
            
            <div className="text-center">
              <div className={`text-lg font-medium ${verificationAccess.isVerified ? 'text-green-400' : 'text-yellow-400'}`}>
                {verificationAccess.isVerified ? <Check className="h-5 w-5 mx-auto mb-1" /> : <X className="h-5 w-5 mx-auto mb-1" />}
              </div>
              <p className="text-luxury-platinum/80 text-sm">Verified</p>
            </div>
          </div>
        </motion.div>

        {/* Permission Tests */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
          {/* Role-Based Access */}
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.2 }}
            className="luxury-glass p-6 rounded-2xl"
          >
            <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4 flex items-center">
              <Shield className="h-5 w-5 mr-2" />
              Role-Based Access
            </h3>
            
            <div className="space-y-3">
              <AccessControl requireAuth={true} fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>User only content (not authenticated)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>User only content (authenticated)</span>
                </div>
              </AccessControl>

              <AdminOnly fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>Admin only content (insufficient role)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>Admin only content (has access)</span>
                </div>
              </AdminOnly>

              <SuperAdminOnly fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>Super Admin only content (insufficient role)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>Super Admin only content (has access)</span>
                </div>
              </SuperAdminOnly>
            </div>
          </motion.div>

          {/* Membership-Based Access */}
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
            className="luxury-glass p-6 rounded-2xl"
          >
            <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4 flex items-center">
              <Crown className="h-5 w-5 mr-2" />
              Membership-Based Access
            </h3>
            
            <div className="space-y-3">
              <DiamondOnly fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>Diamond members only (insufficient tier)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>Diamond members only (has access)</span>
                </div>
              </DiamondOnly>

              <VVIPOnly fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>VVIP only (Diamond + Verified required)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>VVIP only (has access)</span>
                </div>
              </VVIPOnly>

              <AccessControl requireMembership="Black Card" fallback={
                <div className="flex items-center text-red-400">
                  <X className="h-4 w-4 mr-2" />
                  <span>Black Card only (insufficient tier)</span>
                </div>
              }>
                <div className="flex items-center text-green-400">
                  <Check className="h-4 w-4 mr-2" />
                  <span>Black Card only (has access)</span>
                </div>
              </AccessControl>
            </div>
          </motion.div>
        </div>

        {/* Detailed Permissions Matrix */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
            Detailed Permissions Matrix
          </h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {Object.entries(can).map(([permission, hasPermission]) => (
              <div 
                key={permission}
                className={`flex items-center p-3 rounded-lg ${
                  hasPermission ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
                }`}
              >
                {hasPermission ? (
                  <Check className="h-4 w-4 mr-2" />
                ) : (
                  <X className="h-4 w-4 mr-2" />
                )}
                <span className="text-sm font-medium">
                  {permission.replace(/([A-Z])/g, ' $1').toLowerCase()}
                </span>
              </div>
            ))}
          </div>
        </motion.div>

        {/* Navigation Tests */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
          className="luxury-glass p-6 rounded-2xl mt-8"
        >
          <h3 className="text-xl font-luxury font-semibold text-luxury-gold mb-4">
            Protected Route Navigation Tests
          </h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <a
              href="/admin"
              className="block p-4 border border-luxury-gold/30 rounded-lg hover:bg-luxury-gold/10 transition-colors text-center"
            >
              <Shield className="h-6 w-6 mx-auto mb-2 text-luxury-gold" />
              <p className="text-luxury-platinum font-medium">Admin Dashboard</p>
              <p className="text-luxury-platinum/60 text-sm">Requires: Admin+</p>
            </a>
            
            <a
              href="/admin/backups"
              className="block p-4 border border-luxury-gold/30 rounded-lg hover:bg-luxury-gold/10 transition-colors text-center"
            >
              <Shield className="h-6 w-6 mx-auto mb-2 text-luxury-gold" />
              <p className="text-luxury-platinum font-medium">Backup Management</p>
              <p className="text-luxury-platinum/60 text-sm">Requires: Super Admin</p>
            </a>
            
            <a
              href="/vvip"
              className="block p-4 border border-luxury-gold/30 rounded-lg hover:bg-luxury-gold/10 transition-colors text-center"
            >
              <Crown className="h-6 w-6 mx-auto mb-2 text-luxury-gold" />
              <p className="text-luxury-platinum font-medium">VVIP Area</p>
              <p className="text-luxury-platinum/60 text-sm">Requires: Diamond + Verified</p>
            </a>
          </div>
        </motion.div>
      </div>
    </div>
  )
}

export default AccessTestPage