# Cloudflare R2 Simplified Implementation Plan

## Overview
Simple, practical DuckDB backup solution using Cloudflare R2 with graceful shutdown auto-backup and manual web triggers. No complex scheduling or separate buckets.

## Architecture

### Environment-Based Bucket Structure
```
# Development Environment
hesocial-duckdb-dev/
├── backups/
│   ├── hesocial-backup-20241230-120000.duckdb  # Tags: type=shutdown
│   ├── hesocial-backup-20241229-180000.duckdb  # Tags: type=shutdown, status=latest_restored
│   └── hesocial-manual-20241230-153000.duckdb  # Tags: type=manual
└── assets/                                      # Future: user uploads, etc.

# Production Environment  
hesocial-duckdb/
├── backups/
│   ├── hesocial-backup-20241230-120000.duckdb  # Production backups
│   └── hesocial-manual-20241230-153000.duckdb
└── assets/                                      # Production assets
```

### Local vs R2 Storage
```
Local Server:     hesocial.duckdb (active database)
R2 Cloud:         /backups/*.duckdb (backup copies only)
```

### Data Flow
```
Local DuckDB → Graceful Shutdown → R2 Backup
           ↘ Manual Trigger → R2 Backup
```

## Backup Strategy

### 1. Auto-Backup on Graceful Shutdown
- **Trigger**: SIGTERM, SIGINT, process exit
- **Naming**: `hesocial-backup-YYYYMMDD-HHMMSS.duckdb`
- **Location**: `hesocial-production/database/`

### 2. Manual Backup via Web API
- **Endpoint**: `POST /api/admin/backup`
- **Naming**: `hesocial-manual-YYYYMMDD-HHMMSS.duckdb`
- **Use case**: Before deployments, schema changes, manual safety

### 3. Data Loss Tolerance
- **Acceptable loss**: 6-8 hours of data
- **Reasoning**: Event data changes slowly, registrations are infrequent
- **Risk level**: Low for social event platform

## Implementation Plan

### Phase 1: R2 Setup (Manual)
- [ ] Create `hesocial-production` bucket in Cloudflare
- [ ] Generate R2 API tokens with edit permissions
- [ ] Configure lifecycle rules (delete files older than 90 days)
- [ ] Test connectivity

### Phase 2: Backend Integration (Code)
- [ ] Install AWS SDK v3 in backend
- [ ] Add R2 environment configuration
- [ ] Create backup service class
- [ ] Implement graceful shutdown hook

### Phase 3: API & Testing (Code)
- [ ] Create manual backup API endpoint
- [ ] Add error handling and logging
- [ ] Test backup and restore functionality
- [ ] Update deployment procedures

## Technical Implementation

### Environment Configuration
```bash
# R2 Configuration
R2_ACCOUNT_ID=your-account-id
R2_ACCESS_KEY_ID=your-access-key
R2_SECRET_ACCESS_KEY=your-secret-key
R2_BUCKET_NAME=hesocial-duckdb-dev  # or hesocial-duckdb for production
R2_ENDPOINT=https://account-id.r2.cloudflarestorage.com
R2_REGION=auto

# Backup Configuration
BACKUP_ENABLED=true
R2_BACKUP_PATH=/backups/
BACKUP_RETENTION_DAYS=30
```

### Backup Service Architecture
```typescript
class R2BackupService {
  async backupOnShutdown(): Promise<string>
  async createManualBackup(): Promise<string>
  async uploadToR2(localPath: string, r2Key: string): Promise<void>
  async downloadFromR2(r2Key: string, localPath: string): Promise<void>
  async listBackups(): Promise<BackupInfo[]>
  async cleanupOldBackups(): Promise<void>
}
```

### API Endpoints
```typescript
// Manual backup trigger
POST /api/admin/backup
Response: { 
  success: true, 
  backupId: "hesocial-manual-20241230-153000.duckdb",
  timestamp: "2024-12-30T15:30:00Z",
  size: "142MB",
  tags: { type: "manual" }
}

// List available backups
GET /api/admin/backups
Response: {
  backups: [
    {
      id: "hesocial-backup-20241230-120000.duckdb",
      type: "shutdown",
      timestamp: "2024-12-30T12:00:00Z",
      size: "140MB"
    },
    {
      id: "hesocial-backup-20241229-180000.duckdb", 
      type: "shutdown",
      status: "latest_restored",
      timestamp: "2024-12-29T18:00:00Z",
      size: "138MB"
    }
  ]
}

// Update backup status (when restored)
PUT /api/admin/backup/:id/status
Body: { status: "latest_restored" }
```

### Graceful Shutdown Implementation
```typescript
// server.ts
process.on('SIGTERM', async () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  
  // Stop accepting new connections
  server.close();
  
  // Create backup
  await r2BackupService.backupOnShutdown();
  
  // Close database
  await duckdbConnection.close();
  
  process.exit(0);
});
```

## File Structure
```
backend/src/
├── services/
│   └── r2-backup.ts           # R2 backup service
├── routes/
│   └── admin.ts               # Admin API routes
├── utils/
│   └── r2-config.ts           # R2 configuration
└── server.ts                  # Graceful shutdown hooks
```

## Cost Analysis

### Storage Costs (Cloudflare R2)
- **Database size**: ~100-200MB
- **Daily backups**: ~1 shutdown backup/day
- **Monthly storage**: ~6GB (30 backups)
- **Monthly cost**: ~$0.09/month ($0.015/GB)

### Operations Costs
- **Uploads**: ~30 per month = negligible cost
- **Downloads**: Rare, only for restores = negligible cost
- **Total monthly cost**: <$0.10

## Security & Best Practices

### Access Control
- R2 bucket set to private
- API tokens with minimal required permissions
- Admin API protected by authentication
- Environment variables for all credentials

### Data Protection
- Automatic backup on shutdown prevents data loss
- Manual backup before risky operations
- Lifecycle rules prevent unlimited storage growth
- Local backup validation before upload

### Monitoring
- Log all backup operations
- Alert on backup failures
- Track backup sizes and frequency
- Monitor R2 storage usage

## Testing Strategy

### Backup Testing
1. **Shutdown backup**: Test graceful shutdown process
2. **Manual backup**: Test API endpoint functionality
3. **Large database**: Test with realistic data size
4. **Network failure**: Test retry and error handling

### Restore Testing
1. **Download backup**: Test R2 to local download
2. **Database validation**: Verify backup integrity
3. **Restore process**: Test complete restore workflow
4. **Rollback scenario**: Test emergency restore procedures

## Deployment Considerations

### Development
- Use local DuckDB file
- Mock R2 service for testing
- Validate backup/restore locally

### Production
- Enable R2 backups
- Configure proper credentials
- Set up monitoring and alerts
- Document restore procedures

### CI/CD Integration
```bash
# Before deployment
curl -X POST https://api.hesocial.com/api/admin/backup

# Deploy application
# New version automatically creates shutdown backup
```

## Success Criteria

- ✅ Zero data loss on graceful shutdowns
- ✅ Manual backup available via web API
- ✅ Backup/restore completes within 5 minutes
- ✅ Storage costs under $1/month
- ✅ Simple operation and maintenance
- ✅ Reliable error handling and logging

This simplified approach provides essential data protection without complexity, perfectly suited for the HeSocial platform's needs.