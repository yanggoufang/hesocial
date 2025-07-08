# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with this luxury social event platform.

## Project Overview

High-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets) facilitating luxury events like private dinners, yacht parties, and art appreciation.

**Status**: Phase 1-3 Complete ✅ - 15/15 major system components production ready  
**Current Phase**: Phase 4 - Build System Fixes & TypeScript Optimization ✅ Major Progress
**Latest Update**: Sales Management System Complete + Chinese Localization - Full-stack CRM with luxury UI and Traditional Chinese interface

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

### ✅ Production Ready Systems (15/15)
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
- **Admin Interface Chinese Localization (Taiwan)**
- **Event Data Seeding System (12 Luxury Events)**
- **Frontend UI Bug Fixes (Dropdown Menu Visibility)**
- **Sales Management System (Full Stack with Luxury UI)**
- **Access Control Chinese Localization (Traditional Chinese)**

### ✅ Recently Completed - Sales Management System Complete + Chinese Localization
- **Sales Backend Routes** - Enabled comprehensive CRM API endpoints (leads, opportunities, activities, metrics)
- **Sales Frontend Interface** - Complete sales dashboard with luxury design theme and three-tab interface
- **Lead Management** - Auto-scoring, status tracking, financial data display, action buttons
- **Opportunity Pipeline** - Stage management, probability visualization, membership tier tracking
- **Sales Analytics** - KPI dashboard, sales funnel, conversion metrics, team performance
- **Luxury Design Applied** - Midnight black background, gold accents, glass effects, framer-motion animations
- **Chinese Localization** - Access control messages, verification warnings, role/membership translations
- **Customer Service UI** - Friendly MessageCircle icons replacing warning triangles for better UX

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

### Test Accounts
- **Admin**: `admin@hesocial.com` / `admin123`
- **Test User**: `test.platinum@example.com` / `test123`

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

### Authentication
- `POST /api/auth/login` - Email/password login
- `GET /api/auth/google` - Google OAuth
- `GET /api/auth/profile` - Get user profile

### Events
- `GET /api/events` - List events
- `POST /api/events` - Create event (Admin+)
- `GET /api/events/:id` - Event details

### Sales Management
- `GET /api/sales/leads` - Lead management with automatic scoring
- `GET /api/sales/opportunities` - Opportunity pipeline tracking
- `GET /api/sales/activities` - Activity and interaction logging
- `GET /api/sales/metrics` - Sales analytics and reporting
- `GET /api/sales/pipeline/stages` - Pipeline configuration
- `GET /api/sales/team` - Sales team management

### Admin
- `GET /api/admin/users` - User management (Admin+)
- `POST /api/admin/backup` - Create backup (Admin+)
- `GET /api/system/health` - System health (Admin+)

📖 **[Complete API Reference](docs/api/API_REFERENCE.md)**

## Task Management

When working on complex tasks:
1. Use TodoWrite tool for planning
2. Break down into manageable steps
3. Mark tasks complete as you finish
4. Reference documentation for context

**Remember**: This is a production-ready system. Follow established patterns, maintain quality standards, and ensure all changes are properly tested.