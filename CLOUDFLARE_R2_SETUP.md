# Cloudflare R2 Setup Guide for HeSocial

## Overview
This guide walks through setting up Cloudflare R2 for DuckDB persistence. You need to configure R2 **before** implementing the sync code.

## When to Configure R2

**Configure Cloudflare R2 FIRST** - before implementing the sync code. This is required for:
- Getting the necessary credentials
- Testing connectivity
- Setting up the bucket structure

## Step 1: Create Cloudflare R2 Bucket

### 1.1 Access Cloudflare Dashboard
1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com)
2. Select your account
3. Navigate to **R2 Object Storage** in the sidebar

### 1.2 Create Bucket
1. Click **Create bucket**
2. **Bucket name**: `hesocial-duckdb`
3. **Location**: Auto (or choose your preferred region)
4. Click **Create bucket**

### 1.3 Configure Bucket Settings
1. Go to your bucket → **Settings**
2. **Public access**: Disabled (keep private)
3. **Lifecycle rules**: Configure automatic cleanup (R2 doesn't have native versioning)
   - Delete files older than 30 days (optional)
   - Keep recent backups for disaster recovery

## Step 2: Generate API Tokens

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

### 2.2 Alternative: Account ID and Global API Key
If you prefer using account credentials:
1. **Account ID**: Found in R2 overview page (right sidebar)
2. **Global API Key**: My Profile → API Tokens → Global API Key

## Step 3: Environment Configuration

### 3.1 Backend Environment Variables
Add to your `backend/.env` file:

```bash
# Cloudflare R2 Configuration
R2_ACCOUNT_ID=your-account-id-here
R2_ACCESS_KEY_ID=your-access-key-here
R2_SECRET_ACCESS_KEY=your-secret-key-here
R2_BUCKET_NAME=hesocial-duckdb
R2_ENDPOINT=https://your-account-id.r2.cloudflarestorage.com
R2_REGION=auto

# Sync Configuration
R2_SYNC_ENABLED=true
R2_BACKUP_INTERVAL_MINUTES=30
R2_DATABASE_KEY=hesocial-production.duckdb
R2_BACKUP_RETENTION_DAYS=30
R2_COMPRESS_UPLOADS=true
R2_VALIDATE_DOWNLOADS=true
```

### 3.2 Replace Placeholder Values

Replace these values with your actual Cloudflare credentials:

- `your-account-id-here`: From Cloudflare Dashboard → R2 → Account ID
- `your-access-key-here`: From your R2 API token or Access Key
- `your-secret-key-here`: From your R2 API token or Secret Key

## Step 4: Test Connectivity (Optional)

You can test R2 connectivity using AWS CLI (since R2 is S3-compatible):

### 4.1 Install AWS CLI
```bash
# Ubuntu/Debian
sudo apt install awscli

# macOS
brew install awscli
```

### 4.2 Configure AWS CLI for R2
```bash
aws configure set aws_access_key_id YOUR_R2_ACCESS_KEY
aws configure set aws_secret_access_key YOUR_R2_SECRET_KEY
aws configure set region auto
```

### 4.3 Test Connection
```bash
# List buckets
aws s3 ls --endpoint-url https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com

# Test upload/download
echo "test" > test.txt
aws s3 cp test.txt s3://hesocial-duckdb/test.txt --endpoint-url https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
aws s3 ls s3://hesocial-duckdb/ --endpoint-url https://YOUR_ACCOUNT_ID.r2.cloudflarestorage.com
rm test.txt
```

## Step 5: Security Best Practices

### 5.1 Credential Management
- **Never commit credentials** to version control
- Use environment variables for all sensitive data
- Consider using Cloudflare Workers for additional security layer

### 5.2 Access Control
- Create specific API tokens with minimal required permissions
- Regularly rotate API tokens (every 90 days recommended)
- Monitor API usage in Cloudflare Dashboard

### 5.3 Bucket Security
- Keep bucket private (no public access)
- Use timestamp-based file naming for backup history (R2 doesn't have native versioning)
- Set up lifecycle rules to manage storage costs and retention

## Step 6: Cost Optimization

### 6.1 Expected Costs
- **Storage**: ~$0.015/GB/month
- **Operations**: Minimal (Class A: $4.50/million, Class B: $0.36/million)
- **Data transfer**: Free egress to internet

### 6.2 Cost Management
- Enable versioning with lifecycle rules
- Set up alerts for unusual usage
- Monitor storage growth in dashboard

## Step 7: Verification Checklist

Before implementing sync code, verify:

- [ ] R2 bucket created and accessible
- [ ] API tokens generated with correct permissions
- [ ] Environment variables configured in `backend/.env`
- [ ] Connectivity tested (optional but recommended)
- [ ] Security settings configured

## Troubleshooting

### Common Issues

**Access Denied Errors**
- Check API token permissions
- Verify Account ID matches your account
- Ensure bucket name is correct

**Connection Timeouts**
- Check internet connectivity
- Verify endpoint URL format
- Test with AWS CLI first

**Permission Errors**
- Ensure R2:Edit permission is granted
- Check if token has expired
- Verify account-level access

## Next Steps

Once R2 is configured:
1. Install AWS SDK v3 in backend
2. Implement R2 client wrapper
3. Create upload/download functionality
4. Integrate with DuckDB lifecycle

## Support Resources

- [Cloudflare R2 Documentation](https://developers.cloudflare.com/r2/)
- [R2 API Reference](https://developers.cloudflare.com/r2/api/)
- [AWS SDK v3 Documentation](https://docs.aws.amazon.com/AWSJavaScriptSDK/v3/latest/)