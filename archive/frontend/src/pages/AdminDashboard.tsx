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
                管理後台
              </h1>
            </div>
            <p className="text-luxury-platinum/80">
              歡迎回來，{user.firstName}。管理系統運作並監控平台健康狀況。
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
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">資料庫</h3>
              <p className={`text-sm font-medium ${systemHealth ? getStatusColor(systemHealth.database.status) : 'text-gray-400'}`}>
                {loading ? '載入中...' : systemHealth?.database.connected ? '已連線' : '未連線'}
              </p>
              {systemHealth?.database.queryTime && (
                <p className="text-luxury-platinum/60 text-xs mt-1">
                  查詢時間：{systemHealth.database.queryTime}ms
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
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">R2 備份</h3>
              <p className={`text-sm font-medium ${systemHealth ? getStatusColor(systemHealth.r2Sync.status) : 'text-gray-400'}`}>
                {loading ? '載入中...' : systemHealth?.r2Sync.enabled ? 
                  (systemHealth.r2Sync.connectionHealthy ? '正常' : '連線問題') : 
                  '已停用'
                }
              </p>
              {systemHealth?.r2Sync.backupCount !== undefined && (
                <p className="text-luxury-platinum/60 text-xs mt-1">
                  {systemHealth.r2Sync.backupCount} 個備份可用
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
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">效能</h3>
              <p className="text-sm font-medium text-green-400">最佳</p>
              <p className="text-luxury-platinum/60 text-xs mt-1">
                所有系統正常運作
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
              <h3 className="text-luxury-platinum text-lg font-medium mb-2">使用者</h3>
              <p className="text-sm font-medium text-green-400">活躍</p>
              <p className="text-luxury-platinum/60 text-xs mt-1">
                認證系統已上線
              </p>
            </motion.div>
          </div>

          {/* Quick Actions */}
          <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 mb-8">
            {/* Analytics Dashboard */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <BarChart3 className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  分析儀表板
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                查看活動表現、營收分析和會員參與度等關鍵業務指標。
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/analytics')}
                  className="w-full luxury-button text-center"
                >
                  查看分析數據
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button 
                    onClick={() => navigate('/admin/analytics')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    營收報告
                  </button>
                  <button 
                    onClick={() => navigate('/admin/analytics')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    會員分析
                  </button>
                </div>
              </div>
            </motion.div>

            {/* Backup Management */}
            <motion.div
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.6 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Database className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  備份管理
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                建立、還原和管理資料庫備份。建議使用手動備份以獲得完整控制。
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/backups')}
                  className="w-full luxury-button text-center"
                >
                  管理備份
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    快速備份
                  </button>
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    檢視健康狀況
                  </button>
                </div>
              </div>
            </motion.div>

            {/* System Monitoring */}
            <motion.div
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.7 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Activity className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  系統監控
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                即時監控系統健康狀況、效能指標和運作狀態。
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/system')}
                  className="w-full luxury-button text-center"
                >
                  檢視系統健康狀況
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button 
                    onClick={loadSystemHealth}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    重新整理狀態
                  </button>
                  <button 
                    onClick={() => navigate('/admin/system')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    健康儀表板
                  </button>
                </div>
              </div>
            </motion.div>

            {/* User Management */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.8 }}
              className="luxury-glass p-6 rounded-xl"
            >
              <div className="flex items-center space-x-3 mb-4">
                <Users className="h-6 w-6 text-luxury-gold" />
                <h2 className="text-xl font-luxury font-bold text-luxury-gold">
                  使用者管理
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                管理使用者帳戶、角色、驗證狀態和會員等級。
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/admin/users')}
                  className="w-full luxury-button text-center"
                >
                  管理使用者
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    待審核驗證
                  </button>
                  <button className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm">
                    使用者統計
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
                  活動管理
                </h2>
              </div>
              <p className="text-luxury-platinum/80 mb-6">
                為平台建立和管理頂級社交活動、場地和類別。
              </p>
              <div className="space-y-3">
                <button
                  onClick={() => navigate('/events/manage')}
                  className="w-full luxury-button text-center"
                >
                  管理活動
                </button>
                <div className="grid grid-cols-2 gap-3">
                  <button 
                    onClick={() => navigate('/events/venues')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    場地
                  </button>
                  <button 
                    onClick={() => navigate('/events/categories')}
                    className="px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm"
                  >
                    類別
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
                  最近備份活動
                </h2>
              </div>
              <div className="space-y-3">
                {systemHealth.r2Sync.recentBackups.slice(0, 3).map((backup) => (
                  <div key={backup.id} className="flex items-center justify-between p-3 bg-luxury-midnight-black/50 rounded-lg">
                    <div>
                      <p className="text-luxury-platinum font-medium">{backup.type} 備份</p>
                      <p className="text-luxury-platinum/60 text-sm">
                        {new Date(backup.timestamp).toLocaleString('zh-TW')}
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