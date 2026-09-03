import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { useNavigate } from 'react-router-dom'
import { 
  Database, 
  Download, 
  Upload, 
  Trash2, 
  RefreshCw, 
  Shield, 
  Clock,
  HardDrive,
  AlertTriangle,
  CheckCircle,
  ArrowLeft
} from 'lucide-react'
import { useAuth } from '../hooks/useAuth'
import { adminService, BackupInfo } from '../services/adminService'

const BackupManagement = () => {
  const navigate = useNavigate()
  const { user, isAuthenticated } = useAuth()
  const [backups, setBackups] = useState<BackupInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState('')
  const [showRestoreConfirm, setShowRestoreConfirm] = useState<string | null>(null)

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

    loadBackups()
  }, [isAuthenticated, user, navigate])

  const loadBackups = async () => {
    try {
      setLoading(true)
      setError('')
      const result = await adminService.listBackups(50)
      
      if (result.success && result.data) {
        setBackups(result.data)
      } else {
        setError(result.error || '載入備份失敗')
      }
    } catch (err) {
      setError('載入備份失敗')
    } finally {
      setLoading(false)
    }
  }

  const handleCreateBackup = async () => {
    try {
      setActionLoading('create')
      setError('')
      setSuccess('')
      
      const result = await adminService.createBackup()
      
      if (result.success) {
        setSuccess('手動備份建立成功！')
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || '建立備份失敗')
      }
    } catch (err) {
      setError('建立備份失敗')
    } finally {
      setActionLoading(null)
    }
  }

  const handleRestoreBackup = async (backupId: string, force: boolean = false) => {
    try {
      setActionLoading(backupId)
      setError('')
      setSuccess('')
      
      const result = await adminService.restoreBackup(backupId, force)
      
      if (result.success) {
        setSuccess('資料庫還原成功！')
        setShowRestoreConfirm(null)
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || '還原備份失敗')
      }
    } catch (err) {
      setError('還原備份失敗')
    } finally {
      setActionLoading(null)
    }
  }

  const handleCleanupBackups = async () => {
    try {
      setActionLoading('cleanup')
      setError('')
      setSuccess('')
      
      const result = await adminService.cleanupBackups()
      
      if (result.success) {
        setSuccess('舊備份清理成功！')
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || '清理備份失敗')
      }
    } catch (err) {
      setError('清理備份失敗')
    } finally {
      setActionLoading(null)
    }
  }

  const getBackupTypeIcon = (type: string) => {
    switch (type) {
      case 'manual':
        return <HardDrive className="h-4 w-4 text-luxury-gold" />
      case 'periodic':
        return <Clock className="h-4 w-4 text-blue-400" />
      case 'shutdown':
        return <Shield className="h-4 w-4 text-green-400" />
      default:
        return <Database className="h-4 w-4 text-gray-400" />
    }
  }

  const getBackupTypeColor = (type: string) => {
    switch (type) {
      case 'manual':
        return 'text-luxury-gold'
      case 'periodic':
        return 'text-blue-400'
      case 'shutdown':
        return 'text-green-400'
      default:
        return 'text-gray-400'
    }
  }

  if (!isAuthenticated || !user || !['admin', 'super_admin'].includes(user.role || '')) {
    return null
  }

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-8 px-4">
      <div className="max-w-6xl mx-auto">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8 }}
        >
          {/* Header */}
          <div className="mb-8">
            <div className="flex items-center space-x-4 mb-4">
              <button
                onClick={() => navigate('/admin')}
                className="flex items-center space-x-2 text-luxury-platinum/60 hover:text-luxury-gold transition-colors"
              >
                <ArrowLeft className="h-5 w-5" />
                <span>返回儀表板</span>
              </button>
            </div>
            <div className="flex items-center space-x-3 mb-4">
              <Database className="h-8 w-8 text-luxury-gold" />
              <h1 className="text-3xl font-luxury font-bold text-luxury-gold">
                備份管理
              </h1>
            </div>
            <p className="text-luxury-platinum/80">
              建立、還原和管理資料庫備份。手動備份提供對您資料的完整控制。
            </p>
          </div>

          {/* Action Buttons */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
            <motion.button
              onClick={handleCreateBackup}
              disabled={actionLoading === 'create'}
              className="luxury-button flex items-center justify-center space-x-2 disabled:opacity-50 disabled:cursor-not-allowed"
              whileHover={{ scale: actionLoading === 'create' ? 1 : 1.02 }}
              whileTap={{ scale: actionLoading === 'create' ? 1 : 0.98 }}
            >
              {actionLoading === 'create' ? (
                <RefreshCw className="h-5 w-5 animate-spin" />
              ) : (
                <Upload className="h-5 w-5" />
              )}
              <span>{actionLoading === 'create' ? '建立中...' : '建立手動備份'}</span>
            </motion.button>

            <motion.button
              onClick={loadBackups}
              disabled={loading}
              className="px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
              whileHover={{ scale: loading ? 1 : 1.02 }}
              whileTap={{ scale: loading ? 1 : 0.98 }}
            >
              <RefreshCw className={`h-5 w-5 ${loading ? 'animate-spin' : ''}`} />
              <span>重新整理清單</span>
            </motion.button>

            <motion.button
              onClick={handleCleanupBackups}
              disabled={actionLoading === 'cleanup'}
              className="px-6 py-3 border border-red-500/30 text-red-400 rounded-lg hover:bg-red-500/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
              whileHover={{ scale: actionLoading === 'cleanup' ? 1 : 1.02 }}
              whileTap={{ scale: actionLoading === 'cleanup' ? 1 : 0.98 }}
            >
              {actionLoading === 'cleanup' ? (
                <RefreshCw className="h-5 w-5 animate-spin" />
              ) : (
                <Trash2 className="h-5 w-5" />
              )}
              <span>{actionLoading === 'cleanup' ? '清理中...' : '清理舊備份'}</span>
            </motion.button>
          </div>

          {/* Status Messages */}
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm flex items-center space-x-2"
            >
              <AlertTriangle className="h-4 w-4" />
              <span>{error}</span>
            </motion.div>
          )}

          {success && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              className="mb-6 p-4 bg-green-500/20 border border-green-500/50 rounded-lg text-green-400 text-sm flex items-center space-x-2"
            >
              <CheckCircle className="h-4 w-4" />
              <span>{success}</span>
            </motion.div>
          )}

          {/* Backup List */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="luxury-glass rounded-xl p-6"
          >
            <h2 className="text-xl font-luxury font-bold text-luxury-gold mb-6">
              可用備份 ({backups.length})
            </h2>

            {loading ? (
              <div className="flex items-center justify-center py-12">
                <RefreshCw className="h-8 w-8 text-luxury-gold animate-spin" />
                <span className="ml-3 text-luxury-platinum">載入備份中...</span>
              </div>
            ) : backups.length === 0 ? (
              <div className="text-center py-12">
                <Database className="h-12 w-12 text-luxury-platinum/30 mx-auto mb-4" />
                <p className="text-luxury-platinum/60">沒有可用的備份</p>
                <p className="text-luxury-platinum/40 text-sm mt-2">
                  使用上方按鈕建立您的第一個備份
                </p>
              </div>
            ) : (
              <div className="space-y-4">
                {backups.map((backup) => (
                  <motion.div
                    key={backup.id}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="bg-luxury-midnight-black/50 rounded-lg p-4 flex items-center justify-between"
                  >
                    <div className="flex items-center space-x-4">
                      {getBackupTypeIcon(backup.type)}
                      <div>
                        <div className="flex items-center space-x-2">
                          <h3 className="text-luxury-platinum font-medium">
                            {backup.type === 'manual' ? '手動' : backup.type === 'periodic' ? '定期' : backup.type === 'shutdown' ? '關機' : backup.type} 備份
                          </h3>
                          {backup.status === 'latest_restored' && (
                            <span className="px-2 py-1 bg-green-500/20 text-green-400 text-xs rounded">
                              最新已還原
                            </span>
                          )}
                        </div>
                        <p className="text-luxury-platinum/60 text-sm">
                          {new Date(backup.timestamp).toLocaleString('zh-TW')}
                        </p>
                        <p className="text-luxury-platinum/40 text-xs">
                          ID: {backup.id.substring(0, 16)}...
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center space-x-4">
                      <div className="text-right">
                        <p className="text-luxury-gold font-medium">{backup.size}</p>
                        <p className={`text-xs ${getBackupTypeColor(backup.type)}`}>
                          {backup.type === 'manual' ? '手動' : backup.type === 'periodic' ? '定期' : backup.type === 'shutdown' ? '關機' : backup.type}
                        </p>
                      </div>

                      {user.role === 'super_admin' && (
                        <button
                          onClick={() => setShowRestoreConfirm(backup.id)}
                          disabled={actionLoading === backup.id}
                          className="px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
                        >
                          {actionLoading === backup.id ? (
                            <RefreshCw className="h-4 w-4 animate-spin" />
                          ) : (
                            <Download className="h-4 w-4" />
                          )}
                          <span>{actionLoading === backup.id ? '還原中...' : '還原'}</span>
                        </button>
                      )}
                    </div>
                  </motion.div>
                ))}
              </div>
            )}
          </motion.div>

          {/* Restore Confirmation Modal */}
          {showRestoreConfirm && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
            >
              <motion.div
                initial={{ scale: 0.9, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                className="luxury-glass p-6 rounded-xl max-w-md w-full"
              >
                <div className="flex items-center space-x-3 mb-4">
                  <AlertTriangle className="h-6 w-6 text-yellow-400" />
                  <h3 className="text-xl font-luxury font-bold text-luxury-gold">
                    確認還原
                  </h3>
                </div>
                
                <p className="text-luxury-platinum/80 mb-6">
                  您確定要從此備份還原嗎？這將以備份資料取代目前的資料庫。
                </p>

                <div className="flex space-x-3">
                  <button
                    onClick={() => handleRestoreBackup(showRestoreConfirm, true)}
                    disabled={actionLoading === showRestoreConfirm}
                    className="flex-1 luxury-button disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {actionLoading === showRestoreConfirm ? '還原中...' : '確認還原'}
                  </button>
                  <button
                    onClick={() => setShowRestoreConfirm(null)}
                    disabled={actionLoading === showRestoreConfirm}
                    className="flex-1 px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    取消
                  </button>
                </div>
              </motion.div>
            </motion.div>
          )}
        </motion.div>
      </div>
    </div>
  )
}

export default BackupManagement