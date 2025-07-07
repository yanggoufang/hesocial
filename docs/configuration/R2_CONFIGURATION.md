# R2 Storage Configuration

For production media storage using Cloudflare R2:

## Setup Status
**⚠️ SETUP REQUIRED**: Current R2 credentials are placeholders and need to be replaced with actual Cloudflare R2 values. See `R2_SETUP_GUIDE.md` for detailed setup instructions.

## Environment Variables
```bash
# Required environment variables for R2 integration
# Replace [YOUR-ACCOUNT-ID] with actual Cloudflare account ID
R2_ENDPOINT=https://[YOUR-ACCOUNT-ID].r2.cloudflarestorage.com
R2_ACCESS_KEY_ID=[YOUR-ACCESS-KEY-ID]  # ~20+ chars from Cloudflare R2 dashboard
R2_SECRET_ACCESS_KEY=[YOUR-SECRET-ACCESS-KEY]  # ~40+ chars from Cloudflare R2 dashboard
R2_BUCKET_NAME=hesocial-duckdb-dev  # or hesocial-duckdb for production
R2_BACKUP_PATH=backups/

# Optional
R2_REGION=auto
R2_PUBLIC_URL=https://media.hesocial.com
```

## Features
- Automatic image optimization and thumbnail generation
- Secure document storage with signed URLs
- CDN delivery for public images
- File validation and size limits (10MB)
- Render.com compatible (no ephemeral file system dependency)

## Status
R2 backup service will automatically disable if credentials are invalid or missing.

## Temporary Development Server
For development and testing when the main DuckDB server has connection issues:

```bash
# Start temporary server with mock authentication
node backend/temp-server.cjs

# Available endpoints:
# POST /api/auth/login - Mock authentication
# GET /api/auth/profile - Mock profile retrieval
# GET /api/health - Health check

# Test accounts:
# Admin: admin@hesocial.com / admin123
# User: test@example.com / test123
```

## Related Documentation
- [R2 Setup Guide](../setup/CLOUDFLARE_R2_SETUP.md)
- [R2 Backup Implementation](../R2_BACKUP_IMPLEMENTATION.md)
- [System Health Dashboard](../systems/SYSTEM_HEALTH_DASHBOARD.md)