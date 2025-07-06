import React, { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import {
  Activity,
  Server,
  Database,
  Shield,
  AlertTriangle,
  CheckCircle,
  XCircle,
  HardDrive,
  Zap,
  Cloud,
  Lock,
  TrendingUp,
  RefreshCw,
  Eye,
  Settings
} from 'lucide-react'
import systemHealthService, { 
  SystemHealthData, 
  DetailedHealthCheck, 
  SystemMetrics, 
  SystemDiagnostics 
} from '../services/systemHealthService'

const SystemHealthDashboard: React.FC = () => {
  const [healthData, setHealthData] = useState<SystemHealthData | null>(null)
  const [detailedHealth, setDetailedHealth] = useState<DetailedHealthCheck | null>(null)
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null)
  const [diagnostics, setDiagnostics] = useState<SystemDiagnostics[] | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [activeTab, setActiveTab] = useState<'overview' | 'detailed' | 'metrics' | 'diagnostics'>('overview')
  const [error, setError] = useState<string | null>(null)

  const loadHealthData = async () => {
    try {
      setError(null)
      const [health, detailed, metrics, diag] = await Promise.all([
        systemHealthService.getSystemHealth(),
        systemHealthService.getDetailedHealthCheck(),
        systemHealthService.getSystemMetrics(),
        systemHealthService.runSystemDiagnostics()
      ])
      
      setHealthData(health)
      setDetailedHealth(detailed)
      setSystemMetrics(metrics)
      setDiagnostics(diag)
    } catch (error) {
      console.error('Failed to load health data:', error)
      setError('Failed to load system health data')
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }

  const refreshData = async () => {
    setRefreshing(true)
    await loadHealthData()
  }

  useEffect(() => {
    loadHealthData()
    
    // Set up auto-refresh every 30 seconds
    const interval = setInterval(loadHealthData, 30000)
    return () => clearInterval(interval)
  }, [])

  if (loading) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-luxury-gold"></div>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen bg-luxury-midnight-black py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="luxury-glass p-8 rounded-2xl text-center">
            <XCircle className="h-16 w-16 text-red-400 mx-auto mb-4" />
            <h2 className="text-2xl font-luxury font-bold text-luxury-gold mb-2">
              System Health Unavailable
            </h2>
            <p className="text-luxury-platinum/80 mb-6">{error}</p>
            <button
              onClick={refreshData}
              className="luxury-button-primary"
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              Retry
            </button>
          </div>
        </div>
      </div>
    )
  }

  const getOverallStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircle className="h-8 w-8 text-green-400" />
      case 'warning':
        return <AlertTriangle className="h-8 w-8 text-yellow-400" />
      case 'error':
        return <XCircle className="h-8 w-8 text-red-400" />
      default:
        return <Activity className="h-8 w-8 text-gray-400" />
    }
  }

  const renderOverviewTab = () => (
    <div className="space-y-6">
      {/* Overall Status */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="luxury-glass p-6 rounded-2xl"
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            {getOverallStatusIcon(healthData?.overall || 'error')}
            <div>
              <h3 className="text-2xl font-luxury font-bold text-luxury-gold">
                System Status: {healthData?.overall?.toUpperCase()}
              </h3>
              <p className="text-luxury-platinum/80">
                Last updated: {healthData?.timestamp ? new Date(healthData.timestamp).toLocaleString() : 'Unknown'}
              </p>
            </div>
          </div>
          <button
            onClick={refreshData}
            disabled={refreshing}
            className="luxury-button-secondary flex items-center space-x-2"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
            <span>Refresh</span>
          </button>
        </div>
      </motion.div>

      {/* Quick Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* System Stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <div className="flex items-center space-x-3 mb-4">
            <Server className="h-6 w-6 text-luxury-gold" />
            <h4 className="text-lg font-luxury font-semibold text-luxury-platinum">System</h4>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Uptime:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.system ? systemHealthService.formatUptime(healthData.system.uptime) : 'N/A'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Memory:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.system?.memory.usage}%
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Environment:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.system?.environment}
              </span>
            </div>
          </div>
        </motion.div>

        {/* Database Stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <div className="flex items-center space-x-3 mb-4">
            <Database className="h-6 w-6 text-luxury-gold" />
            <h4 className="text-lg font-luxury font-semibold text-luxury-platinum">Database</h4>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Status:</span>
              <span className={`font-medium ${healthData?.database?.connected ? 'text-green-400' : 'text-red-400'}`}>
                {healthData?.database?.connected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Users:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.database?.userCount || 0}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Events:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.database?.eventCount || 0}
              </span>
            </div>
          </div>
        </motion.div>

        {/* R2 Backup Stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <div className="flex items-center space-x-3 mb-4">
            <Cloud className="h-6 w-6 text-luxury-gold" />
            <h4 className="text-lg font-luxury font-semibold text-luxury-platinum">R2 Backup</h4>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Status:</span>
              <span className={`font-medium ${
                healthData?.r2Backup?.enabled 
                  ? (healthData.r2Backup.connectionHealthy ? 'text-green-400' : 'text-red-400')
                  : 'text-yellow-400'
              }`}>
                {!healthData?.r2Backup?.enabled ? 'Disabled' : 
                 healthData.r2Backup.connectionHealthy ? 'Healthy' : 'Error'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Backups:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.r2Backup?.backupCount || 0}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Size:</span>
              <span className="text-luxury-platinum font-medium">
                {healthData?.r2Backup ? systemHealthService.formatBytes(healthData.r2Backup.totalBackupSize) : 'N/A'}
              </span>
            </div>
          </div>
        </motion.div>

        {/* Security Stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <div className="flex items-center space-x-3 mb-4">
            <Shield className="h-6 w-6 text-luxury-gold" />
            <h4 className="text-lg font-luxury font-semibold text-luxury-platinum">Security</h4>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">HTTPS:</span>
              <span className={`font-medium ${healthData?.security?.httpsEnabled ? 'text-green-400' : 'text-yellow-400'}`}>
                {healthData?.security?.httpsEnabled ? 'Enabled' : 'Dev Mode'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">CORS:</span>
              <span className={`font-medium ${healthData?.security?.corsConfigured ? 'text-green-400' : 'text-red-400'}`}>
                {healthData?.security?.corsConfigured ? 'Configured' : 'Missing'}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-luxury-platinum/80">Auth:</span>
              <span className={`font-medium ${
                systemHealthService.getStatusColor(healthData?.services?.authentication?.status || 'error')
              }`}>
                {healthData?.services?.authentication?.status || 'Unknown'}
              </span>
            </div>
          </div>
        </motion.div>
      </div>

      {/* Services Status */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="luxury-glass p-6 rounded-2xl"
      >
        <h3 className="text-xl font-luxury font-bold text-luxury-gold mb-6">Service Status</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Authentication Service */}
          <div className={`p-4 rounded-lg ${
            systemHealthService.getStatusBgColor(healthData?.services?.authentication?.status || 'error')
          }`}>
            <div className="flex items-center space-x-3 mb-3">
              <Lock className="h-5 w-5 text-luxury-platinum" />
              <h4 className="font-medium text-luxury-platinum">Authentication</h4>
              <span className="text-sm">
                {systemHealthService.getStatusIcon(healthData?.services?.authentication?.status || 'error')}
              </span>
            </div>
            <p className="text-sm text-luxury-platinum/80 mb-2">
              OAuth Providers: {healthData?.services?.authentication?.oauthProviders?.join(', ') || 'None'}
            </p>
            <p className="text-sm text-luxury-platinum/80">
              JWT: {healthData?.services?.authentication?.jwtConfigured ? 'Configured' : 'Missing'}
            </p>
          </div>

          {/* Payments Service */}
          <div className={`p-4 rounded-lg ${
            systemHealthService.getStatusBgColor(healthData?.services?.payments?.status || 'error')
          }`}>
            <div className="flex items-center space-x-3 mb-3">
              <Zap className="h-5 w-5 text-luxury-platinum" />
              <h4 className="font-medium text-luxury-platinum">Payments</h4>
              <span className="text-sm">
                {systemHealthService.getStatusIcon(healthData?.services?.payments?.status || 'error')}
              </span>
            </div>
            <p className="text-sm text-luxury-platinum/80">
              Stripe: {healthData?.services?.payments?.stripeConfigured ? 'Configured' : 'Not configured'}
            </p>
          </div>

          {/* Media Service */}
          <div className={`p-4 rounded-lg ${
            systemHealthService.getStatusBgColor(healthData?.services?.media?.status || 'error')
          }`}>
            <div className="flex items-center space-x-3 mb-3">
              <HardDrive className="h-5 w-5 text-luxury-platinum" />
              <h4 className="font-medium text-luxury-platinum">Media Storage</h4>
              <span className="text-sm">
                {systemHealthService.getStatusIcon(healthData?.services?.media?.status || 'error')}
              </span>
            </div>
            <p className="text-sm text-luxury-platinum/80">
              R2 Storage: {healthData?.services?.media?.r2Configured ? 'Configured' : 'Not configured'}
            </p>
          </div>
        </div>
      </motion.div>
    </div>
  )

  const renderDetailedTab = () => (
    <div className="space-y-6">
      {detailedHealth && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <h3 className="text-xl font-luxury font-bold text-luxury-gold mb-6">Detailed Health Check</h3>
          
          {/* Overall Status */}
          <div className="mb-6 p-4 rounded-lg bg-luxury-midnight-black/50">
            <div className="flex items-center space-x-3">
              {getOverallStatusIcon(detailedHealth.overall)}
              <span className="text-lg font-medium text-luxury-platinum">
                Overall Status: {detailedHealth.overall.toUpperCase()}
              </span>
            </div>
          </div>

          {/* Component Results */}
          <div className="space-y-4">
            {detailedHealth.results.map((result, index) => (
              <div 
                key={index}
                className={`p-4 rounded-lg ${systemHealthService.getStatusBgColor(result.status)}`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="text-sm">
                      {systemHealthService.getStatusIcon(result.status)}
                    </span>
                    <div>
                      <h4 className="font-medium text-luxury-platinum">{result.component}</h4>
                      <p className="text-sm text-luxury-platinum/80">{result.message}</p>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Critical Errors */}
          {detailedHealth.criticalErrors.length > 0 && (
            <div className="mt-6">
              <h4 className="text-lg font-semibold text-red-400 mb-3">Critical Errors</h4>
              <div className="space-y-2">
                {detailedHealth.criticalErrors.map((error, index) => (
                  <div key={index} className="p-3 bg-red-500/20 rounded-lg">
                    <p className="text-red-400 text-sm">{error}</p>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Warnings */}
          {detailedHealth.warnings.length > 0 && (
            <div className="mt-6">
              <h4 className="text-lg font-semibold text-yellow-400 mb-3">Warnings</h4>
              <div className="space-y-2">
                {detailedHealth.warnings.map((warning, index) => (
                  <div key={index} className="p-3 bg-yellow-500/20 rounded-lg">
                    <p className="text-yellow-400 text-sm">{warning}</p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </motion.div>
      )}
    </div>
  )

  const renderMetricsTab = () => (
    <div className="space-y-6">
      {systemMetrics && (
        <>
          {/* System Information */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="luxury-glass p-6 rounded-2xl"
          >
            <h3 className="text-xl font-luxury font-bold text-luxury-gold mb-6">System Information</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Environment</h4>
                <p className="text-lg font-semibold text-luxury-platinum">{systemMetrics.environment}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Node Version</h4>
                <p className="text-lg font-semibold text-luxury-platinum">{systemMetrics.nodeVersion}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Platform</h4>
                <p className="text-lg font-semibold text-luxury-platinum">{systemMetrics.platform}</p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Architecture</h4>
                <p className="text-lg font-semibold text-luxury-platinum">{systemMetrics.architecture}</p>
              </div>
            </div>
          </motion.div>

          {/* Memory Usage */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="luxury-glass p-6 rounded-2xl"
          >
            <h3 className="text-xl font-luxury font-bold text-luxury-gold mb-6">Memory Usage</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">RSS</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {systemHealthService.formatBytes(systemMetrics.memory.rss)}
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Heap Total</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {systemHealthService.formatBytes(systemMetrics.memory.heapTotal)}
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">Heap Used</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {systemHealthService.formatBytes(systemMetrics.memory.heapUsed)}
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">External</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {systemHealthService.formatBytes(systemMetrics.memory.external)}
                </p>
              </div>
            </div>
          </motion.div>

          {/* CPU Usage */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="luxury-glass p-6 rounded-2xl"
          >
            <h3 className="text-xl font-luxury font-bold text-luxury-gold mb-6">CPU Usage</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">User Time</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {Math.round(systemMetrics.cpu.user / 1000)}ms
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium text-luxury-platinum/80 mb-2">System Time</h4>
                <p className="text-lg font-semibold text-luxury-platinum">
                  {Math.round(systemMetrics.cpu.system / 1000)}ms
                </p>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </div>
  )

  const renderDiagnosticsTab = () => (
    <div className="space-y-6">
      {diagnostics && (
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="luxury-glass p-6 rounded-2xl"
        >
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-luxury font-bold text-luxury-gold">System Diagnostics</h3>
            <button
              onClick={refreshData}
              className="luxury-button-secondary flex items-center space-x-2"
            >
              <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
              <span>Run Tests</span>
            </button>
          </div>
          
          <div className="space-y-4">
            {diagnostics.map((diagnostic, index) => (
              <div 
                key={index}
                className={`p-4 rounded-lg ${
                  diagnostic.status === 'passed' ? 'bg-green-500/20' :
                  diagnostic.status === 'failed' ? 'bg-red-500/20' : 'bg-yellow-500/20'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="text-sm">
                      {diagnostic.status === 'passed' ? '✅' :
                       diagnostic.status === 'failed' ? '❌' : '⏭️'}
                    </span>
                    <div>
                      <h4 className="font-medium text-luxury-platinum">{diagnostic.test}</h4>
                      <p className="text-sm text-luxury-platinum/80">{diagnostic.message}</p>
                    </div>
                  </div>
                  <span className="text-xs text-luxury-platinum/60">
                    {new Date(diagnostic.timestamp).toLocaleTimeString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      )}
    </div>
  )

  return (
    <div className="min-h-screen bg-luxury-midnight-black py-12">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-center mb-8"
        >
          <h1 className="text-4xl font-luxury font-bold text-luxury-gold mb-4">
            System Health Dashboard
          </h1>
          <p className="text-luxury-platinum/80 text-lg">
            Real-time monitoring and diagnostics for the HeSocial platform
          </p>
        </motion.div>

        {/* Tab Navigation */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="flex flex-wrap justify-center space-x-1 mb-8"
        >
          {[
            { id: 'overview', label: 'Overview', icon: Activity },
            { id: 'detailed', label: 'Detailed', icon: Eye },
            { id: 'metrics', label: 'Metrics', icon: TrendingUp },
            { id: 'diagnostics', label: 'Diagnostics', icon: Settings }
          ].map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveTab(id as any)}
              className={`flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors ${
                activeTab === id
                  ? 'bg-luxury-gold text-luxury-midnight-black'
                  : 'text-luxury-platinum hover:bg-luxury-gold/10'
              }`}
            >
              <Icon className="h-4 w-4" />
              <span>{label}</span>
            </button>
          ))}
        </motion.div>

        {/* Tab Content */}
        <div className="tab-content">
          {activeTab === 'overview' && renderOverviewTab()}
          {activeTab === 'detailed' && renderDetailedTab()}
          {activeTab === 'metrics' && renderMetricsTab()}
          {activeTab === 'diagnostics' && renderDiagnosticsTab()}
        </div>
      </div>
    </div>
  )
}

export default SystemHealthDashboard