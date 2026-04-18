# Project Overview

This is a high-end social event platform targeting affluent individuals aged 45-65 with high net worth (NT$5M+ annual income or NT$30M+ net assets). The platform facilitates luxury social events like private dinners, yacht parties, and art appreciation gatherings.

## Current Status: Phase 1 Complete ✅
- **Authentication System**: Production-ready with JWT + Google OAuth
- **User Registration**: Complete 3-step registration with membership tiers
- **Admin Interface**: Full dashboard with user and backup management
- **Manual Backup System**: Preferred approach with complete admin controls

## Current Development Phase 🚧
- **Phase 3: Frontend Optimization & Analytics**: 🚧 **IN PROGRESS** - Lazy loading, code splitting, and analytics dashboard
- **All Major Systems**: ✅ **COMPLETED** - 13/13 major system components production ready
- **R2 Credentials Setup**: ✅ **COMPLETED** - Cloudflare R2 properly configured with SSL connectivity verified

## Completed Phases ✅
- **Event Content Management**: ✅ **Phase 1 COMPLETED** - Complete API architecture and database schema
- **Event Registration System**: ✅ **Phase 2 COMPLETED** - Complete member registration and management system
- **Sales Management System**: ✅ **Phase 3 COMPLETED** - CRM and sales pipeline for membership business
- **System Health Dashboard**: ✅ **Phase 4 COMPLETED** - Real-time monitoring and diagnostics interface
- **Event Media Management**: ✅ **Phase 5 COMPLETED** - R2 storage integration with seamless workflow
- **Admin Route Protection**: ✅ **Phase 6 COMPLETED** - Enterprise-grade frontend security system

## Technology Stack

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
- **Runtime**: Node.js 22.16.0 with Express framework
- **Language**: TypeScript with ESM modules
- **Database**: DuckDB as primary database with Cloudflare R2 persistence
- **Hosting**: Render Node web service via `render.yaml`
- **Security**: Helmet, CORS, rate limiting, compression
- **Logging**: Winston with Morgan middleware
- **File Processing**: Multer with Sharp for image processing
- **Payments**: Stripe integration
- **Health Monitoring**: Startup health checks and operational metrics

### Database
- **Primary**: DuckDB with schema defined in `database/duckdb-schema.sql`
- **Production Persistence**: Cloudflare R2 backup system
- **Development**: Local DuckDB file for development
- **Key Tables**: Users, Events, Venues, Event Categories, Registrations, Financial verification, Server state tracking

### Deployment Targets
- **Frontend**: Render static site `hesocial-frontend`
- **Backend**: Render Node web service `hesocial-api`
- **Cloudflare**: R2 storage/backups/media and possible external DNS/domain routing
- **Not currently configured**: Cloudflare Workers, Cloudflare Pages Functions, D1, KV, Durable Objects

See [Deployment Targets](./DEPLOYMENT_TARGETS.md).

## Project Architecture

### Monorepo Structure
```
hesocial/
├── frontend/          # React + Vite frontend application
├── backend/           # Express + TypeScript backend API
├── database/          # SQL schemas and seed data
├── docs/              # Documentation (this directory)
└── package.json       # Root workspace configuration
```

### Backend Server Variants
1. **Production Server** (`server.ts`): Node/Express with DuckDB and optional Cloudflare R2 persistence
2. **DuckDB Server** (`server-duckdb.ts`): Local DuckDB file for development
3. **Demo Server** (`server-demo.ts`): Mock data for quick prototyping

## Documentation Structure

- **API Reference**: `/docs/api/` - Complete API documentation
- **Authentication**: `/docs/authentication/` - Auth system details
- **Systems**: `/docs/systems/` - Individual system documentation
- **Commands**: `/docs/commands/` - Development commands and scripts
- **Configuration**: `/docs/configuration/` - Setup and configuration guides
- **Database**: `/docs/database/` - Database and migration information
- **Architecture**: `/docs/architecture/` - System architecture details
- **Development**: `/docs/development/` - Development workflows and guides
