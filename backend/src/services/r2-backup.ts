import { S3Client, PutObjectCommand, GetObjectCommand, ListObjectsV2Command, DeleteObjectCommand } from '@aws-sdk/client-s3';
import { Upload } from '@aws-sdk/lib-storage';
import { createReadStream, createWriteStream, existsSync, statSync } from 'fs';
import { join, basename } from 'path';
import logger from '../utils/logger.js';

interface BackupMetadata {
  type: 'shutdown' | 'manual';
  timestamp: string;
  size: number;
  status?: 'latest_restored';
}

interface BackupInfo {
  id: string;
  type: 'shutdown' | 'manual';
  timestamp: string;
  size: string;
  status?: 'latest_restored';
}

export class R2BackupService {
  private s3Client: S3Client;
  private bucketName: string;
  private backupPath: string;
  private enabled: boolean;

  constructor() {
    this.enabled = process.env.BACKUP_ENABLED === 'true';
    
    if (!this.enabled) {
      logger.info('R2 backup service disabled');
      return;
    }

    // Validate required environment variables
    const requiredEnvs = ['R2_ACCOUNT_ID', 'R2_ACCESS_KEY_ID', 'R2_SECRET_ACCESS_KEY', 'R2_BUCKET_NAME', 'R2_ENDPOINT'];
    const missingEnvs = requiredEnvs.filter(env => !process.env[env]);
    
    if (missingEnvs.length > 0) {
      throw new Error(`Missing required R2 environment variables: ${missingEnvs.join(', ')}`);
    }

    this.bucketName = process.env.R2_BUCKET_NAME!;
    this.backupPath = process.env.R2_BACKUP_PATH || '/backups/';
    
    // Initialize S3 client for R2
    this.s3Client = new S3Client({
      region: process.env.R2_REGION || 'auto',
      endpoint: process.env.R2_ENDPOINT!,
      credentials: {
        accessKeyId: process.env.R2_ACCESS_KEY_ID!,
        secretAccessKey: process.env.R2_SECRET_ACCESS_KEY!,
      },
    });

    logger.info('R2BackupService initialized', {
      bucket: this.bucketName,
      backupPath: this.backupPath,
      enabled: this.enabled
    });
  }

  /**
   * Create a backup on graceful shutdown
   */
  async backupOnShutdown(): Promise<string | null> {
    if (!this.enabled) {
      logger.warn('Backup attempted but service is disabled');
      return null;
    }

    try {
      logger.info('Starting shutdown backup...');
      const backupId = await this.createBackup('shutdown');
      logger.info('Shutdown backup completed successfully', { backupId });
      return backupId;
    } catch (error) {
      logger.error('Shutdown backup failed', { error: error instanceof Error ? error.message : error });
      return null;
    }
  }

  /**
   * Create a manual backup
   */
  async createManualBackup(): Promise<string> {
    if (!this.enabled) {
      throw new Error('Backup service is disabled');
    }

    logger.info('Starting manual backup...');
    const backupId = await this.createBackup('manual');
    logger.info('Manual backup completed successfully', { backupId });
    return backupId;
  }

  /**
   * Create a backup of the current database
   */
  private async createBackup(type: 'shutdown' | 'manual'): Promise<string> {
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
      size: stats.size
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

    logger.info('Uploading to R2...', {
      localPath,
      r2Key,
      size: `${(stats.size / 1024 / 1024).toFixed(2)}MB`
    });

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
          size: metadata.size.toString()
        }
      }
    });

    await upload.done();
    
    logger.info('Successfully uploaded to R2', { r2Key });
  }

  /**
   * Download backup from R2
   */
  async downloadFromR2(backupId: string, localPath: string): Promise<void> {
    if (!this.enabled) {
      throw new Error('Backup service is disabled');
    }

    const r2Key = `${this.backupPath.replace(/^\//, '')}${backupId}`;
    
    logger.info('Downloading from R2...', { backupId, r2Key, localPath });

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
        // Node.js ReadableStream
        const nodeStream = response.Body as any;
        nodeStream.pipe(writeStream);
        nodeStream.on('end', resolve);
        nodeStream.on('error', reject);
      } else {
        reject(new Error('Unexpected response body type'));
      }
    });

    logger.info('Successfully downloaded from R2', { backupId, localPath });
  }

  /**
   * List available backups
   */
  async listBackups(limit = 20): Promise<BackupInfo[]> {
    if (!this.enabled) {
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
        let type: 'shutdown' | 'manual' = 'manual';
        if (filename.includes('-shutdown-')) {
          type = 'shutdown';
        }

        return {
          id: filename,
          type,
          timestamp: obj.LastModified?.toISOString() || '',
          size,
          // Note: status would need to be stored in metadata or separate tracking
        };
      })
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

    return backups;
  }

  /**
   * Update backup status (e.g., mark as restored)
   */
  async updateBackupStatus(backupId: string, status: 'latest_restored'): Promise<void> {
    if (!this.enabled) {
      throw new Error('Backup service is disabled');
    }

    // For simplicity, we'll log this. In a more complex system, 
    // you might store metadata in a separate file or database
    logger.info('Backup status updated', { backupId, status });
    
    // TODO: Implement metadata storage for status tracking
    // This could be done by:
    // 1. Storing a metadata.json file in R2
    // 2. Using object tags (if supported by R2)
    // 3. Using a local database table for backup metadata
  }

  /**
   * Clean up old backups based on retention policy
   */
  async cleanupOldBackups(): Promise<void> {
    if (!this.enabled) {
      return;
    }

    const retentionDays = parseInt(process.env.BACKUP_RETENTION_DAYS || '30');
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - retentionDays);

    logger.info('Starting backup cleanup...', { retentionDays, cutoffDate });

    const allBackups = await this.listBackups(1000); // Get more for cleanup
    const oldBackups = allBackups.filter(backup => 
      new Date(backup.timestamp) < cutoffDate
    );

    if (oldBackups.length === 0) {
      logger.info('No old backups to clean up');
      return;
    }

    logger.info(`Cleaning up ${oldBackups.length} old backups`);

    for (const backup of oldBackups) {
      try {
        const r2Key = `${this.backupPath.replace(/^\//, '')}${backup.id}`;
        
        const command = new DeleteObjectCommand({
          Bucket: this.bucketName,
          Key: r2Key
        });

        await this.s3Client.send(command);
        logger.info('Deleted old backup', { backupId: backup.id });
      } catch (error) {
        logger.error('Failed to delete old backup', { 
          backupId: backup.id, 
          error: error instanceof Error ? error.message : error 
        });
      }
    }

    logger.info('Backup cleanup completed');
  }

  /**
   * Smart restore logic - only restore if backup is newer than local DB
   */
  async smartRestore(forceRestore: boolean = false): Promise<boolean> {
    if (!this.enabled) {
      logger.info('Backup service disabled, skipping smart restore');
      return false;
    }

    try {
      const dbPath = join(process.cwd(), 'hesocial.duckdb');
      const localDbExists = existsSync(dbPath);
      
      // Get latest backup
      const backups = await this.listBackups(1);
      if (backups.length === 0) {
        logger.info('No backups found, skipping restore');
        return false;
      }

      const latestBackup = backups[0];
      const backupTimestamp = new Date(latestBackup.timestamp);
      
      // If no local DB exists, restore from backup
      if (!localDbExists) {
        logger.info('No local database found, restoring from backup', { 
          backupId: latestBackup.id,
          backupTimestamp: backupTimestamp.toISOString()
        });
        await this.downloadFromR2(latestBackup.id, dbPath);
        await this.updateBackupStatus(latestBackup.id, 'latest_restored');
        return true;
      }

      // Compare timestamps if local DB exists
      const localDbStats = statSync(dbPath);
      const localDbTimestamp = localDbStats.mtime;
      
      logger.info('Comparing timestamps for smart restore', {
        localDbTimestamp: localDbTimestamp.toISOString(),
        backupTimestamp: backupTimestamp.toISOString(),
        forceRestore
      });

      // Only restore if backup is newer or force restore is requested
      if (forceRestore || backupTimestamp > localDbTimestamp) {
        logger.info('Restoring from backup', { 
          reason: forceRestore ? 'forced' : 'backup is newer',
          backupId: latestBackup.id
        });
        
        // Create backup of current local DB before restoring
        const backupPath = `${dbPath}.backup-${Date.now()}`;
        await this.createLocalBackup(dbPath, backupPath);
        
        await this.downloadFromR2(latestBackup.id, dbPath);
        await this.updateBackupStatus(latestBackup.id, 'latest_restored');
        return true;
      } else {
        logger.info('Local database is up-to-date, skipping restore');
        return false;
      }
    } catch (error) {
      logger.error('Smart restore failed', { 
        error: error instanceof Error ? error.message : error 
      });
      return false;
    }
  }

  /**
   * Create local backup copy
   */
  private async createLocalBackup(sourcePath: string, backupPath: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const readStream = createReadStream(sourcePath);
      const writeStream = createWriteStream(backupPath);
      
      readStream.pipe(writeStream);
      readStream.on('error', reject);
      writeStream.on('error', reject);
      writeStream.on('finish', () => {
        logger.info('Local backup created', { sourcePath, backupPath });
        resolve();
      });
    });
  }

  /**
   * Get database file timestamp
   */
  getDatabaseTimestamp(): Date | null {
    const dbPath = join(process.cwd(), 'hesocial.duckdb');
    if (!existsSync(dbPath)) {
      return null;
    }
    const stats = statSync(dbPath);
    return stats.mtime;
  }

  /**
   * Test R2 connectivity
   */
  async testConnection(): Promise<boolean> {
    if (!this.enabled) {
      return false;
    }

    try {
      const command = new ListObjectsV2Command({
        Bucket: this.bucketName,
        MaxKeys: 1
      });

      await this.s3Client.send(command);
      logger.info('R2 connection test successful');
      return true;
    } catch (error) {
      logger.error('R2 connection test failed', { 
        error: error instanceof Error ? error.message : error 
      });
      return false;
    }
  }
}