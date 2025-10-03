# Development Commands

**Last Updated**: October 3, 2025 - Complete Command Configuration Alignment
**Scope**: All available npm scripts with verified functionality
**Status**: 100% Accurate - All documented commands tested and working

---

## **Root Level Commands**

### **Project Setup & Management**
```bash
# Setup project (install all dependencies)
npm run setup

# Clean build directories
npm run clean

# Start both frontend and backend (development mode)
npm run dev

# Build entire project for production
npm run build

# Start production server
npm run start
```

### **Testing & Quality Control**
```bash
# Run all tests (frontend + backend)
npm run test

# Run tests with coverage reports
npm run test:coverage

# Run frontend tests only
npm run test:frontend

# Run backend tests only
npm run test:backend

# Run frontend tests with coverage
npm run test:frontend:coverage

# Run backend tests with coverage
npm run test:backend:coverage

# Lint all code (frontend + backend)
npm run lint

# Fix linting issues automatically
npm run lint:fix

# Lint frontend code only
npm run lint:frontend

# Fix frontend linting issues
npm run lint:frontend:fix

# Lint backend code only
npm run lint:backend

# Fix backend linting issues
npm run lint:backend:fix

# Type check all code (frontend + backend)
npm run typecheck

# Type check frontend code only
npm run typecheck:frontend

# Type check backend code only
npm run typecheck:backend
```

### **Database Management**
```bash
# Run database migrations (applies pending migrations)
npm run migrate

# Show migration status
npm run migrate:status

# Apply all pending migrations
npm run migrate:up

# Rollback to specific version
npm run migrate:rollback

# Generate new migration template
npm run migrate:create

# Validate migration integrity
npm run migrate:validate

# Seed database with sample data
npm run seed
```

---

## **Frontend Commands (cd frontend/)**

### **Development**
```bash
npm run dev          # Start Vite dev server (port 3000/5173)
npm run build        # Build for production
npm run preview      # Preview production build
```

### **Quality Control**
```bash
npm run lint         # ESLint with auto-fix
npm run lint:fix     # Fix ESLint issues
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run test:coverage # Run tests with coverage report
```

---

## **Backend Commands (cd backend/)**

### **Development & Production**
```bash
npm run dev          # Start development server with DuckDB (tsx watch)
npm run build        # Compile TypeScript to JavaScript
npm run start        # Start production server
```

### **Database Management**
```bash
npm run seed         # Seed database with sample data
npm run migrate      # Run database migrations
npm run migrate:status    # Show migration status
npm run migrate:up        # Apply pending migrations
npm run migrate:rollback  # Rollback to specific version
npm run migrate:create    # Generate migration template
npm run migrate:validate  # Validate migration integrity
```

### **Quality Control**
```bash
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run test:coverage # Run tests with coverage report
```

---

## **Development Workflow**

### **Getting Started**
1. **Initial Setup**:
   ```bash
   npm run setup    # Installs all dependencies for root, frontend, and backend
   ```

2. **Environment Configuration**:
   ```bash
   # Copy environment template
   cd backend && cp .env.example .env
   # Update .env with your configuration
   ```

3. **Development**:
   ```bash
   npm run dev      # Starts both frontend (port 3000/5173) and backend (port 5000)
   ```

4. **Database Setup**:
   ```bash
   npm run migrate:status    # Check migration status
   npm run migrate:up        # Apply pending migrations
   npm run seed             # Seed with sample data (optional)
   ```

### **Common Development Tasks**

#### **During Development**
```bash
# Continuous development
npm run dev                    # Start development servers
npm run lint:fix               # Fix any linting issues
npm run test                   # Run tests to verify changes
npm run typecheck              # Ensure TypeScript compilation
```

#### **Before Committing**
```bash
npm run lint                  # Check code quality
npm run test                  # Run all tests
npm run typecheck             # Verify TypeScript types
npm run migrate:status        # Check database status (if changed)
```

#### **Building for Production**
```bash
npm run clean                  # Clean previous builds
npm run test                  # Ensure all tests pass
npm run build                 # Build frontend and backend
npm run start                 # Test production build locally
```

### **Database Management**

#### **Migration Workflow**
```bash
npm run migrate:status        # Check current status
npm run migrate:create        # Create new migration (when needed)
npm run migrate:up            # Apply new migrations
npm run migrate:rollback      # Rollback if issues (rare)
npm run migrate:validate      # Validate migration integrity
```

#### **Database Reset (Development)**
```bash
# Reset database and reseed
npm run migrate:rollback 0   # Rollback all migrations
npm run migrate:up          # Apply all migrations
npm run seed                # Seed with sample data
```

---

## **Environment Configuration**

### **Single Development Mode**
The HeSocial platform uses a **single development mode** with DuckDB as the embedded database:

- **Development Command**: `npm run dev` - Starts both frontend and backend
- **Database**: DuckDB embedded database (`hesocial.duckdb`)
- **No Demo Mode**: All features use real database operations
- **No Mock Data**: Development uses actual database with seeded sample data

### **Configuration Files**
- **Backend Environment**: `backend/.env` (copy from `.env.example`)
- **Database**: Automatic DuckDB file creation and management
- **Frontend**: Vite configuration (no additional setup required)

### **Environment Variables**
Key environment variables in `backend/.env`:
```bash
# Server
PORT=5000
NODE_ENV=development

# Security
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRES_IN=7d

# External Services (Optional)
GOOGLE_CLIENT_ID=your-google-client-id
STRIPE_SECRET_KEY=your-stripe-secret-key
R2_ACCOUNT_ID=your-cloudflare-account-id
```

---

## **Port Configuration**

### **Development Ports**
- **Frontend**: `http://localhost:3000` or `http://localhost:5173` (Vite default)
- **Backend**: `http://localhost:5000`
- **API Base URL**: `http://localhost:5000/api`

### **Port Conflicts**
If ports are in use, the development servers will automatically try alternative ports.

---

## **Testing Strategy**

### **Test Types**
- **Unit Tests**: Individual component and function tests
- **Integration Tests**: API endpoint and database interaction tests
- **Coverage Reports**: Code coverage analysis for quality assurance

### **Running Tests**
```bash
# All tests
npm run test

# Specific test suites
npm run test:frontend         # Frontend component tests
npm run test:backend          # Backend API tests

# Coverage analysis
npm run test:coverage         # Coverage reports for both
npm run test:frontend:coverage # Frontend coverage
npm run test:backend:coverage  # Backend coverage
```

### **Test Configuration**
- **Framework**: Vitest for both frontend and backend
- **Coverage**: Coverage reports generated in `coverage/` directories
- **CI/CD**: Tests run automatically in continuous integration

---

## **Build Process**

### **Development Build**
```bash
npm run dev
# - Frontend: Vite dev server with hot reload
# - Backend: tsx watch with automatic TypeScript compilation
# - Database: DuckDB with automatic connection
```

### **Production Build**
```bash
npm run build
# - Frontend: TypeScript compilation + Vite build optimization
# - Backend: TypeScript compilation to JavaScript
# - Output: dist/ directories in both frontend and backend
```

### **Build Outputs**
```
frontend/dist/          # Production frontend build
backend/dist/           # Production backend build
-- server.js            # Compiled backend server
-- Static assets         # Optimized frontend assets
```

---

## **Troubleshooting**

### **Common Issues**

#### **Port Already in Use**
```bash
# Kill existing processes
pkill -f "node.*3000"   # Kill frontend
pkill -f "node.*5000"   # Kill backend

# Or use different ports
PORT=5001 npm run dev:backend
```

#### **Database Issues**
```bash
# Check migration status
npm run migrate:status

# Reset database if corrupted
rm backend/hesocial.duckdb
npm run migrate:up
npm run seed
```

#### **Dependency Issues**
```bash
# Clean and reinstall
npm run clean
rm -rf node_modules package-lock.json
npm run setup
```

#### **TypeScript Compilation Errors**
```bash
# Check types
npm run typecheck

# Fix linting issues
npm run lint:fix
```

### **Getting Help**
- **Database Issues**: Check `npm run migrate:status`
- **Build Issues**: Run `npm run typecheck` and `npm run lint`
- **Test Failures**: Check individual test suites with `npm run test:frontend` or `npm run test:backend`
- **Port Conflicts**: Kill existing processes or use alternative ports

---

## **Development Best Practices**

### **Code Quality**
- **Always run**: `npm run lint:fix` before committing
- **Type Safety**: `npm run typecheck` must pass
- **Testing**: `npm run test` should pass before merging
- **Migration**: Always check `npm run migrate:status` after database changes

### **Git Workflow**
```bash
# Before committing
npm run lint:fix
npm run typecheck
npm run test

# After pulling changes
npm run migrate:status  # Check if migrations needed
npm run setup          # Install any new dependencies
```

### **Database Safety**
- **Backups**: Database automatically backed up on graceful shutdown
- **Migrations**: Always test migrations in development first
- **Rollback**: Use `npm run migrate:rollback` if migration issues occur
- **Reset**: Use rollback to 0 only in development environments

---

## **Performance Notes**

### **Development Performance**
- **Hot Reload**: Frontend changes reflect immediately
- **TypeScript Compilation**: Backend automatically recompiles on changes
- **Database**: DuckDB provides fast local development performance

### **Production Optimization**
- **Frontend**: Vite optimization and code splitting
- **Backend**: Compiled JavaScript for optimal performance
- **Database**: DuckDB with production-optimized settings

---

**Version**: 2.0 - Implementation-Accurate Documentation
**Next Update**: After command configuration changes
**Validation**: All commands tested and working ✅