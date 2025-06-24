# Database Setup Guide for HeSocial Platform

## Current Status
✅ **Frontend**: Running at http://localhost:3000  
✅ **Backend**: Running at http://localhost:5000 (Demo Mode with mock data)

## Database Setup Options

### Option 1: Docker (Recommended)

If you have Docker running, use these commands:

```bash
# Start PostgreSQL
docker run --name hesocial-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=hesocial \
  -e POSTGRES_USER=postgres \
  -p 5432:5432 \
  -d postgres:15

# Start Redis
docker run --name hesocial-redis \
  -p 6379:6379 \
  -d redis:7-alpine

# Wait for containers to start (30 seconds)
sleep 30

# Run database migrations
cd /home/yanggf/a/hesocial/backend
npm run migrate

# Load sample data (optional)
npm run seed

# Start full backend (stop demo mode first)
npm run dev
```

### Option 2: Install Locally

#### On Ubuntu/Debian:
```bash
# Update package list
sudo apt update

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib

# Install Redis
sudo apt install redis-server

# Start services
sudo systemctl start postgresql
sudo systemctl start redis

# Create database user and database
sudo -u postgres createuser --interactive
sudo -u postgres createdb hesocial
```

#### On macOS:
```bash
# Install using Homebrew
brew install postgresql redis

# Start services
brew services start postgresql
brew services start redis

# Create database
createdb hesocial
```

### Option 3: Cloud Databases

For production or if local setup is complex:

1. **PostgreSQL**: Use services like:
   - AWS RDS
   - Google Cloud SQL
   - Heroku Postgres
   - Supabase

2. **Redis**: Use services like:
   - AWS ElastiCache
   - Redis Cloud
   - Heroku Redis

Update `.env` file with cloud database URLs.

## After Database Setup

1. **Stop demo backend**:
   ```bash
   # Press Ctrl+C in the terminal running demo backend
   ```

2. **Run migrations**:
   ```bash
   cd /home/yanggf/a/hesocial/backend
   npm run migrate
   ```

3. **Load sample data** (optional):
   ```bash
   npm run seed
   ```

4. **Start full backend**:
   ```bash
   npm run dev
   ```

## Verification

Once databases are running, verify with:

```bash
# Check PostgreSQL connection
psql -h localhost -U postgres -d hesocial -c "SELECT version();"

# Check Redis connection
redis-cli ping

# Test backend API
curl http://localhost:5000/api/health
```

## Current Demo Endpoints

While in demo mode, you can test these endpoints:

- **Health Check**: http://localhost:5000/api/health
- **Mock Events**: http://localhost:5000/api/events
- **Categories**: http://localhost:5000/api/events/categories

## Environment Variables

The backend uses these database settings (in `.env`):

```
DB_HOST=localhost
DB_PORT=5432
DB_NAME=hesocial
DB_USER=postgres
DB_PASSWORD=password

REDIS_HOST=localhost
REDIS_PORT=6379
```

Modify these if using different credentials or hosts.

## Troubleshooting

1. **Connection refused**: Ensure databases are running
2. **Permission denied**: Check user credentials
3. **Port conflicts**: Change ports in `.env` if needed
4. **Docker issues**: Ensure Docker daemon is running

## Security Note

The current `.env` uses default passwords. For production:
- Use strong, unique passwords
- Enable SSL/TLS connections
- Configure firewall rules
- Use environment-specific configurations