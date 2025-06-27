# Cloudflare R2 DuckDB Persistence Implementation Plan

## Overview
Implement Cloudflare R2 as persistent storage for DuckDB database files, enabling data persistence across server restarts and deployments while maintaining local performance.

## Current State Analysis

### Existing DuckDB Setup
- **Database Path**: `hesocial.duckdb` in project root
- **Connection**: Single connection instance with local file storage
- **Schema**: Complete schema with users, events, venues, registrations, financial_verification
- **Lifecycle**: Basic startup/shutdown with server state tracking

### Requirements for R2 Integration
- Persist DuckDB database file to Cloudflare R2 object storage
- Maintain fast local performance for queries
- Ensure data durability across deployments
- Support backup and restore operations

## Architecture Strategy

### 1. Hybrid Local + Cloud Architecture
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │◄──►│  Local DuckDB   │◄──►│  Cloudflare R2  │
│                 │    │   (Fast I/O)    │    │ (Persistence)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

- **Local DuckDB**: Fast read/write operations on local filesystem
- **R2 Storage**: Persistent backup and restore mechanism  
- **Sync Strategy**: Periodic uploads and on-demand downloads

### 2. Core Components

#### R2 Database Sync Service
```typescript
class R2DatabaseSync {
  - uploadDatabase(): Promise<void>
  - downloadDatabase(): Promise<boolean>
  - checkRemoteExists(): Promise<boolean>
  - getLastModified(): Promise<Date>
  - validateDatabase(): Promise<boolean>
}
```

#### Enhanced DuckDB Connection
```typescript
class DuckDBConnection {
  - syncService: R2DatabaseSync
  - connect(): Promise<void>     // Download from R2 if needed
  - close(): Promise<void>       // Upload to R2 on shutdown
  - periodicSync(): void         // Background sync timer
}
```

### 3. Sync Mechanisms

#### Startup Sequence
1. Check if local database exists
2. If not, download latest from R2
3. If local exists, compare timestamps with R2
4. Use newer version or merge if needed
5. Initialize DuckDB connection

#### Periodic Backup
- Upload database to R2 every 30 minutes (configurable)
- Background process doesn't block operations
- Retry with exponential backoff on failures

#### Shutdown Sequence
1. Stop accepting new connections
2. Wait for active queries to complete
3. Upload final database state to R2
4. Close DuckDB connection

#### Manual Operations
- API endpoints for manual backup/restore
- Health check endpoint showing sync status
- Database export/import utilities

### 4. Environment Configuration

```bash
# Cloudflare R2 Configuration
R2_ACCOUNT_ID=your-account-id
R2_ACCESS_KEY_ID=your-access-key
R2_SECRET_ACCESS_KEY=your-secret-key
R2_BUCKET_NAME=hesocial-duckdb
R2_ENDPOINT=https://account-id.r2.cloudflarestorage.com
R2_REGION=auto

# Sync Configuration
R2_SYNC_ENABLED=true
R2_BACKUP_INTERVAL_MINUTES=30
R2_DATABASE_KEY=hesocial-production.duckdb
R2_BACKUP_RETENTION_DAYS=30
R2_COMPRESS_UPLOADS=true
R2_VALIDATE_DOWNLOADS=true
```

### 5. File Structure

```
backend/src/
├── database/
│   ├── duckdb-connection.ts      # Enhanced with R2 sync
│   ├── r2-sync.ts               # R2 sync service
│   └── r2-client.ts             # R2 client wrapper
├── utils/
│   └── r2-config.ts             # R2 configuration
└── routes/
    └── database-admin.ts         # Admin endpoints for sync
```

### 6. Error Handling & Recovery

#### Network Failures
- Retry with exponential backoff (1s, 2s, 4s, 8s, 16s)
- Fall back to local operations if R2 unavailable
- Queue failed uploads for retry when connection restored

#### Data Integrity
- SHA-256 checksums for upload/download validation
- Database file size verification
- DuckDB connection test after download

#### Conflict Resolution
- Timestamp-based resolution (newer wins)
- Manual conflict resolution via admin API
- Backup conflicted versions with timestamps

### 7. Monitoring & Health Checks

#### Metrics to Track
- Last successful backup timestamp
- Backup/restore operation success rates
- Database file size and growth
- Sync operation latency

#### Health Endpoints
```
GET /api/health/database     # Database connection status
GET /api/health/r2-sync      # R2 sync status and metrics
POST /api/admin/backup       # Manual backup trigger
POST /api/admin/restore      # Manual restore from R2
```

## Implementation Phases

### Phase 1: Core R2 Integration (2-3 days)
- [ ] Install AWS SDK v3 for S3-compatible R2 access
- [ ] Create R2 client wrapper with configuration
- [ ] Implement basic upload/download functionality
- [ ] Add R2 environment variables to config
- [ ] Create R2DatabaseSync service class

### Phase 2: DuckDB Integration (1-2 days)  
- [ ] Integrate R2Sync with DuckDBConnection lifecycle
- [ ] Implement startup database restoration logic
- [ ] Add graceful shutdown with backup upload
- [ ] Error handling and fallback mechanisms
- [ ] Database validation after sync operations

### Phase 3: Advanced Features (2-3 days)
- [ ] Periodic background sync with configurable intervals
- [ ] Database versioning with timestamp suffixes
- [ ] Admin API endpoints for manual operations
- [ ] Health check endpoints with sync status
- [ ] Comprehensive logging and monitoring

### Phase 4: Production Hardening (1-2 days)
- [ ] Security improvements (credential rotation, IAM)
- [ ] Performance optimization (compression, chunking)
- [ ] Comprehensive error handling and recovery
- [ ] Documentation and deployment guides
- [ ] End-to-end testing scenarios

### Phase 5: Deployment & Monitoring (1 day)
- [ ] Update deployment configuration for R2
- [ ] Set up monitoring dashboards
- [ ] Create backup/restore runbooks
- [ ] Performance testing and optimization
- [ ] Documentation for operations team

## Dependencies

### Required Packages
```json
{
  "@aws-sdk/client-s3": "^3.0.0",
  "@aws-sdk/lib-storage": "^3.0.0", 
  "mime-types": "^2.1.35"
}
```

### Cloudflare R2 Setup
1. Create R2 bucket in Cloudflare dashboard
2. Generate API tokens with R2 permissions
3. Configure CORS if needed for admin UI
4. Set up bucket lifecycle policies for retention

## Success Criteria

### Functional Requirements
- ✅ Database persists across server restarts
- ✅ Automatic backup every 30 minutes
- ✅ Fast startup with R2 restore capability
- ✅ Manual backup/restore via API
- ✅ Graceful handling of network failures

### Performance Requirements
- ✅ Startup time increase < 30 seconds
- ✅ Backup operation completes within 5 minutes
- ✅ No impact on query performance during sync
- ✅ 99.9% backup success rate
- ✅ Recovery time objective (RTO) < 10 minutes

### Security Requirements
- ✅ Encrypted data in transit and at rest
- ✅ Secure credential management
- ✅ Access logging and audit trails
- ✅ Data integrity validation
- ✅ Compliance with data retention policies

## Risk Assessment

### High Risk
- **Data Loss**: Mitigated by multiple backup versions and validation
- **Sync Conflicts**: Mitigated by timestamp-based resolution
- **R2 Outages**: Mitigated by local operation fallback

### Medium Risk  
- **Performance Impact**: Mitigated by background sync and optimization
- **Storage Costs**: Mitigated by retention policies and compression
- **Security Vulnerabilities**: Mitigated by IAM and encryption

### Low Risk
- **Implementation Complexity**: Well-documented AWS SDK patterns
- **Operational Overhead**: Automated with manual override options

## Cost Analysis

### Storage Costs (Cloudflare R2)
- **Database Size**: ~100MB (estimated)
- **Daily Backups**: ~3GB/month storage
- **Monthly Cost**: ~$0.015/GB = $0.045/month
- **Operations**: Minimal cost for PUT/GET operations

### Development Costs
- **Implementation**: 8-10 developer days
- **Testing**: 2-3 developer days  
- **Documentation**: 1 developer day
- **Total**: ~2 weeks of development effort

This plan provides a comprehensive roadmap for implementing robust DuckDB persistence using Cloudflare R2 while maintaining performance and reliability.