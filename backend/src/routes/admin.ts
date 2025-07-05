import { Router } from 'express';
import { r2BackupService } from '../services/R2BackupService.js';
import { duckdb } from '../database/duckdb-connection.js';
import { authenticateToken, requireAdmin, requireSuperAdmin } from '../middleware/auth.js';
import logger from '../utils/logger.js';

const router = Router();

/**
 * POST /api/admin/backup
 * Create a manual backup to R2
 * Requires admin authentication
 */
router.post('/backup', authenticateToken, requireAdmin, async (req, res) => {
  try {
    if (!r2BackupService.isEnabled()) {
      return res.status(503).json({
        success: false,
        error: 'R2 backup service is disabled',
        message: 'Cannot create backup when R2 sync is disabled'
      });
    }

    logger.info('Manual backup requested via API');
    const backupId = await r2BackupService.createManualBackup();

    res.json({
      success: true,
      message: 'Manual backup created successfully',
      data: {
        backupId,
        type: 'manual',
        timestamp: new Date().toISOString()
      }
    });
  } catch (error) {
    logger.error('Manual backup failed via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to create manual backup',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

/**
 * POST /api/admin/restore
 * Restore database from R2 backup
 * Requires super admin authentication
 */
router.post('/restore', authenticateToken, requireSuperAdmin, async (req, res) => {
  try {
    if (!r2BackupService.isEnabled()) {
      return res.status(503).json({
        success: false,
        error: 'R2 backup service is disabled',
        message: 'Cannot restore when R2 sync is disabled'
      });
    }

    const { backupId, force = false } = req.body;

    logger.info('Database restore requested via API', { backupId, force });

    let restoredBackup: string | null;
    if (backupId) {
      // Restore specific backup (not implemented in R2BackupService yet)
      return res.status(400).json({
        success: false,
        error: 'Specific backup restore not implemented',
        message: 'Currently only latest backup restore is supported'
      });
    } else {
      // Restore latest backup
      restoredBackup = await r2BackupService.restoreLatestBackup(force);
    }

    if (restoredBackup) {
      res.json({
        success: true,
        message: 'Database restored successfully',
        data: {
          restoredBackup,
          timestamp: new Date().toISOString(),
          force
        }
      });
    } else {
      res.json({
        success: true,
        message: 'No restore needed - local database is up to date',
        data: {
          timestamp: new Date().toISOString(),
          force
        }
      });
    }
  } catch (error) {
    logger.error('Database restore failed via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to restore database',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

/**
 * GET /api/admin/backups
 * List available backups in R2
 * Requires admin authentication
 */
router.get('/backups', authenticateToken, requireAdmin, async (req, res) => {
  try {
    if (!r2BackupService.isEnabled()) {
      return res.status(503).json({
        success: false,
        error: 'R2 backup service is disabled',
        data: []
      });
    }

    const limit = parseInt(req.query.limit as string) || 20;
    const backups = await r2BackupService.listBackups(limit);

    res.json({
      success: true,
      message: `Found ${backups.length} backups`,
      data: backups.map(backup => ({
        id: backup.id,
        type: backup.type,
        timestamp: backup.timestamp,
        size: backup.size,
        schemaVersion: backup.schema_version,
        status: backup.status
      })),
      meta: {
        limit,
        count: backups.length
      }
    });
  } catch (error) {
    logger.error('Failed to list backups via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to list backups',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

/**
 * POST /api/admin/cleanup
 * Clean up old backups based on retention policy
 * Requires admin authentication
 */
router.post('/cleanup', authenticateToken, requireAdmin, async (req, res) => {
  try {
    if (!r2BackupService.isEnabled()) {
      return res.status(503).json({
        success: false,
        error: 'R2 backup service is disabled',
        message: 'Cannot cleanup when R2 sync is disabled'
      });
    }

    logger.info('Backup cleanup requested via API');
    await r2BackupService.cleanupOldBackups();

    res.json({
      success: true,
      message: 'Backup cleanup completed successfully',
      data: {
        timestamp: new Date().toISOString(),
        retentionDays: parseInt(process.env.R2_BACKUP_RETENTION_DAYS || '30')
      }
    });
  } catch (error) {
    logger.error('Backup cleanup failed via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to cleanup backups',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

/**
 * GET /api/admin/database/stats
 * Get database statistics and information
 */
router.get('/database/stats', async (req, res) => {
  try {
    // Get server stats
    const serverStats = await duckdb.getServerStats();
    
    // Get table information
    const tablesResult = await duckdb.query(`
      SELECT table_name, 
             (SELECT COUNT(*) FROM information_schema.columns c WHERE c.table_name = t.table_name) as column_count
      FROM information_schema.tables t 
      WHERE table_schema = 'main'
      ORDER BY table_name
    `);

    // Get schema version
    let schemaVersion = 'unknown';
    try {
      const versionResult = await duckdb.query('SELECT version FROM schema_migrations ORDER BY id DESC LIMIT 1');
      if (versionResult.rows.length > 0) {
        schemaVersion = versionResult.rows[0].version;
      }
    } catch (error) {
      // schema_migrations table might not exist in older versions
    }

    res.json({
      success: true,
      data: {
        schemaVersion,
        serverStats: serverStats ? {
          startCount: serverStats.start_count,
          firstStartTime: serverStats.first_start_time,
          lastStartTime: serverStats.last_start_time,
          lastStopTime: serverStats.last_stop_time,
          totalLifetime: serverStats.total_lifetime,
          lastSessionDuration: serverStats.last_session_duration
        } : null,
        tables: tablesResult.rows.map((row: any) => ({
          name: row.table_name,
          columnCount: row.column_count
        })),
        meta: {
          totalTables: tablesResult.rows.length,
          timestamp: new Date().toISOString()
        }
      }
    });
  } catch (error) {
    logger.error('Failed to get database stats via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to get database statistics',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

/**
 * POST /api/admin/database/checkpoint
 * Manually trigger database checkpoint/flush
 */
router.post('/database/checkpoint', async (req, res) => {
  try {
    // DuckDB automatically manages checkpoints, but we can force a checkpoint
    await duckdb.query('CHECKPOINT');
    
    logger.info('Database checkpoint triggered via API');
    
    res.json({
      success: true,
      message: 'Database checkpoint completed successfully',
      data: {
        timestamp: new Date().toISOString()
      }
    });
  } catch (error) {
    logger.error('Database checkpoint failed via API:', error);
    res.status(500).json({
      success: false,
      error: 'Failed to checkpoint database',
      message: error instanceof Error ? error.message : 'Unknown error occurred'
    });
  }
});

export default router;