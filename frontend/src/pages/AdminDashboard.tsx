import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { 
  Shield, 
  Database, 
  Cloud, 
  Activity, 
  Users, 
  BarChart3,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Calendar
} from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { adminService, SystemHealth } from '../services/adminService'

const AdminDashboard = () => {
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()
  const [systemHealth, setSystemHealth] = useState<SystemHealth | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    // Check if user is authenticated and has admin role
    if (!isAuthenticated || !user) {
      navigate('/login')
      return
    }

    if (!['admin', 'super_admin'].includes(user.role || '')) {
      navigate('/')
      return
    }

    loadSystemHealth()
  }, [isAuthenticated, user, navigate])

  const loadSystemHealth = async () => {
    try {
      setLoading(true)
      const result = await adminService.getSystemHealth()
      
      if (result.success && result.data) {
        setSystemHealth(result.data)
      } else {
        setError(result.error || 'Failed to load system health')
      }
    } catch (err) {
      setError('Failed to load system health')
    } finally {
      setLoading(false)
    }
  }

  const getStatusIcon = (status: 'healthy' | 'unhealthy' | 'disabled') => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="h-5 w-5 text-green-400" />
      case 'unhealthy':
        return <XCircle className="h-5 w-5 text-red-400" />
      case 'disabled':
        return <AlertTriangle className="h-5 w-5 text-yellow-400" />
      default:
        return <XCircle className="h-5 w-5 text-gray-400" />
    }
  }

  const getStatusColor = (status: 'healthy' | 'unhealthy' | 'disabled') => {
    switch (status) {
      case 'healthy':
        return 'text-green-400'
      case 'unhealthy':
        return 'text-red-400'
      case 'disabled':
        return 'text-yellow-400'
      default:
        return 'text-gray-400'
    }
  }

  if (!isAuthenticated || !user || !['admin', 'super_admin'].includes(user.role || '')) {
    return null
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8 px-4">
      <div className="max-w-7xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          {/* Header */}
          <div className="mb-8">
            <div className="flex items-center space-x-3 mb-4">
              <Shield className="h-8 w-8 text-luxury-gold" />
              <h1 className="text-3xl font-luxury font-bold text-luxury-gold">
                Admin Dashboard
              </h1>
            </div>
            <p className="text-luxury-platinum/80">
              Welcome back, {user.firstName}. Manage system operations and monitor platform health.
            </p>
          </div>

          {/* System Status Cards */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            {/* Database Status */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center justify-between mb-4">
                <Database className="h-8 w-8 text-luxury-gold" />
                {systemHealth && getStatusIcon(systemHealth.database.status)}
              </div>
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">Database</h3>
              <p className={`text-sm font-medium ${systemHealth ? getStatusColor(systemHealth.database.status) : 'text-gray-400'}`}>
                {loading ? 'Loading...' : systemHealth?.database.connected ? 'Connected' : 'Disconnected'}
              </p>
              {systemHealth?.database.queryTime && (
                <p className="text-luxury-platinum/60 text-xs mt-1">
                  Query time: {systemHealth.database.queryTime}ms
                </p>
              )}
            </motion.div>

            {/* R2 Backup Status */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center justify-between mb-4">
                <Cloud className="h-8 w-8 text-luxury-gold" />
                {systemHealth && getStatusIcon(systemHealth.r2Sync.status)}
              </div>
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">R2 Backup</h3>
              <p className={`text-sm font-medium ${systemHealth ? getStatusColor(systemHealth.r2Sync.status) : 'text-gray-400'}`}>
                {loading ? 'Loading...' : systemHealth?.r2Sync.enabled ? 
                  (systemHealth.r2Sync.connectionHealthy ? 'Healthy' : 'Connection Issue') : 
                  'Disabled'
                }
              </p>
              {systemHealth?.r2Sync.backupCount !== undefined && (
                <p className="text-luxury-platinum/60 text-xs mt-1">
                  {systemHealth.r2Sync.backupCount} backups available
                </p>
              )}
            </motion.div>

            {/* System Performance */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center justify-between mb-4">
                <Activity className="h-8 w-8 text-luxury-gold" />
                <CheckCircle className="h-5 w-5 text-green-400" />
              </div>
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">Performance</h3>
              <p className="text-sm font-medium text-green-400">Optimal</p>
              <p className="text-luxury-platinum/60 text-xs mt-1">
                All systems operational
              </p>
            </motion.div>

            {/* User Management */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center justify-between mb-4">
                <Users className="h-8 w-8 text-luxury-gold" />
                <CheckCircle className="h-5 w-5 text-green-400" />
              </div>
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">Users</h3>
              <p className="text-sm font-medium text-green-400">Active</p>
              <p className="text-luxury-platinum/60 text-xs mt-1">
                Authentication system online
              </p>
            </motion.div>
          </div>

          {/* Quick Actions */}
          <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 mb-8">
            {/* Backup Management */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Database className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  Backup Management
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                Create, restore, and manage database backups. Manual backups are recommended for full control.
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/backups')}
                  className="w-full luxury-button text-center"
                >
                  Manage Backups
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    Quick Backup
                  </button>
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    View Health
                  </button>
                </div>
              </div>
            </motion.div>

            {/* System Monitoring */}
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.6 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <BarChart3 className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  System Monitoring
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                Monitor system health, performance metrics, and operational status in real-time.
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/monitoring')}
                  className="w-full luxury-button text-center"
                >
                  View Monitoring
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button 
                    onClick={loadSystemHealth}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    Refresh Status
                  </button>
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    Health Report
                  </button>
                </div>
              </div>
            </motion.div>

            {/* User Management */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Users className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  User Management
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                Manage user accounts, roles, verification status, and membership tiers.
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/users')}
                  className="w-full luxury-button text-center"
                >
                  Manage Users
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    Pending Verifications
                  </button>
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    User Stats
                  </button>
                </div>
              </div>
            </motion.div>

            {/* Event Management */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.8 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Calendar className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  Event Management
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                Create and manage luxury social events, venues, and categories for the platform.
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/events/manage')}
                  className="w-full luxury-button text-center"
                >
                  Manage Events
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button 
                    onClick={() => navigate('/events/venues')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    Venues
                  </button>
                  <button 
                    onClick={() => navigate('/events/categories')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    Categories
                  </button>
                </div>
              </div>
            </motion.div>
          </div>

          {/* Recent Activity */}
          {systemHealth?.r2Sync.recentBackups && systemHealth.r2Sync.recentBackups.length > 0 && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.7 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Activity className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  Recent Backup Activity
                </h2>
              </div>
              <div className="space-y-3">
                {systemHealth.r2Sync.recentBackups.slice(0, 3).map((backup) => (
                  <div key={backup.id} className="flex items-center justify-between p-3 bg-luxury-midnight-black/50 rounded-lg">
                    <div>
                      <p className="text-luxury-platinum font-medium">{backup.type} backup</p>
                      <p className="text-luxury-platinum/60 text-sm">
                        {new Date(backup.timestamp).toLocaleString()}
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="text-luxury-gold text-sm">{backup.size}</p>
                      <p className="text-luxury-platinum/60 text-xs">
                        {backup.id.substring(0, 8)}...
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            </motion.div>
          )}

          {/* Error Display */}
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm"
            >
              {error}
            </motion.div>
          )}
        </motion.div>
      </div>
    </div>
  )
}

export default AdminDashboard