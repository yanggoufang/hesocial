# ⚠️ OUTDATED: Cloudflare R2 + rclone Setup Guide

**⚠️ WARNING: This rclone approach is OUTDATED.**

**RECOMMENDED APPROACH: Backend integration with graceful shutdown backup**

See `CLOUDFLARE_R2_SIMPLIFIED_PLAN.md` for the current simplified approach.

---

# Cloudflare R2 + rclone Setup Guide for HeSocial (LEGACY)

## Overview
~~This guide sets up Cloudflare R2 with rclone for simple, reliable DuckDB backup and restore. This approach is much simpler than custom AWS SDK implementation and provides proven reliability.~~

**REPLACED BY**: Backend AWS SDK integration with graceful shutdown auto-backup and manual web triggers.

## Why rclone?

### Benefits Over Custom Implementation
- ✅ **Proven reliability** - mature, battle-tested tool
- ✅ **Simple setup** - no custom AWS SDK code needed
- ✅ **Built-in features** - sync, encryption, compression, retry logic
- ✅ **Flexible scheduling** - cron jobs or systemd timers
- ✅ **Easy troubleshooting** - standard tool with good documentation
- ✅ **Less maintenance** - no custom code to debug

### Architecture
```
DuckDB Local File → rclone sync → R2 Backup Bucket
```

## Step 1: Create R2 Backup Bucket

### 1.1 Access Cloudflare Dashboard
1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Navigate to **R2 Object Storage**

### 1.2 Create Main Bucket
1. Click **Create bucket**
2. **Bucket name**: `hesocial-production`
3. **Location**: Auto (or choose your preferred region)
4. Click **Create bucket**

### 1.3 Configure Bucket Settings
1. Go to your bucket → **Settings**
2. **Public access**: Disabled (keep private)
3. **Lifecycle rules**: Configure automatic cleanup for cost management
   - Delete objects older than 90 days (final safety net)
   - Transition old backups to cheaper storage if available
   - Prevents unlimited storage growth from automated backups

## Step 2: Generate R2 API Tokens

### 2.1 Create R2 API Token
1. In Cloudflare Dashboard → **My Profile** → **API Tokens**
2. Click **Create Token**
3. Use **Custom token** template
4. Configure permissions:
   - **Account**: Your account
   - **Zone Resources**: Include All zones
   - **Account Resources**: Include All accounts
   - **Permissions**:
     - `Cloudflare R2:Edit` (for read/write access)

### 2.2 Save Credentials
Save these values for rclone configuration:
- **Account ID**: Found in R2 overview page (right sidebar)
- **Access Key ID**: From your R2 API token
- **Secret Access Key**: From your R2 API token

## Step 3: Install rclone

### 3.1 Ubuntu/Debian
```bash
# Install rclone
sudo apt update
sudo apt install rclone

# Verify installation
rclone --version
```

### 3.2 macOS
```bash
# Install using Homebrew
brew install rclone

# Verify installation
rclone --version
```

### 3.3 Manual Installation
If package managers don't work:
```bash
# Download and install latest version
curl https://rclone.org/install.sh | sudo bash
```

## Step 4: Configure rclone for R2

### 4.1 Run rclone Configuration
```bash
rclone config
```

### 4.2 Configuration Steps
1. Choose **n** for new remote
2. **Name**: `hesocial-r2`
3. **Storage type**: Choose **Amazon S3** (R2 is S3-compatible)
4. **Provider**: Choose **Cloudflare R2**
5. **Access Key ID**: Enter your R2 access key
6. **Secret Access Key**: Enter your R2 secret key
7. **Region**: `auto`
8. **Endpoint**: `https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com`
9. **Location constraint**: Leave blank
10. **ACL**: `private`
11. **Storage class**: Leave blank
12. **Advanced config**: **No**
13. **Test connection**: **Yes**
14. **Save**: **Yes**

### 4.3 Test Configuration
```bash
# List buckets
rclone lsd hesocial-r2:

# Should show your hesocial-backups bucket
```

## Step 5: Create Backup Scripts

### 5.1 Create Backup Script
Create `/home/yanggf/a/hesocial/scripts/backup-database.sh`:

```bash
#!/bin/bash

# DuckDB Backup Script for HeSocial
# Backs up DuckDB database to Cloudflare R2 using rclone

set -e

# Configuration
DB_PATH="/home/yanggf/a/hesocial/hesocial.duckdb"
BACKUP_NAME="hesocial-$(date +%Y%m%d-%H%M%S).duckdb"
R2_REMOTE="hesocial-r2:hesocial-backups"
LOG_FILE="/home/yanggf/a/hesocial/logs/backup.log"

# Create logs directory if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

log "Starting DuckDB backup..."

# Check if database file exists
if [ ! -f "$DB_PATH" ]; then
    log "ERROR: Database file not found: $DB_PATH"
    exit 1
fi

# Get database file size
DB_SIZE=$(stat -f%z "$DB_PATH" 2>/dev/null || stat -c%s "$DB_PATH" 2>/dev/null)
log "Database size: $DB_SIZE bytes"

# Backup database to R2
log "Uploading $DB_PATH as $BACKUP_NAME..."
if rclone copy "$DB_PATH" "$R2_REMOTE/" --s3-upload-cutoff 0 --s3-chunk-size 64M --progress --log-level INFO; then
    log "✅ Successfully uploaded database to R2"
    
    # Rename to timestamped version
    if rclone moveto "$R2_REMOTE/hesocial.duckdb" "$R2_REMOTE/$BACKUP_NAME"; then
        log "✅ Successfully renamed to $BACKUP_NAME"
    else
        log "⚠️  Upload succeeded but rename failed"
    fi
else
    log "❌ Failed to upload database to R2"
    exit 1
fi

# Clean up old backups (keep last 7 days)
log "Cleaning up old backups..."
rclone delete "$R2_REMOTE/" --min-age 7d --dry-run --log-level INFO | tee -a "$LOG_FILE"
# Remove --dry-run to actually delete files

log "Backup completed successfully"
```

### 5.2 Create Restore Script
Create `/home/yanggf/a/hesocial/scripts/restore-database.sh`:

```bash
#!/bin/bash

# DuckDB Restore Script for HeSocial
# Restores DuckDB database from Cloudflare R2 using rclone

set -e

# Configuration
DB_PATH="/home/yanggf/a/hesocial/hesocial.duckdb"
R2_REMOTE="hesocial-r2:hesocial-backups"
LOG_FILE="/home/yanggf/a/hesocial/logs/restore.log"

# Create logs directory if it doesn't exist
mkdir -p "$(dirname "$LOG_FILE")"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

log "Starting DuckDB restore..."

# List available backups
log "Available backups:"
rclone ls "$R2_REMOTE/" | grep "\.duckdb$" | sort -r | head -10 | tee -a "$LOG_FILE"

# Get latest backup
LATEST_BACKUP=$(rclone ls "$R2_REMOTE/" | grep "\.duckdb$" | sort -r | head -1 | awk '{print $2}')

if [ -z "$LATEST_BACKUP" ]; then
    log "❌ No backup files found in R2"
    exit 1
fi

log "Latest backup: $LATEST_BACKUP"

# Backup current database if it exists
if [ -f "$DB_PATH" ]; then
    BACKUP_CURRENT="$DB_PATH.backup-$(date +%Y%m%d-%H%M%S)"
    log "Backing up current database to: $BACKUP_CURRENT"
    cp "$DB_PATH" "$BACKUP_CURRENT"
fi

# Download latest backup
log "Downloading $LATEST_BACKUP..."
if rclone copy "$R2_REMOTE/$LATEST_BACKUP" "$(dirname "$DB_PATH")/" --progress --log-level INFO; then
    # Rename downloaded file to expected name
    if [ "$LATEST_BACKUP" != "hesocial.duckdb" ]; then
        mv "$(dirname "$DB_PATH")/$LATEST_BACKUP" "$DB_PATH"
    fi
    log "✅ Successfully restored database from R2"
else
    log "❌ Failed to download database from R2"
    exit 1
fi

# Verify database integrity
log "Verifying database integrity..."
if sqlite3 "$DB_PATH" "PRAGMA integrity_check;" > /dev/null 2>&1; then
    log "✅ Database integrity check passed"
else
    log "⚠️  Database integrity check failed or not supported"
fi

log "Restore completed successfully"
```

### 5.3 Make Scripts Executable
```bash
chmod +x /home/yanggf/a/hesocial/scripts/backup-database.sh
chmod +x /home/yanggf/a/hesocial/scripts/restore-database.sh
```

## Step 6: Set Up Periodic Backups

### 6.1 Create Cron Job
```bash
# Edit crontab
crontab -e

# Add backup every 30 minutes
*/30 * * * * /home/yanggf/a/hesocial/scripts/backup-database.sh

# Add daily cleanup at 2 AM
0 2 * * * rclone delete hesocial-r2:hesocial-backups/ --min-age 7d --log-level INFO
```

### 6.2 Alternative: Systemd Timer (Recommended for servers)
Create `/etc/systemd/system/hesocial-backup.service`:
```ini
[Unit]
Description=HeSocial DuckDB Backup to R2
After=network.target

[Service]
Type=oneshot
User=yanggf
ExecStart=/home/yanggf/a/hesocial/scripts/backup-database.sh
```

Create `/etc/systemd/system/hesocial-backup.timer`:
```ini
[Unit]
Description=Run HeSocial backup every 30 minutes
Requires=hesocial-backup.service

[Timer]
OnCalendar=*:0/30
Persistent=true

[Install]
WantedBy=timers.target
```

Enable the timer:
```bash
sudo systemctl daemon-reload
sudo systemctl enable hesocial-backup.timer
sudo systemctl start hesocial-backup.timer
```

## Step 7: Environment Configuration

### 7.1 Environment Variables (Optional)
Add to `backend/.env` for application awareness:
```bash
# Backup configuration
BACKUP_ENABLED=true
BACKUP_INTERVAL_MINUTES=30
BACKUP_RETENTION_DAYS=7
R2_BACKUP_BUCKET=hesocial-backups
```

## Step 8: Testing

### 8.1 Test Backup
```bash
# Run backup manually
./scripts/backup-database.sh

# Check logs
tail -f logs/backup.log

# Verify in R2
rclone ls hesocial-r2:hesocial-backups/
```

### 8.2 Test Restore
```bash
# Run restore manually (BE CAREFUL - this overwrites your database)
./scripts/restore-database.sh

# Check logs
tail -f logs/restore.log
```

## Step 9: Monitoring

### 9.1 Log Monitoring
```bash
# Monitor backup logs
tail -f /home/yanggf/a/hesocial/logs/backup.log

# Check for errors
grep "ERROR\|❌" /home/yanggf/a/hesocial/logs/backup.log
```

### 9.2 Cron Job Monitoring
```bash
# Check cron job status
crontab -l

# Check system logs for cron
sudo journalctl -u cron -f
```

## Step 10: Troubleshooting

### Common Issues

**rclone command not found**
```bash
# Install rclone
sudo apt install rclone

# Or use manual installation
curl https://rclone.org/install.sh | sudo bash
```

**Permission denied errors**
```bash
# Fix script permissions
chmod +x scripts/*.sh

# Check file ownership
ls -la scripts/
```

**R2 connection errors**
```bash
# Test rclone configuration
rclone config show hesocial-r2

# Test connection
rclone lsd hesocial-r2:
```

**Database file locked**
```bash
# Make sure backend is stopped before backup
sudo systemctl stop hesocial-backend  # if using systemd
# or kill the Node.js process
```

## Cost Optimization

### Expected Costs (Cloudflare R2)
- **Storage**: ~$0.015/GB/month
- **Database backups**: ~100MB × 48 backups/day = ~4.8GB/month
- **Monthly cost**: ~$0.072/month
- **Operations**: Minimal cost for uploads/downloads

### Cost Management
- Automatic cleanup after 7 days
- Compress backups if needed: `rclone copy --compress`
- Monitor storage usage: `rclone size hesocial-r2:hesocial-backups/`

## Security Best Practices

1. **Credentials**: Never commit R2 credentials to version control
2. **Encryption**: Enable encryption at rest in R2
3. **Access Control**: Use minimal permissions for R2 tokens
4. **Monitoring**: Set up alerts for backup failures
5. **Testing**: Regularly test restore procedures

## Success Criteria

- ✅ Automatic backups every 30 minutes
- ✅ Successful restore within 5 minutes
- ✅ 7-day retention with automatic cleanup
- ✅ Comprehensive logging and monitoring
- ✅ Cost under $1/month for storage

This rclone approach provides a robust, simple, and cost-effective backup solution for HeSocial's DuckDB database.