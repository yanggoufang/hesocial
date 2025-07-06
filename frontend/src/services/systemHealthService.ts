import axios from 'axios'

const api = axios.create({
  baseURL: 'http://localhost:5000/api',
  withCredentials: true
})

export interface SystemHealthData {
  overall: 'healthy' | 'warning' | 'error'
  timestamp: string
  system: {
    uptime: number
    memory: {
      used: number
      total: number
      usage: number
    }
    cpu: {
      usage: number
    }
    environment: string
    nodeVersion: string
  }
  database: {
    type: string
    connected: boolean
    tableCount: number
    userCount: number
    eventCount: number
    registrationCount: number
    connectionPool: {
      active: number
      idle: number
      total: number
    }
    serverStats?: {
      startCount: number
      firstStartTime: string
      lastStartTime: string
      totalLifetime: number
      lastSessionDuration: number
    }
    queries: {
      total: number
      slow: number
      failed: number
    }
  }
  r2Backup: {
    enabled: boolean
    connectionHealthy: boolean
    lastBackup: string | null
    backupCount: number
    totalBackupSize: number
    periodicBackupsEnabled: boolean
    periodicBackupInterval: number
    recentBackups: Array<{
      id: string
      type: string
      timestamp: string
      size: number
      schemaVersion: string
    }>
  }
  services: {
    authentication: {
      status: 'healthy' | 'warning' | 'error'
      oauthProviders: string[]
      jwtConfigured: boolean
    }
    payments: {
      status: 'healthy' | 'warning' | 'error'
      stripeConfigured: boolean
    }
    media: {
      status: 'healthy' | 'warning' | 'error'
      r2Configured: boolean
    }
  }
  migrations: {
    status: 'healthy' | 'warning' | 'error'
    applied: number
    pending: number
    latest: string
  }
  security: {
    httpsEnabled: boolean
    corsConfigured: boolean
    rateLimitingEnabled: boolean
    helmetEnabled: boolean
  }
  performance: {
    avgResponseTime: number
    totalRequests: number
    errorRate: number
  }
}

export interface DetailedHealthCheck {
  overall: 'healthy' | 'warning' | 'error'
  timestamp: string
  results: Array<{
    component: string
    status: 'healthy' | 'warning' | 'error'
    message: string
    details?: any
  }>
  criticalErrors: string[]
  warnings: string[]
}

export interface SystemMetrics {
  timestamp: string
  uptime: number
  memory: {
    rss: number
    heapTotal: number
    heapUsed: number
    external: number
    arrayBuffers: number
  }
  cpu: {
    user: number
    system: number
  }
  environment: string
  nodeVersion: string
  platform: string
  architecture: string
}

export interface SystemDiagnostics {
  test: string
  status: 'passed' | 'failed' | 'skipped'
  message: string
  timestamp: string
}

class SystemHealthService {
  /**
   * Get comprehensive system health data
   */
  async getSystemHealth(): Promise<SystemHealthData> {
    const response = await api.get('/system/health')
    return response.data.data
  }

  /**
   * Get detailed health check results
   */
  async getDetailedHealthCheck(): Promise<DetailedHealthCheck> {
    const response = await api.get('/system/health/detailed')
    return response.data.data
  }

  /**
   * Get system performance metrics
   */
  async getSystemMetrics(): Promise<SystemMetrics> {
    const response = await api.get('/system/metrics')
    return response.data.data
  }

  /**
   * Run system diagnostics
   */
  async runSystemDiagnostics(): Promise<SystemDiagnostics[]> {
    const response = await api.get('/system/diagnostics')
    return response.data.data
  }

  /**
   * Format bytes to human readable format
   */
  formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  /**
   * Format uptime to human readable format
   */
  formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = Math.floor(seconds % 60)

    if (days > 0) {
      return `${days}d ${hours}h ${minutes}m`
    } else if (hours > 0) {
      return `${hours}h ${minutes}m`
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`
    } else {
      return `${secs}s`
    }
  }

  /**
   * Get status color based on health status
   */
  getStatusColor(status: 'healthy' | 'warning' | 'error'): string {
    switch (status) {
      case 'healthy':
        return 'text-green-400'
      case 'warning':
        return 'text-yellow-400'
      case 'error':
        return 'text-red-400'
      default:
        return 'text-gray-400'
    }
  }

  /**
   * Get status background color
   */
  getStatusBgColor(status: 'healthy' | 'warning' | 'error'): string {
    switch (status) {
      case 'healthy':
        return 'bg-green-500/20'
      case 'warning':
        return 'bg-yellow-500/20'
      case 'error':
        return 'bg-red-500/20'
      default:
        return 'bg-gray-500/20'
    }
  }

  /**
   * Get status icon
   */
  getStatusIcon(status: 'healthy' | 'warning' | 'error'): string {
    switch (status) {
      case 'healthy':
        return '✅'
      case 'warning':
        return '⚠️'
      case 'error':
        return '❌'
      default:
        return '❓'
    }
  }
}

export default new SystemHealthService()