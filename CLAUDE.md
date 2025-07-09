# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this luxury social event platform.

## Project Overview

High-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets) facilitating luxury events like private dinners, yacht parties, and art appreciation.

**Status**: Phase 1-13 Complete ✅ - 17/17 major system components production ready  
**Current Phase**: Phase 14 - Advanced Features & Polish  
**Latest Update**: Social Networking Frontend Complete - Participant discovery and privacy management system fully operational

📖 **[Full Project Overview](docs/PROJECT_OVERVIEW.md)**

## Quick Start

### Development Commands
```bash
npm run setup    # Install dependencies
npm run dev      # Start frontend + backend
npm run build    # Build project
npm run test     # Run tests
```

📖 **[Complete Development Commands](docs/commands/DEVELOPMENT_COMMANDS.md)**

## System Status

### ✅ Production Ready Systems (17/17)
- Authentication System (JWT + OAuth 2.0)
- Event Content Management (Full Stack)
- Event Registration System (Full Stack)
- Event Media Management (R2 Storage)
- Sales Management System (Full Stack)
- User Management (Full Stack)
- Database Migration System
- Admin Route Protection
- System Health Dashboard
- R2 Backup System
- Frontend Route Optimization (Lazy Loading + Code Splitting)
- Admin Console Health Monitoring
- API Route Management
- Development Health Endpoints
- **Participant Access Control System (Social Networking Foundation)**
- **Social Networking Frontend (Participant Discovery & Privacy Management)**
- **Critical API Endpoints (Backend Infrastructure)**

### ✅ Recently Completed - Social Networking Frontend (July 9, 2025)
- **Participant Discovery Frontend** - Complete event participant viewing with payment gate integration
- **Privacy Management System** - 5-tier privacy control interface with visual indicators
- **Database Interface Migration** - DuckDB async operations fully implemented
- **Participant Access Control** - Backend API endpoints for participant discovery and privacy settings
- **Payment-Gated Social Features** - Restricted participant access based on event payment status
- **Luxury Design Integration** - Consistent midnight black theme with gold accents across all components
- **Real-time Privacy Controls** - Dynamic privacy settings with immediate effect on participant visibility
- **Contact Management System** - Foundation for participant contact requests and messaging

📖 **[Detailed System Status](docs/systems/COMPLETED_SYSTEMS.md)**  
📖 **[Current Development Status](docs/systems/DEVELOPMENT_STATUS.md)**

## Technology Stack

**Frontend**: React 18 + TypeScript + Vite + Tailwind CSS  
**Backend**: Node.js 22 + Express + TypeScript + DuckDB  
**Storage**: Cloudflare R2 + Local DuckDB  
**Auth**: JWT + Google OAuth 2.0  

## Development Environment

### Database Modes
- **Full Mode**: `npm run dev` - DuckDB with R2 backup
- **DuckDB Mode**: `npm run dev:duckdb` - Local DuckDB only

### Test Accounts & Authentication
- **Admin**: `admin@hesocial.com` / `admin123`
- **Test User**: `test.platinum@example.com` / `test123`
- **Dev Token**: `dev-token-12345` (for development API testing)

📖 **[Authentication System](docs/authentication/AUTHENTICATION_SYSTEM.md)**

## Key Implementation Notes

### Code Style
- **NO COMMENTS** unless explicitly requested
- Follow existing patterns and conventions
- Use TypeScript strict mode
- Prefer editing existing files over creating new ones

### Security
- Never commit secrets or credentials
- Use role-based access control
- Validate all inputs
- Follow enterprise security patterns

### Development Workflow
1. Check existing implementation in codebase
2. Use appropriate tools (TodoWrite for complex tasks)
3. Follow testing requirements
4. Run lint/typecheck before completion

## Configuration

### R2 Storage (Cloudflare)
⚠️ **Setup Required**: Replace placeholder credentials with actual values
📖 **[R2 Configuration Guide](docs/configuration/R2_CONFIGURATION.md)**

### Database
- **Schema**: `database/duckdb-schema.sql`
- **Migrations**: `npm run migrate:status`
- **Seeding**: `npm run seed`

📖 **[Database System](docs/database/DATABASE_SYSTEM.md)**

## Documentation Structure

- **`docs/PROJECT_OVERVIEW.md`** - Complete project information
- **`docs/commands/`** - Development commands and scripts
- **`docs/systems/`** - Individual system documentation
- **`docs/authentication/`** - Authentication system details
- **`docs/database/`** - Database and migration information
- **`docs/configuration/`** - Setup and configuration guides
- **`docs/api/`** - API documentation
- **`docs/architecture/`** - System architecture
- **`docs/development/`** - Development workflows

## API Endpoints (Summary)

### Authentication ✅ **WORKING**
- `POST /api/auth/login` - Email/password login
- `GET /api/auth/google` - Google OAuth
- `GET /api/auth/profile` - Get user profile

### Events ✅ **WORKING**
- `GET /api/events` - List events
- `POST /api/events` - Create event (Admin+)
- `GET /api/events/:id` - Event details

### Registration System ✅ **WORKING**
- `GET /api/registrations/user` - User registration history
- `POST /api/registrations/events/:eventId` - Register for event
- `GET /api/registrations/:id` - Registration details

### Participant Discovery ✅ **WORKING**
- `GET /api/events/:eventId/participants` - List event participants with privacy filtering
- `GET /api/events/:eventId/participant-access` - Check viewer's access level
- `GET /api/events/:eventId/participants/:participantId` - Get participant details
- `POST /api/events/:eventId/participants/:participantId/contact` - Initiate contact
- `GET /api/events/:eventId/privacy-settings` - Get user's privacy settings
- `PUT /api/events/:eventId/privacy-settings` - Update privacy settings

### Sales Management ✅ **WORKING**
- `GET /api/sales/leads` - Lead management with automatic scoring
- `GET /api/sales/opportunities` - Opportunity pipeline tracking
- `GET /api/sales/activities` - Activity and interaction logging
- `GET /api/sales/metrics` - Sales analytics and reporting
- `GET /api/sales/pipeline/stages` - Pipeline configuration
- `GET /api/sales/team` - Sales team management

### Admin & System Health ✅ **WORKING**
- `GET /api/admin/users` - User management (Admin+)
- `POST /api/admin/backup` - Create backup (Admin+)
- `GET /api/system/health` - System health overview
- `GET /api/system/health/detailed` - Detailed health diagnostics
- `GET /api/system/metrics` - Performance metrics
- `GET /api/system/diagnostics` - System diagnostics

📖 **[Complete API Reference](docs/api/API_REFERENCE.md)**

## Current Development Status

### ✅ **SOCIAL NETWORKING FRONTEND COMPLETED (July 9, 2025)**
**Participant Discovery & Privacy Management System**

All critical social networking features have been implemented:
- ✅ Participant Discovery Frontend - Complete event participant viewing with payment gate
- ✅ Privacy Management Interface - 5-tier privacy control system with visual indicators
- ✅ Database Interface Migration - DuckDB async operations fully operational
- ✅ Backend API Endpoints - Complete participant access control system
- ✅ Payment-Gated Access - Restricted participant viewing based on event payment status
- ✅ Luxury Design Integration - Consistent midnight black theme with gold accents

### 🎯 **PHASE 14 CURRENT FOCUS**
**Advanced Features & Polish**

**High Priority Tasks:**
1. **Advanced Event Filters** - Enhanced filtering and search capabilities for events and participants
2. **Mobile Responsive Admin** - Mobile-optimized admin interface components
3. **Event Calendar Integration** - Advanced scheduling and calendar management

**Medium Priority Tasks:**
4. **Registration Analytics** - Conversion tracking and member engagement metrics
5. **Contact Request System** - Implement participant contact request functionality with messaging
6. **Advanced Social Features** - Member messaging system and social forums

**Low Priority Tasks:**
7. **Google Analytics 4 Integration** - Complete GA4 setup using visitor tracking foundation
8. **Push Notifications** - Real-time notifications for events and social interactions
9. **Advanced Member Matching** - AI-powered participant recommendations

## Task Management

When working on complex tasks:
1. Use TodoWrite tool for planning
2. Break down into manageable steps
3. Mark tasks complete as you finish
4. Reference documentation for context

**Remember**: This is a production-ready system with complete social networking functionality. All 17 major system components are operational including participant discovery, privacy management, and payment-gated social features. Focus on advanced features and polish for Phase 14.