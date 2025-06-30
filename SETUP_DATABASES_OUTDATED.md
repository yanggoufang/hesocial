# ⚠️ OUTDATED: PostgreSQL/Redis Database Setup Guide

**⚠️ WARNING: This guide is OUTDATED and no longer applies to the current HeSocial platform.**

**The project has migrated to DuckDB-only architecture. This file is kept for historical reference only.**

---

## Current Database Architecture (DuckDB)

The HeSocial platform now uses:
- **DuckDB**: Embedded SQL database (no external database server required)
- **Cloudflare R2**: For database persistence and backups
- **Local Development**: Database file stored as `hesocial.duckdb`

## Getting Started (Current Method)

```bash
# Install all dependencies
npm run setup

# Start development (includes DuckDB setup)
npm run dev

# Run database migrations
npm run migrate

# Load sample data
npm run seed
```

## Database Files

- **Schema**: `database/duckdb-schema.sql`
- **Database File**: `hesocial.duckdb` (auto-created)
- **Seed Data**: `database/seed-*.sql`

## Current Status
✅ **Frontend**: Running at http://localhost:3000  
✅ **Backend**: Running at http://localhost:5000 (DuckDB embedded)
✅ **Database**: DuckDB embedded (no external setup required)

## For Production

See `CLOUDFLARE_R2_PLAN.md` for production persistence setup with Cloudflare R2.

---

## ⚠️ LEGACY CONTENT BELOW (PostgreSQL/Redis - NO LONGER USED)

### Option 1: Docker (OUTDATED - NOT USED)

```bash
# Start PostgreSQL (NOT USED ANYMORE)
docker run --name hesocial-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=hesocial \
  -e POSTGRES_USER=postgres \
  -p 5432:5432 \
  -d postgres:15

# Start Redis (NOT USED ANYMORE)
docker run --name hesocial-redis \
  -p 6379:6379 \
  -d redis:7-alpine
```

### Option 2: Install Locally (OUTDATED - NOT USED)

#### On Ubuntu/Debian:
```bash
# Install PostgreSQL (NOT USED ANYMORE)
sudo apt install postgresql postgresql-contrib

# Install Redis (NOT USED ANYMORE)
sudo apt install redis-server
```

### Legacy Environment Variables (NOT USED)

```
DB_HOST=localhost
DB_PORT=5432
DB_NAME=hesocial
DB_USER=postgres
DB_PASSWORD=password

REDIS_HOST=localhost
REDIS_PORT=6379
```