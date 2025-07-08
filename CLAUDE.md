# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this luxury social event platform.

## Project Overview

High-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets) facilitating luxury events like private dinners, yacht parties, and art appreciation.

**Status**: Phase 1-13 Complete ✅ - 17/17 major system components production ready  
**Current Phase**: Phase 14 - Advanced Features & Polish
**Latest Update**: Phase 13 Complete - All high priority analytics and frontend optimization tasks completed

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

### ✅ Production Ready Systems (16/16)
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

### ✅ Recently Completed - Phase 13 Advanced Analytics & Frontend Optimization
- **Event Registration Frontend** - Replaced mock data with real API integration and user feedback
- **Profile API Integration** - Complete real-time data integration with loading states
- **Frontend Route Optimization** - Enhanced lazy loading, bundle optimization, and progressive loading
- **Event Analytics Dashboard** - Complete backend API with overview, performance, and engagement metrics
- **Backend API Routes** - All critical routes uncommented and functional (auth, registration, system health)
- **Authentication Middleware** - Dev token bypass implemented for seamless development workflow
- **Registration System API** - GET /api/registrations/user endpoint fully operational
- **System Health Monitoring** - Complete admin dashboard with real-time diagnostics

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

### ✅ **PHASE 13 COMPLETED (July 8, 2025)**
**Advanced Analytics & Frontend Optimization**

All high priority Phase 13 tasks have been completed:
- ✅ Event Registration Frontend - Real API integration with user feedback
- ✅ Profile API Integration - Complete real-time data integration
- ✅ Frontend Route Optimization - Enhanced lazy loading and bundle optimization
- ✅ Event Analytics Dashboard - Complete backend API with performance metrics

### 🎯 **PHASE 14 CURRENT FOCUS**
**Advanced Features & Polish**

**Medium Priority Tasks:**
1. **Advanced Event Filters** - Enhanced filtering and search capabilities
2. **Mobile Responsive Admin** - Mobile-optimized admin interface components
3. **Event Calendar Integration** - Advanced scheduling and calendar management
4. **Registration Analytics** - Conversion tracking and member engagement metrics

**Low Priority Tasks:**
5. **Contact Request System** - Implement participant contact request functionality
6. **Google Analytics 4 Integration** - Complete GA4 setup using visitor tracking foundation
7. **Advanced Social Features** - Member messaging system and forums

## Task Management

When working on complex tasks:
1. Use TodoWrite tool for planning
2. Break down into manageable steps
3. Mark tasks complete as you finish
4. Reference documentation for context

**Remember**: This is a production-ready system with all critical backend infrastructure complete. Focus on frontend optimization and advanced analytics features. All core API endpoints are functional and properly authenticated.