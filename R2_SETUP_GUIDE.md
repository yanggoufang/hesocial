# Cloudflare R2 Setup Guide

## Issue Identified
The current R2 configuration has placeholder/invalid credentials that need to be replaced with actual Cloudflare R2 values.

## Current Issues Found
1. **R2_ENDPOINT**: Contains placeholder `account-id` instead of actual account ID
2. **R2_ACCESS_KEY_ID**: Only 15 characters (too short, should be ~20+ chars)
3. **R2_SECRET_ACCESS_KEY**: Only 15 characters (too short, should be ~40+ chars)

## How to Get Correct R2 Credentials

### Step 1: Access Cloudflare Dashboard
1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Navigate to **R2 Object Storage**

### Step 2: Get Account ID
1. In the R2 dashboard, note your **Account ID** (visible in the right sidebar)
2. Your endpoint should be: `https://[ACCOUNT-ID].r2.cloudflarestorage.com`

### Step 3: Create API Token
1. Go to **Manage R2 API tokens**
2. Click **Create API token**
3. Choose **R2 Token** (not Cloudflare API token)
4. Set permissions:
   - **Object Read & Write** for your bucket
   - **Account permissions**: Allow
5. Click **Create API token**
6. **IMPORTANT**: Copy both the **Access Key ID** and **Secret Access Key** immediately

### Step 4: Create/Verify Bucket
1. In R2 dashboard, create bucket named `hesocial-duckdb-dev` (for development)
2. For production, create `hesocial-duckdb` bucket

### Step 5: Update Environment Variables
Create or update your `.env` file:

```bash
# Replace [YOUR-ACCOUNT-ID] with actual account ID from step 2
R2_ENDPOINT=https://[YOUR-ACCOUNT-ID].r2.cloudflarestorage.com

# Replace with actual credentials from step 3  
R2_ACCESS_KEY_ID=[YOUR-ACCESS-KEY-ID]
R2_SECRET_ACCESS_KEY=[YOUR-SECRET-ACCESS-KEY]

# Bucket names
R2_BUCKET_NAME=hesocial-duckdb-dev
R2_BACKUP_PATH=backups/

# Optional
R2_REGION=auto
```

### Step 6: Test Connection
After updating the environment variables, restart your application and check the logs for:
- `✅ R2BackupService initialized` (should show correct configuration)
- `✅ R2 connection test successful` (during health checks)

## Expected Credential Formats
- **Access Key ID**: ~20+ characters, alphanumeric
- **Secret Access Key**: ~40+ characters, alphanumeric with special chars
- **Endpoint**: Must contain your actual account ID, not placeholder text

## Troubleshooting
If you still get SSL handshake errors after correct credentials:
1. Verify your network allows HTTPS connections to Cloudflare
2. Check if corporate firewall/proxy is interfering
3. Try testing from a different network environment

## Security Note
- Never commit actual credentials to version control
- Use `.env` files that are in `.gitignore`
- Consider using environment-specific credentials for dev/staging/prod