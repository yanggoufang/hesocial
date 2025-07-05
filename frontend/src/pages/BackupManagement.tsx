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
        setError(result.error || 'Failed to load backups')
      }
    } catch (err) {
      setError('Failed to load backups')
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
        setSuccess('Manual backup created successfully!')
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || 'Failed to create backup')
      }
    } catch (err) {
      setError('Failed to create backup')
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
        setSuccess('Database restored successfully!')
        setShowRestoreConfirm(null)
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || 'Failed to restore backup')
      }
    } catch (err) {
      setError('Failed to restore backup')
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
        setSuccess('Old backups cleaned up successfully!')
        await loadBackups() // Refresh the list
      } else {
        setError(result.error || 'Failed to cleanup backups')
      }
    } catch (err) {
      setError('Failed to cleanup backups')
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
                <span>Back to Dashboard</span>
              </button>
            </div>
            <div className="flex items-center space-x-3 mb-4">
              <Database className="h-8 w-8 text-luxury-gold" />
              <h1 className="text-3xl font-luxury font-bold text-luxury-gold">
                Backup Management
              </h1>
            </div>
            <p className="text-luxury-platinum/80">
              Create, restore, and manage database backups. Manual backups provide full control over your data.
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
              <span>{actionLoading === 'create' ? 'Creating...' : 'Create Manual Backup'}</span>
            </motion.button>

            <motion.button
              onClick={loadBackups}
              disabled={loading}
              className="px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center space-x-2"
              whileHover={{ scale: loading ? 1 : 1.02 }}
              whileTap={{ scale: loading ? 1 : 0.98 }}
            >
              <RefreshCw className={`h-5 w-5 ${loading ? 'animate-spin' : ''}`} />
              <span>Refresh List</span>
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
              <span>{actionLoading === 'cleanup' ? 'Cleaning...' : 'Cleanup Old Backups'}</span>
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
              Available Backups ({backups.length})
            </h2>

            {loading ? (
              <div className="flex items-center justify-center py-12">
                <RefreshCw className="h-8 w-8 text-luxury-gold animate-spin" />
                <span className="ml-3 text-luxury-platinum">Loading backups...</span>
              </div>
            ) : backups.length === 0 ? (
              <div className="text-center py-12">
                <Database className="h-12 w-12 text-luxury-platinum/30 mx-auto mb-4" />
                <p className="text-luxury-platinum/60">No backups available</p>
                <p className="text-luxury-platinum/40 text-sm mt-2">
                  Create your first backup using the button above
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
                            {backup.type.charAt(0).toUpperCase() + backup.type.slice(1)} Backup
                          </h3>
                          {backup.status === 'latest_restored' && (
                            <span className="px-2 py-1 bg-green-500/20 text-green-400 text-xs rounded">
                              Latest Restored
                            </span>
                          )}
                        </div>
                        <p className="text-luxury-platinum/60 text-sm">
                          {new Date(backup.timestamp).toLocaleString()}
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
                          {backup.type}
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
                          <span>{actionLoading === backup.id ? 'Restoring...' : 'Restore'}</span>
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
                    Confirm Restore
                  </h3>
                </div>
                
                <p className="text-luxury-platinum/80 mb-6">
                  Are you sure you want to restore from this backup? This will replace the current database with the backup data.
                </p>

                <div className="flex space-x-3">
                  <button
                    onClick={() => handleRestoreBackup(showRestoreConfirm, true)}
                    disabled={actionLoading === showRestoreConfirm}
                    className="flex-1 luxury-button disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {actionLoading === showRestoreConfirm ? 'Restoring...' : 'Confirm Restore'}
                  </button>
                  <button
                    onClick={() => setShowRestoreConfirm(null)}
                    disabled={actionLoading === showRestoreConfirm}
                    className="flex-1 px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Cancel
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