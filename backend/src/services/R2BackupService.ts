import { S3Client, PutObjectCommand, GetObjectCommand, ListObjectsV2Command, DeleteObjectCommand } from '@aws-sdk/client-s3';
import { Upload } from '@aws-sdk/lib-storage';
import { createReadStream, createWriteStream, existsSync, statSync } from 'fs';
import { join, basename } from 'path';
import logger from '../utils/logger.js';

interface BackupMetadata {
  type: 'shutdown' | 'manual' | 'periodic';
  timestamp: string;
  size: number;
  schema_version?: string;
  status?: 'latest_restored';
}

interface BackupInfo {
  id: string;
  type: 'shutdown' | 'manual' | 'periodic';
  timestamp: string;
  size: string;
  schema_version?: string;
  status?: 'latest_restored';
}

export class R2BackupService {
  private s3Client: S3Client | null = null;
  private bucketName: string = '';
  private backupPath: string = '';
  private enabled: boolean;
  private periodicBackupInterval: NodeJS.Timeout | null = null;

  constructor() {
    this.enabled = process.env.R2_SYNC_ENABLED === 'true';
    
    if (!this.enabled) {
      logger.info('📦 R2 backup service disabled');
      return;
    }

    // Validate required environment variables
    const requiredEnvs = [
      'R2_ACCESS_KEY_ID', 
      'R2_SECRET_ACCESS_KEY', 
      'R2_BUCKET_NAME', 
      'R2_ENDPOINT'
    ];
    const missingEnvs = requiredEnvs.filter(env => !process.env[env]);
    
    if (missingEnvs.length > 0) {
      logger.warn(`⚠️  R2 backup disabled: Missing environment variables: ${missingEnvs.join(', ')}`);
      this.enabled = false;
      return;
    }

    this.bucketName = process.env.R2_BUCKET_NAME!;
    this.backupPath = process.env.R2_BACKUP_PATH || 'backups/';
    
    // Initialize S3 client for R2
    this.s3Client = new S3Client({
      region: process.env.R2_REGION || 'auto',
      endpoint: process.env.R2_ENDPOINT!,
      credentials: {
        accessKeyId: process.env.R2_ACCESS_KEY_ID!,
        secretAccessKey: process.env.R2_SECRET_ACCESS_KEY!,
      },
    });

    logger.info('✅ R2BackupService initialized', {
      bucket: this.bucketName,
      backupPath: this.backupPath,
      enabled: this.enabled
    });
  }

  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Create a backup on graceful shutdown
   */
  async backupOnShutdown(): Promise<string | null> {
    if (!this.enabled || !this.s3Client) {
      logger.warn('Backup attempted but service is disabled or not configured');
      return null;
    }

    try {
      logger.info('🔄 Starting shutdown backup...');
      const backupId = await this.createBackup('shutdown');
      logger.info('✅ Shutdown backup completed successfully', { backupId });
      return backupId;
    } catch (error) {
      logger.error('❌ Shutdown backup failed', { error: error instanceof Error ? error.message : error });
      return null;
    }
  }

  /**
   * Create a manual backup
   */
  async createManualBackup(): Promise<string> {
    if (!this.enabled || !this.s3Client) {
      throw new Error('Backup service is disabled or not configured');
    }

    logger.info('🔄 Starting manual backup...');
    const backupId = await this.createBackup('manual');
    logger.info('✅ Manual backup completed successfully', { backupId });
    return backupId;
  }

  /**
   * Create a periodic backup
   */
  async createPeriodicBackup(): Promise<string> {
    if (!this.enabled || !this.s3Client) {
      throw new Error('Backup service is disabled or not configured');
    }

    logger.info('⏰ Starting periodic backup...');
    const backupId = await this.createBackup('periodic');
    logger.info('✅ Periodic backup completed successfully', { backupId });
    return backupId;
  }

  /**
   * Create a backup of the current database
   */
  private async createBackup(type: 'shutdown' | 'manual' | 'periodic'): Promise<string> {
    const dbPath = join(process.cwd(), 'hesocial.duckdb');
    
    if (!existsSync(dbPath)) {
      throw new Error(`Database file not found: ${dbPath}`);
    }

    // Generate backup filename with timestamp
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5); // YYYY-MM-DDTHH-MM-SS
    const backupId = `hesocial-${type}-${timestamp}.duckdb`;
    const r2Key = `${this.backupPath.replace(/^\//, '')}${backupId}`;

    // Get file stats
    const stats = statSync(dbPath);
    const metadata: BackupMetadata = {
      type,
      timestamp: new Date().toISOString(),
      size: stats.size,
      schema_version: 'v1.0.0' // TODO: Get from database
    };

    // Upload to R2
    await this.uploadToR2(dbPath, r2Key, metadata);
    
    return backupId;
  }

  /**
   * Upload file to R2
   */
  private async uploadToR2(localPath: string, r2Key: string, metadata: BackupMetadata): Promise<void> {
    const fileStream = createReadStream(localPath);
    const stats = statSync(localPath);

    logger.info('📤 Uploading to R2...', {
      localPath,
      r2Key,
      size: `${(stats.size / 1024 / 1024).toFixed(2)}MB`,
      type: metadata.type
    });

    if (!this.s3Client) {
      throw new Error('S3 client not initialized');
    }

    const upload = new Upload({
      client: this.s3Client,
      params: {
        Bucket: this.bucketName,
        Key: r2Key,
        Body: fileStream,
        ContentType: 'application/octet-stream',
        Metadata: {
          type: metadata.type,
          timestamp: metadata.timestamp,
          size: metadata.size.toString(),
          schema_version: metadata.schema_version || 'unknown'
        }
      }
    });

    await upload.done();
    
    logger.info('✅ Successfully uploaded to R2', { r2Key, size: `${(stats.size / 1024 / 1024).toFixed(2)}MB` });
  }

  /**
   * Download backup from R2
   */
  async downloadFromR2(backupId: string, localPath: string): Promise<void> {
    if (!this.enabled || !this.s3Client) {
      throw new Error('Backup service is disabled');
    }

    const r2Key = `${this.backupPath.replace(/^\//, '')}${backupId}`;
    
    logger.info('📥 Downloading from R2...', { backupId, r2Key, localPath });

    const command = new GetObjectCommand({
      Bucket: this.bucketName,
      Key: r2Key
    });

    const response = await this.s3Client.send(command);
    
    if (!response.Body) {
      throw new Error('No data received from R2');
    }

    // Write to local file
    const writeStream = createWriteStream(localPath);
    
    await new Promise((resolve, reject) => {
      if (response.Body instanceof ReadableStream) {
        // Node.js ReadableStream handling
        const nodeStream = response.Body as any;
        nodeStream.pipe(writeStream);
        nodeStream.on('end', resolve);
        nodeStream.on('error', reject);
        writeStream.on('error', reject);
      } else {
        reject(new Error('Unexpected response body type'));
      }
    });

    logger.info('✅ Successfully downloaded from R2', { backupId, localPath });
  }

  /**
   * Restore latest backup from R2 with smart restore logic
   */
  async restoreLatestBackup(forceRestore: boolean = false): Promise<string | null> {
    if (!this.enabled || !this.s3Client) {
      logger.warn('Restore attempted but R2 service is disabled');
      return null;
    }

    try {
      const backups = await this.listBackups(10);
      
      if (backups.length === 0) {
        logger.info('📭 No backups found to restore');
        return null;
      }

      const dbPath = join(process.cwd(), 'hesocial.duckdb');
      
      // Check if local database exists
      if (existsSync(dbPath) && !forceRestore) {
        const localStats = statSync(dbPath);
        const latestBackup = backups[0];
        const backupTime = new Date(latestBackup.timestamp);
        const localTime = localStats.mtime;
        
        logger.info('🔍 Comparing database timestamps:');
        logger.info(`   Local DB: ${localTime.toISOString()} (${(localStats.size / 1024).toFixed(1)} KB)`);
        logger.info(`   Latest backup: ${backupTime.toISOString()} (${latestBackup.size})`);
        
        if (localTime > backupTime) {
          logger.info('✨ Local database is newer than backup - keeping local version');
          return null;
        } else {
          logger.info('📥 Backup is newer than local database - restoring...');
        }
      } else if (!existsSync(dbPath)) {
        logger.info('🆕 No local database found - restoring from backup...');
      } else {
        logger.info('🔄 Force restore requested - overwriting local database...');
      }

      // Get the most recent backup (already sorted by date)
      const latestBackup = backups[0];
      
      logger.info('📦 Restoring backup...', { 
        backupId: latestBackup.id, 
        type: latestBackup.type,
        size: latestBackup.size,
        schema_version: latestBackup.schema_version
      });
      
      // Download and restore
      await this.downloadFromR2(latestBackup.id, dbPath);
      await this.updateBackupStatus(latestBackup.id, 'latest_restored');
      
      logger.info('✅ Backup restored successfully', { backupId: latestBackup.id });
      return latestBackup.id;
      
    } catch (error) {
      logger.error('❌ Failed to restore latest backup:', error);
      return null;
    }
  }

  /**
   * List available backups
   */
  async listBackups(limit = 20): Promise<BackupInfo[]> {
    if (!this.enabled || !this.s3Client) {
      throw new Error('Backup service is disabled');
    }

    const command = new ListObjectsV2Command({
      Bucket: this.bucketName,
      Prefix: this.backupPath.replace(/^\//, ''),
      MaxKeys: limit
    });

    const response = await this.s3Client.send(command);
    
    if (!response.Contents) {
      return [];
    }

    const backups: BackupInfo[] = response.Contents
      .filter(obj => obj.Key?.endsWith('.duckdb'))
      .map(obj => {
        const filename = basename(obj.Key!);
        const size = obj.Size ? `${(obj.Size / 1024 / 1024).toFixed(2)}MB` : 'Unknown';
        
        // Parse type from filename
        let type: 'shutdown' | 'manual' | 'periodic' = 'manual';
        if (filename.includes('-shutdown-')) {
          type = 'shutdown';
        } else if (filename.includes('-periodic-')) {
          type = 'periodic';
        }

        return {
          id: filename,
          type,
          timestamp: obj.LastModified?.toISOString() || '',
          size,
          schema_version: 'v1.0.0' // TODO: Extract from metadata
        };
      })
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

    return backups;
  }

  /**
   * Update backup status (e.g., mark as restored)
   */
  private async updateBackupStatus(backupId: string, status: 'latest_restored'): Promise<void> {
    // For now, just log. In the future, this could update metadata or a tracking table
    logger.info('📝 Backup status updated', { backupId, status });
  }

  /**
   * Clean up old backups based on retention policy
   */
  async cleanupOldBackups(): Promise<void> {
    if (!this.enabled || !this.s3Client) {
      return;
    }

    const retentionDays = parseInt(process.env.R2_BACKUP_RETENTION_DAYS || '30');
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - retentionDays);

    logger.info('🧹 Starting backup cleanup...', { retentionDays, cutoffDate });

    const allBackups = await this.listBackups(1000);
    const oldBackups = allBackups.filter(backup => 
      new Date(backup.timestamp) < cutoffDate
    );

    if (oldBackups.length === 0) {
      logger.info('✨ No old backups to clean up');
      return;
    }

    logger.info(`🗑️  Cleaning up ${oldBackups.length} old backups`);

    for (const backup of oldBackups) {
      try {
        const r2Key = `${this.backupPath.replace(/^\//, '')}${backup.id}`;
        
        const command = new DeleteObjectCommand({
          Bucket: this.bucketName,
          Key: r2Key
        });

        await this.s3Client.send(command);
        logger.info('🗑️  Deleted old backup', { backupId: backup.id });
      } catch (error) {
        logger.error('❌ Failed to delete old backup', { 
          backupId: backup.id, 
          error: error instanceof Error ? error.message : error 
        });
      }
    }

    logger.info('✅ Backup cleanup completed');
  }

  /**
   * Test R2 connectivity
   */
  async testConnection(): Promise<boolean> {
    if (!this.enabled || !this.s3Client) {
      return false;
    }

    try {
      const command = new ListObjectsV2Command({
        Bucket: this.bucketName,
        MaxKeys: 1
      });

      await this.s3Client.send(command);
      logger.info('✅ R2 connection test successful');
      return true;
    } catch (error) {
      logger.error('❌ R2 connection test failed', { 
        error: error instanceof Error ? error.message : error 
      });
      return false;
    }
  }

  /**
   * Start periodic backups with configurable interval
   */
  startPeriodicBackups(): void {
    if (!this.enabled) {
      logger.info('📦 Periodic backups disabled - R2 service not enabled');
      return;
    }

    const periodicBackupEnabled = process.env.R2_PERIODIC_BACKUP_ENABLED === 'true';
    if (!periodicBackupEnabled) {
      logger.info('📦 Periodic backups disabled via configuration');
      return;
    }

    const intervalHours = parseInt(process.env.R2_PERIODIC_BACKUP_INTERVAL_HOURS || '24');
    const intervalMs = intervalHours * 60 * 60 * 1000;

    logger.info('⏰ Starting periodic backup scheduler', { intervalHours });

    this.periodicBackupInterval = setInterval(async () => {
      try {
        await this.createPeriodicBackup();
        // Clean up old backups after each periodic backup
        await this.cleanupOldBackups();
      } catch (error) {
        logger.error('❌ Periodic backup failed', { 
          error: error instanceof Error ? error.message : error 
        });
      }
    }, intervalMs);
  }

  /**
   * Stop periodic backups
   */
  stopPeriodicBackups(): void {
    if (this.periodicBackupInterval) {
      clearInterval(this.periodicBackupInterval);
      this.periodicBackupInterval = null;
      logger.info('⏹️  Periodic backup scheduler stopped');
    }
  }

  /**
   * Get backup service status and metrics
   */
  async getStatus(): Promise<{
    enabled: boolean;
    lastBackup?: string;
    backupCount: number;
    connectionHealthy: boolean;
    periodicBackupsEnabled?: boolean;
    periodicBackupInterval?: number;
  }> {
    if (!this.enabled) {
      return {
        enabled: false,
        backupCount: 0,
        connectionHealthy: false,
        periodicBackupsEnabled: false
      };
    }

    const periodicBackupEnabled = process.env.R2_PERIODIC_BACKUP_ENABLED === 'true';
    const intervalHours = parseInt(process.env.R2_PERIODIC_BACKUP_INTERVAL_HOURS || '24');

    try {
      const connectionHealthy = await this.testConnection();
      const backups = await this.listBackups(5);
      
      return {
        enabled: true,
        lastBackup: backups.length > 0 ? backups[0].timestamp : undefined,
        backupCount: backups.length,
        connectionHealthy,
        periodicBackupsEnabled: periodicBackupEnabled,
        periodicBackupInterval: periodicBackupEnabled ? intervalHours : undefined
      };
    } catch (error) {
      return {
        enabled: true,
        backupCount: 0,
        connectionHealthy: false,
        periodicBackupsEnabled: periodicBackupEnabled,
        periodicBackupInterval: periodicBackupEnabled ? intervalHours : undefined
      };
    }
  }
}

// Export singleton instance
export const r2BackupService = new R2BackupService();