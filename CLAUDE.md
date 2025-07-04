# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-end social event platform targeting affluent individuals aged 45-65 with high net worth (NT$5M+ annual income or NT$30M+ net assets). The platform facilitates luxury social events like private dinners, yacht parties, and art appreciation gatherings.

## Current Technology Stack

### Frontend (React + Vite)
- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite with SWC plugin for fast builds
- **Styling**: Tailwind CSS with luxury color palette
- **Key Libraries**: 
  - React Router DOM for navigation
  - Axios for HTTP requests
  - React Player for media
  - Three.js for 3D/AR features
  - Framer Motion for animations
  - React Hook Form + Zod for form handling
  - Lucide React for icons

### Backend (Node.js + Express)
- **Runtime**: Node.js with Express framework
- **Language**: TypeScript with ESM modules
- **Database**: DuckDB as primary database with Cloudflare R2 persistence
- **Migration System**: Comprehensive database migration framework with rollback capabilities
- **Authentication**: JWT with Passport.js (Google OAuth, LinkedIn OAuth)
- **Security**: Helmet, CORS, rate limiting, compression
- **Logging**: Winston with Morgan middleware
- **File Processing**: Multer with Sharp for image processing
- **Payments**: Stripe integration
- **Health Monitoring**: Startup health checks and operational metrics

### Database Options
- **Primary**: DuckDB with schema defined in `database/duckdb-schema.sql`
- **Migration System**: (COMPLETED - Production Ready)
  - Comprehensive migration framework with version control
  - Rollback capabilities with safety checks
  - CLI tools for migration management
  - Automatic migration validation and integrity checks
  - Smart restore logic with timestamp comparison
- **Production Persistence**: Cloudflare R2 backup system (COMPLETED - Production Ready)
  - Graceful shutdown auto-backup
  - Manual backup API endpoints (`POST /api/admin/backup`, `GET /api/admin/backups`)
  - Environment separation (dev: `hesocial-duckdb-dev`, prod: `hesocial-duckdb`)
- **Development**: Local DuckDB file for development
- **Key Tables**: Users, Events, Registrations, Financial verification, Server state tracking

## Development Commands

### Root Level Commands
```bash
# Setup project (install all dependencies)
npm run setup

# Start both frontend and backend
npm run dev

# Build entire project
npm run build

# Run all tests
npm run test

# Lint all code
npm run lint

# Type check all code
npm run typecheck

# Database operations
npm run migrate      # Run database migrations (help)
npm run seed         # Seed development data

# Migration management
npm run migrate:status    # Show migration status
npm run migrate:up        # Apply pending migrations
npm run migrate:rollback  # Rollback to version
npm run migrate:create    # Generate migration template
npm run migrate:validate  # Validate migration integrity
```

### Frontend Commands (cd frontend/)
```bash
npm run dev          # Start Vite dev server
npm run build        # Build for production
npm run preview      # Preview production build
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run test:coverage # Run tests with coverage
```

### Backend Commands (cd backend/)
```bash
npm run dev          # Start with DuckDB (tsx watch)
npm run dev:demo     # Start demo server with mock data
npm run dev:duckdb   # Start with DuckDB embedded database
npm run build        # Compile TypeScript
npm run start        # Start production server
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run seed         # Seed database with sample data

# Migration management (same as root commands)
npm run migrate:status    # Show migration status
npm run migrate:up        # Apply pending migrations
npm run migrate:rollback  # Rollback to version
npm run migrate:create    # Generate migration template
npm run migrate:validate  # Validate migration integrity
```

## Project Architecture

### Monorepo Structure
```
hesocial/
├── frontend/          # React + Vite frontend application
│   ├── src/
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/         # Route-based page components
│   │   ├── types/         # TypeScript type definitions
│   │   └── styles/        # Global CSS and Tailwind config
│   ├── package.json
│   └── vite.config.ts
├── backend/           # Express + TypeScript backend API
│   ├── src/
│   │   ├── controllers/   # Route handlers
│   │   ├── database/      # DB connections and migrations
│   │   │   └── migrations/    # Migration framework and files
│   │   ├── middleware/    # Express middleware
│   │   ├── routes/        # API route definitions
│   │   ├── services/      # Business logic and system services
│   │   ├── config/        # Configuration management
│   │   ├── types/         # Shared TypeScript types
│   │   └── utils/         # Configuration and logging
│   ├── server.ts          # Main production server with enhanced startup
│   ├── server-duckdb.ts   # DuckDB embedded server
│   └── server-demo.ts     # Demo server with mock data
├── database/          # SQL schemas and seed data
│   ├── schema.sql         # PostgreSQL schema
│   ├── duckdb-schema.sql  # DuckDB schema
│   └── seed files
└── package.json       # Root workspace configuration
```

### Backend Server Variants
The backend supports three different server configurations:
1. **Production Server** (`server.ts`): DuckDB with Cloudflare R2 persistence, enhanced startup sequence
2. **DuckDB Server** (`server-duckdb.ts`): Local DuckDB file for development
3. **Demo Server** (`server-demo.ts`): Mock data for quick prototyping

### Enhanced Server Features
- **9-Step Startup Sequence**: Configuration validation → Service initialization → Smart restore → Database connection → Migration checks → Health validation → Route loading → Server start
- **Health Monitoring**: Comprehensive startup health checks with visual indicators
- **Server State Tracking**: Operational metrics including uptime, session duration, and startup counts
- **Smart Backup/Restore**: Automatic timestamp-based restoration with data loss prevention
- **Graceful Shutdown**: Clean shutdown with automatic backup creation and state recording

### Database Architecture
- **User Management**: Multi-tier membership system (Platinum, Diamond, Black Card)
- **Event System**: Premium events with pricing, exclusivity levels, and venue management
- **Registration System**: Event registration with approval workflows
- **Financial Verification**: Required for platform access
- **Migration System**: Version-controlled schema changes with rollback capabilities
- **Operational Tracking**: Server state, migration history, and audit trails

### Authentication & Security
- JWT-based authentication with refresh tokens
- OAuth 2.0 integration (Google, LinkedIn)
- Rate limiting and request validation
- Helmet.js for security headers
- CORS configuration for cross-origin requests

## Development Workflow

### Getting Started
1. **Initial Setup**: `npm run setup` - installs all dependencies
2. **Development**: `npm run dev` - starts both frontend (port 3000) and backend (port 5000)
3. **Database Setup**: See docs/setup/ for database configuration options

### Testing Commands
- `npm run test` - run all tests (frontend + backend)
- `npm run test:frontend` - frontend tests only
- `npm run test:backend` - backend tests only
- Individual test suites use Vitest

### Code Quality
- `npm run lint` - ESLint for both frontend and backend
- `npm run typecheck` - TypeScript compilation check
- Strict TypeScript configuration with no implicit any

### Database Development Modes
- **Demo Mode**: `npm run dev:demo` - uses mock data, no database required
- **DuckDB Mode**: `npm run dev:duckdb` - local DuckDB file for development
- **Full Mode**: `npm run dev` - DuckDB with Cloudflare R2 for production

## Key Implementation Details

### Frontend Architecture
- React 18 with functional components and hooks
- Vite for fast development and building
- Tailwind CSS for styling with luxury color palette
- TypeScript for type safety
- React Router for client-side routing

### Backend Architecture
- Express.js with TypeScript and ESM modules
- Modular route structure with controllers
- Database abstraction layer supporting multiple databases
- Comprehensive logging with Winston
- File upload handling with Multer and Sharp

### API Design
- RESTful API endpoints under `/api` prefix
- Health check endpoints for monitoring
- Structured error responses with success/error flags
- Request/response logging for debugging
- **Admin backup endpoints** (Production Ready):
  - `POST /api/admin/backup` - Create manual backup
  - `GET /api/admin/backups` - List available backups
  - `PUT /api/admin/backup/:id/status` - Update backup status
  - `DELETE /api/admin/backups/cleanup` - Cleanup old backups
- **Enhanced health endpoints**:
  - `GET /api/health` - Basic health check
  - `GET /api/health/detailed` - Comprehensive system status with migration info, server stats, and uptime

## Database Migration System

### Migration Framework (Production Ready)
A comprehensive database migration system that provides safe schema changes and deployments:

#### Core Features
- **Version Control**: Each migration has unique ID and sequential versioning
- **Dependency Tracking**: Migrations can specify dependencies on other migrations
- **Rollback Capabilities**: Safe reversible operations with down() methods
- **Transaction Safety**: Atomic execution with comprehensive error handling
- **Integrity Validation**: Checksums and validation to detect modified migrations
- **Risk Assessment**: Identifies potentially dangerous operations before execution

#### CLI Commands
```bash
# Migration status and management
npm run migrate:status          # Show current migration status
npm run migrate:up              # Apply all pending migrations
npm run migrate:rollback <ver>  # Rollback to specific version
npm run migrate:create          # Generate new migration template
npm run migrate:validate        # Validate migration integrity

# Advanced options
npm run migrate:rollback <ver> --force    # Force rollback (ignoring risks)
npm run migrate:up --dry-run              # Preview without executing
```

#### Migration File Structure
```typescript
// Example: 001_add_user_preferences.migration.ts
import { BaseMigration } from '../Migration.js';

export default class AddUserPreferences extends BaseMigration {
  id = '001_add_user_preferences';
  version = 1;
  name = 'Add User Preferences';
  description = 'Add user preferences table and columns';
  category = 'schema' as const;

  async up(): Promise<void> {
    await this.executeSQL(`
      CREATE TABLE user_preferences (
        id INTEGER PRIMARY KEY,
        user_id INTEGER NOT NULL,
        preference_key VARCHAR(100) NOT NULL,
        preference_value TEXT
      );
    `);
  }

  async down(): Promise<void> {
    await this.executeSQL(`DROP TABLE user_preferences;`);
  }
}
```

#### Health Check Integration
- **Startup Validation**: Checks migration status during server startup
- **Auto-Migration**: Optional automatic migration execution (configurable)
- **Health Endpoints**: Migration status included in health check responses
- **Error Prevention**: Prevents startup with critical migration issues

#### Safety Features
- **Backup Integration**: Automatic backups before risky operations
- **Production Checks**: Enhanced safety validations for production environments
- **Dependency Resolution**: Automatic validation of migration dependencies
- **Rollback Planning**: Safe rollback validation with risk assessment
- **Dry Run Mode**: Preview changes without executing them

## Current Development Status

### ✅ **Completed Major Features (Production Ready):**

#### **Phase 1: Production Infrastructure** ✅
- **StartupHealthCheck Service**: Comprehensive system validation with visual indicators
- **Smart Backup/Restore Logic**: Intelligent timestamp-based restoration with data loss prevention
- **Server State Tracking**: Operational metrics including uptime, session duration, and startup counts
- **Production Configuration**: Centralized config management with comprehensive validation
- **Enhanced Connection Pool**: PostgreSQL-compatible DuckDB interface with error handling
- **Optimized Startup Sequence**: 9-step production-ready initialization process
- **Enhanced Error Handling**: Detailed context and graceful degradation patterns

#### **Phase 2: Database Management** ✅
- **Database Migration System**: Comprehensive migration framework with rollback capabilities
  - Version control with unique IDs and dependencies
  - CLI tools for migration management (`migrate:status`, `migrate:up`, `migrate:rollback`, etc.)
  - Transaction safety and error handling
  - Risk assessment and backup integration
  - Health check integration and auto-migration support
  - Complete documentation and best practices guide

### 🎯 **Active Development Priorities:**

#### **📊 Medium Priority Tasks:**
1. **Monitoring & Alerting System** - Real-time health metrics and production notifications
2. **Backup Retention Policy** - Automated cleanup schedules and storage management
3. **Performance Monitoring** - Query timing and resource usage tracking
4. **Security Audit Logging** - Comprehensive audit trail for all user actions
5. **Backup Encryption** - Enhanced security for R2 backup storage
6. **Observability Dashboard** - Real-time system health visualization

#### **🔧 Low Priority Tasks:**
7. **Enhanced Rate Limiting** - User-specific limits and dynamic throttling
8. **Environment Config Validation** - Runtime validation and hot-reload capabilities
9. **Load Testing Framework** - Performance testing under stress conditions

### 📈 **Implementation Progress:**
- **Completed**: 2/2 High Priority items (100%)
- **In Progress**: 0/9 remaining tasks
- **Next Focus**: Monitoring & Alerting System

### 🎯 **Implementation Approach:**
Based on analysis of the sirex project's production-ready patterns, this implementation follows enterprise-grade practices:
- **Comprehensive Health Checks**: Visual status indicators and detailed component validation
- **Smart Operational Logic**: Intelligent decision-making (e.g., smart restore, auto-migration)
- **Operational Visibility**: Server state tracking and detailed logging with structured data
- **Production Safety**: Configuration validation, error handling, and graceful degradation
- **Developer Experience**: CLI tools, comprehensive documentation, and clear patterns

## Task Management

### Checking Plans and Todos
When asked to check plans or todos, always search the repository for:
- **Plan files**: `docs/plans/` directory and `**/*plan*`, `**/*PLAN*` patterns
- **Todo files**: `**/*todo*`, `**/*TODO*` patterns
- **Current implementation status**: Check if plan items are implemented in the codebase
- **Use TodoRead tool**: For active session todos
- **Development status**: Reference the Current Development Status section above