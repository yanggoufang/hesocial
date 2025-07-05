import { Router } from 'express';
import { r2BackupService } from '../services/R2BackupService.js';
import { duckdb } from '../database/duckdb-connection.js';
import logger from '../utils/logger.js';

const router = Router();

/**
 * GET /api/health/database
 * Database health check with detailed status
 */
router.get('/database', async (req, res) => {
  try {
    // Get database server stats
    const serverStats = await duckdb.getServerStats();
    
    // Test database connection
    const connectionTest = await duckdb.query('SELECT 1 as test');
    const isConnected = connectionTest.rows.length > 0;
    
    // Get table count
    const tablesResult = await duckdb.query("SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'main'");
    const tableCount = tablesResult.rows[0]?.count || 0;
    
    res.json({
      success: true,
      database: {
        type: 'DuckDB',
        connected: isConnected,
        tableCount,
        serverStats: serverStats ? {
          startCount: serverStats.start_count,
          firstStartTime: serverStats.first_start_time,
          lastStartTime: serverStats.last_start_time,
          totalLifetime: Math.floor(serverStats.total_lifetime / 60), // minutes
          lastSessionDuration: Math.floor(serverStats.last_session_duration / 60) // minutes
        } : null
      },
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    logger.error('Database health check failed:', error);
    res.status(503).json({
      success: false,
      error: 'Database health check failed',
      database: {
        type: 'DuckDB',
        connected: false
      },
      timestamp: new Date().toISOString()
    });
  }
});

/**
 * GET /api/health/r2-sync
 * R2 backup service health check and status
 */
router.get('/r2-sync', async (req, res) => {
  try {
    if (!r2BackupService.isEnabled()) {
      return res.json({
        success: true,
        r2Sync: {
          enabled: false,
          status: 'disabled',
          message: 'R2 sync is disabled'
        },
        timestamp: new Date().toISOString()
      });
    }

    // Get R2 service status
    const r2Status = await r2BackupService.getStatus();
    
    // Get recent backups
    const recentBackups = await r2BackupService.listBackups(5);
    
    res.json({
      success: true,
      r2Sync: {
        enabled: r2Status.enabled,
        connectionHealthy: r2Status.connectionHealthy,
        lastBackup: r2Status.lastBackup,
        backupCount: r2Status.backupCount,
        recentBackups: recentBackups.map(backup => ({
          id: backup.id,
          type: backup.type,
          timestamp: backup.timestamp,
          size: backup.size,
          schemaVersion: backup.schema_version
        })),
        status: r2Status.connectionHealthy ? 'healthy' : 'unhealthy'
      },
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    logger.error('R2 sync health check failed:', error);
    res.status(503).json({
      success: false,
      error: 'R2 sync health check failed',
      r2Sync: {
        enabled: r2BackupService.isEnabled(),
        status: 'error',
        connectionHealthy: false
      },
      timestamp: new Date().toISOString()
    });
  }
});

/**
 * GET /api/health/full
 * Comprehensive health check including database and R2 sync
 */
router.get('/full', async (req, res) => {
  try {
    // Database health
    let databaseHealth;
    try {
      const serverStats = await duckdb.getServerStats();
      const connectionTest = await duckdb.query('SELECT 1 as test');
      const tablesResult = await duckdb.query("SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'main'");
      
      databaseHealth = {
        connected: connectionTest.rows.length > 0,
        tableCount: tablesResult.rows[0]?.count || 0,
        uptime: serverStats ? Math.floor(serverStats.total_lifetime / 60) : 0,
        status: 'healthy'
      };
    } catch (error) {
      databaseHealth = {
        connected: false,
        status: 'unhealthy',
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }

    // R2 sync health
    let r2SyncHealth;
    if (!r2BackupService.isEnabled()) {
      r2SyncHealth = {
        enabled: false,
        status: 'disabled'
      };
    } else {
      try {
        const r2Status = await r2BackupService.getStatus();
        r2SyncHealth = {
          enabled: true,
          connectionHealthy: r2Status.connectionHealthy,
          lastBackup: r2Status.lastBackup,
          backupCount: r2Status.backupCount,
          status: r2Status.connectionHealthy ? 'healthy' : 'unhealthy'
        };
      } catch (error) {
        r2SyncHealth = {
          enabled: true,
          status: 'error',
          connectionHealthy: false,
          error: error instanceof Error ? error.message : 'Unknown error'
        };
      }
    }

    // Overall health status
    const overallHealthy = databaseHealth.status === 'healthy' && 
                          (r2SyncHealth.status === 'healthy' || r2SyncHealth.status === 'disabled');

    res.status(overallHealthy ? 200 : 503).json({
      success: overallHealthy,
      status: overallHealthy ? 'healthy' : 'unhealthy',
      services: {
        database: databaseHealth,
        r2Sync: r2SyncHealth
      },
      environment: process.env.NODE_ENV || 'development',
      version: '1.0.0',
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    logger.error('Full health check failed:', error);
    res.status(503).json({
      success: false,
      status: 'unhealthy',
      error: 'Health check failed',
      timestamp: new Date().toISOString()
    });
  }
});

export default router;