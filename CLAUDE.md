# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-end social event platform targeting affluent individuals aged 45-65 with high net worth (NT$5M+ annual income or NT$30M+ net assets). The platform facilitates luxury social events like private dinners, yacht parties, and art appreciation gatherings.

### **Current Status: Phase 1 Complete** ✅
- **Authentication System**: Production-ready with JWT + Google OAuth
- **User Registration**: Complete 3-step registration with membership tiers
- **Admin Interface**: Full dashboard with user and backup management
- **Manual Backup System**: Preferred approach with complete admin controls

### **Next Phase: Business Features** 🚀
- **Event Content Management**: ✅ **Phase 1 COMPLETED** - Complete API architecture and database schema
- **Event Registration System**: ✅ **Phase 2 COMPLETED** - Complete member registration and management system
- **Sales Management System**: ✅ **Phase 3 COMPLETED** - CRM and sales pipeline for membership business
- **System Health Dashboard**: ✅ **Phase 4 COMPLETED** - Real-time monitoring and diagnostics interface

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
- **Runtime**: Node.js 22.16.0 with Express framework
- **Language**: TypeScript with ESM modules
- **Database**: DuckDB as primary database with Cloudflare R2 persistence
- **Backup System**: AWS SDK v3 with smart restore logic and automatic backups
- **Migration System**: Comprehensive database migration framework with rollback capabilities
- **Authentication & User Management**: (COMPLETED - Production Ready)
  - JWT with Passport.js strategies (Google OAuth 2.0, email/password)
  - Role-based access control (user, admin, super_admin)
  - Automatic membership tier assignment based on financial verification
  - User profile management with comprehensive CRUD operations
  - Token refresh and validation system
  - OAuth callback handling with smart user creation/matching
  - Default admin accounts and comprehensive test users
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
  - Automatic backup on graceful shutdown with SIGTERM/SIGINT handling
  - Smart restore logic with timestamp comparison to prevent data loss
  - Manual backup API endpoints (`POST /api/admin/backup`, `GET /api/admin/backups`, `POST /api/admin/restore`)
  - Health monitoring endpoints (`/api/health/database`, `/api/health/r2-sync`, `/api/health/full`)
  - Environment separation (dev: `hesocial-duckdb-dev`, prod: `hesocial-duckdb`)
  - AWS SDK v3 integration with comprehensive error handling
- **Development**: Local DuckDB file for development
- **Key Tables**: Users, Events, Venues, Event Categories, Registrations, Financial verification, Server state tracking

## Development Environment & Troubleshooting

### Temporary Server for Development
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

### R2 Storage Configuration
For production media storage using Cloudflare R2:
```bash
# Required environment variables for R2 integration
R2_ENDPOINT=https://your-account.r2.cloudflarestorage.com
R2_ACCESS_KEY_ID=your-r2-access-key-id
R2_SECRET_ACCESS_KEY=your-r2-secret-access-key
R2_BUCKET_NAME=hesocial-media
R2_PUBLIC_URL=https://media.hesocial.com

# Features:
# - Automatic image optimization and thumbnail generation
# - Secure document storage with signed URLs
# - CDN delivery for public images
# - File validation and size limits (10MB)
# - Render.com compatible (no ephemeral file system dependency)
```

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

## ✅ **Completed Major Systems**

### **Event Content Management System** - Production Ready ✅
Complete luxury event lifecycle management with comprehensive API, database architecture, and frontend interface:

#### **Backend System** ✅
- **Event CRUD Operations**: Full REST API with admin permissions and role-based access control
- **Venue Management**: Luxury venue database with capacity, amenities, and location data
- **Event Categories**: Pre-configured categories (Private Dining, Yacht Parties, Art Appreciation, etc.)
- **Approval Workflows**: Multi-stage event approval process with draft→pending→approved→published flow
- **Membership Integration**: Event pricing and access control by membership tier (Platinum, Diamond, Black Card)
- **Rich Data Model**: Complete event information including scheduling, pricing, requirements, and metadata

#### **Frontend Management Interface** ✅
- **Event Management Dashboard**: Complete admin interface with filtering, pagination, and real-time operations
- **Event Creation/Editing System**: Comprehensive forms with multi-section organization and validation
- **Venue Management Interface**: Dedicated venue CRUD operations with amenities and contact management
- **Approval Workflow UI**: Real-time event approval/rejection with status tracking and publishing controls
- **Separated Route Architecture**: Dedicated `/events/manage` system separate from general admin routes
- **TypeScript Service Layer**: Complete API wrapper with error handling and type safety

#### **Complete System Architecture** ✅
```bash
# Backend API Endpoints
GET    /api/events                    # List events with filtering and pagination
GET    /api/events/:id                # Get specific event details  
POST   /api/events                    # Create new event (Admin+)
PUT    /api/events/:id                # Update event (Admin+)
DELETE /api/events/:id                # Delete event (Super Admin only)
POST   /api/events/:id/publish        # Publish event (Admin+)
POST   /api/events/:id/approve        # Approve event (Admin+)

GET    /api/venues                    # List luxury venues
POST   /api/venues                    # Create venue (Admin+)
PUT    /api/venues/:id                # Update venue (Admin+)
DELETE /api/venues/:id                # Delete venue (Super Admin only)

GET    /api/categories                # List event categories
POST   /api/categories                # Create category (Admin+)
PUT    /api/categories/:id            # Update category (Admin+)
DELETE /api/categories/:id            # Delete category (Super Admin only)

# Frontend Management Routes
/events/manage                       # Main event management dashboard
/events/venues                       # Venue management interface  
/events/categories                   # Category management interface
/admin                              # Admin dashboard with event quick actions

# Frontend Components
- EventManagement page (600+ lines) # Complete event CRUD interface
- EventForm component (500+ lines)  # Comprehensive creation/editing forms
- VenueManagement page (400+ lines) # Venue CRUD operations
- CategoryManagement page (700+ lines) # Category CRUD management system
- EventService (400+ lines)         # Complete TypeScript API wrapper
```

#### **Event Categories Pre-Configured** ✅
- ✅ **私人晚宴** (Private Dining Experiences) - Diamond, Black Card access
- ✅ **遊艇派對** (Yacht & Marine Events) - Diamond, Black Card access
- ✅ **藝術鑑賞** (Art & Culture Appreciation) - All tiers
- ✅ **商務人脈** (Business Networking) - Diamond, Black Card access
- ✅ **生活品味** (Wellness & Lifestyle) - All tiers
- ✅ **投資理財** (Investment & Finance Seminars) - Black Card only

### **Sales Management System** - Production Ready ✅
Complete CRM and sales pipeline system for luxury membership business with comprehensive tracking and analytics:

#### **Backend System** ✅
- **Sales Leads Management**: Complete lead tracking with automatic scoring, status management, and assignment
- **Sales Opportunities**: Formal sales processes with pipeline stages, probability tracking, and value management
- **Sales Activities**: Comprehensive interaction logging (calls, emails, meetings, demos, proposals)
- **Sales Pipeline**: Configurable stages with default probabilities and visual progress tracking
- **Sales Team Management**: Role-based team structure with territories, commission rates, and manager hierarchy
- **Sales Targets & Quotas**: Performance tracking with revenue, conversion, and lead targets by period
- **Commission Tracking**: Automated commission calculation and payment status management
- **Sales Analytics**: Comprehensive metrics including conversion rates, win rates, pipeline value, and cycle length

#### **API Implementation** ✅
```bash
# Sales Management API Endpoints
GET    /api/sales/leads               # List leads with advanced filtering and pagination
GET    /api/sales/leads/:id           # Get specific lead details with activity history
POST   /api/sales/leads               # Create lead with automatic scoring algorithm
PUT    /api/sales/leads/:id           # Update lead information and status
DELETE /api/sales/leads/:id           # Delete lead (Admin+ only)

GET    /api/sales/opportunities       # List opportunities with pipeline tracking
POST   /api/sales/opportunities       # Create formal sales opportunity
PUT    /api/sales/opportunities/:id   # Update opportunity stage and details

GET    /api/sales/activities          # List sales activities with filtering
POST   /api/sales/activities          # Log sales interaction or outcome

GET    /api/sales/metrics             # Comprehensive sales analytics and KPIs
GET    /api/sales/pipeline/stages     # Get configurable pipeline stages
GET    /api/sales/team                # Sales team structure and performance
```

#### **Database Schema** ✅
- **sales_leads**: Lead information with scoring, status, and assignment tracking
- **sales_opportunities**: Formal sales processes with stages and probability
- **sales_activities**: Complete interaction history and outcome tracking
- **sales_pipeline_stages**: Configurable pipeline with visual indicators
- **sales_targets**: Quota and performance tracking by sales rep and period
- **sales_team_members**: Team structure with roles and commission management
- **sales_commissions**: Automated commission calculation and payment tracking

### **User Management System** - Production Ready ✅
Complete admin interface for user account management with comprehensive CRUD operations and analytics:

#### **Frontend Interface** ✅
- **User Dashboard**: Real-time statistics including total users, recent registrations, pending verifications
- **Advanced Search & Filtering**: Search by name/email/profession, filter by role/membership/verification status
- **User Operations**: View details, edit profiles, verify accounts, manage roles, delete users
- **Membership Management**: Visual tier badges and controls for Platinum/Diamond/Black Card management
- **Financial Information**: Formatted display of annual income and net worth with currency formatting
- **Role-Based Security**: Admin and Super Admin access control with proper permission validation

#### **User Management Routes** ✅
```bash
# User Management Frontend
/admin/users                         # Complete user management interface

# User Management API (Already Existing)
GET    /api/users                    # List users with pagination and filtering
GET    /api/users/:id                # Get specific user details
PUT    /api/users/:id                # Update user information (Admin+)
DELETE /api/users/:id                # Delete user (Super Admin only)
POST   /api/users/:id/verify         # Approve/reject user verification (Admin+)
POST   /api/users/:id/role           # Update user role (Super Admin only)
GET    /api/users/stats/overview     # User statistics and analytics
```

#### **Features** ✅
- **Comprehensive User Display**: Avatar, contact info, profession, membership tier, verification status
- **Bulk Operations**: Pagination with 20 users per page, efficient loading and filtering
- **Modal Interfaces**: Detailed user view, edit form, and delete confirmation dialogs
- **Real-time Updates**: Immediate UI updates after operations with proper error handling
- **Security Implementation**: Role validation and authentication checks throughout

### **Category Management System** - Production Ready ✅
Visual event category management with complete CRUD operations and customization options:

#### **Frontend Interface** ✅
- **Visual Category Cards**: Color-coded design with icons, descriptions, and metadata
- **Category Configuration**: Name, description, icon selection, color customization
- **Capacity Management**: Minimum and maximum capacity settings for each category
- **Membership Targeting**: Target specific membership tiers for exclusive events
- **Sort Order Management**: Visual up/down controls for category organization
- **Active Status Toggle**: Enable/disable categories with visual indicators

#### **Category Management Routes** ✅
```bash
# Category Management Frontend
/events/categories                   # Complete category management interface

# Category Management API (Already Existing)
GET    /api/categories               # List categories (role-based visibility)
GET    /api/categories/:id           # Get specific category details
POST   /api/categories               # Create category (Admin+)
PUT    /api/categories/:id           # Update category (Admin+)
DELETE /api/categories/:id           # Delete category (Super Admin only)
```

#### **Features** ✅
- **Visual Design System**: Custom icons, color palette, and luxury styling
- **Advanced Configuration**: Duration settings, capacity ranges, membership tier targeting
- **Search and Organization**: Category search and sort order management
- **Integration**: Seamless integration with EventManagement page navigation
- **Error Handling**: Comprehensive error states and validation

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
- **Periodic Backups**: Configurable automatic backups with interval scheduling (disabled by default, manual preferred)
- **Graceful Shutdown**: Clean shutdown with automatic backup creation and state recording

### Database Architecture
- **User Management**: Multi-tier membership system (Platinum, Diamond, Black Card)
  - Role-based access control (user, admin, super_admin)
  - Default admin accounts for system administration
  - Comprehensive test users for development and testing
  - Financial verification and automatic tier assignment
- **Event Content Management System** ✅: Complete luxury event lifecycle management
  - **Events Table**: Complete event information with approval workflows and membership-based pricing
  - **Venues Table**: Luxury venue database with capacity, amenities, and location data
  - **Event Categories Table**: Pre-configured luxury event types with membership tier targeting
  - **Event Registrations**: Member participation tracking with payment and status management
  - **Event Feedback**: Rating and review system for event quality control
  - **Event Waitlist**: Queue management for popular events
- **Registration System**: Event registration with approval workflows
- **Migration System**: Version-controlled schema changes with rollback capabilities
- **Operational Tracking**: Server state, migration history, and audit trails

### Authentication & Security
- **JWT-based Authentication**: (COMPLETED - Production Ready)
  - Secure token generation with configurable expiration
  - Automatic token refresh mechanism
  - User session persistence and validation
- **User Registration System**: (COMPLETED - Production Ready)
  - Complete 3-step registration form with real-time validation
  - Automatic membership tier assignment based on financial data
  - Seamless integration with authentication API
  - End-to-end flow from registration to dashboard
- **OAuth 2.0 Integration**: (Google OAuth COMPLETED - Production Ready)
  - Google OAuth 2.0 with Passport.js strategy
  - Smart user creation and profile matching
  - Seamless callback handling and token generation
  - LinkedIn OAuth (configured but not implemented)
- **Security Features**:
  - Role-based access control with admin and super admin roles
  - Rate limiting and request validation
  - Helmet.js for security headers
  - CORS configuration for cross-origin requests
  - Password hashing with bcryptjs (12 salt rounds)
  - Membership tier-based access control
  - Protected admin API endpoints with authentication middleware

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
- **Authentication Integration**: (COMPLETED - Production Ready)
  - React authentication context with useAuth hook
  - Complete login and registration pages with real API calls
  - Session persistence and automatic token management
  - Protected routes and role-based access control

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
- **Authentication API endpoints** (Production Ready):
  - `POST /api/auth/register` - User registration with automatic membership tier assignment
  - `POST /api/auth/login` - Email/password authentication with JWT token generation
  - `GET /api/auth/google` - Initiate Google OAuth 2.0 authentication flow
  - `GET /api/auth/google/callback` - Google OAuth callback with user creation/matching
  - `GET /api/auth/profile` - Get authenticated user profile
  - `PUT /api/auth/profile` - Update user profile information
- **Admin API endpoints** (Production Ready):
  - `POST /api/admin/backup` - Create manual backup (Admin+)
  - `GET /api/admin/backups` - List available backups (Admin+)
  - `POST /api/admin/restore` - Restore database (Super Admin only)
  - `GET /api/users` - User management with pagination and filtering (Admin+)
  - `POST /api/users/:id/verify` - User verification management (Admin+)
  - `POST /api/users/:id/role` - Role management (Super Admin only)
  - `POST /api/auth/refresh` - Refresh JWT token
  - `POST /api/auth/logout` - Logout (client-side token removal)
  - `GET /api/auth/validate` - Validate JWT token and return user data
- **Event Management API endpoints** (Production Ready):
  - `GET /api/events` - List events with filtering and pagination (Public with role-based visibility)
  - `POST /api/events` - Create new event (Admin+)
  - `GET /api/events/:id` - Get specific event details (Public with role-based data)
  - `PUT /api/events/:id` - Update event (Admin+)
  - `DELETE /api/events/:id` - Delete event (Super Admin only)
  - `POST /api/events/:id/publish` - Publish event (Admin+)
  - `POST /api/events/:id/approve` - Approve event (Admin+)
  - `GET /api/venues` - List venues (Public active venues, Admin sees all)
  - `POST /api/venues` - Create venue (Admin+)
  - `GET /api/venues/:id` - Get venue details (Public if active, Admin sees all)
  - `PUT /api/venues/:id` - Update venue (Admin+)
  - `DELETE /api/venues/:id` - Delete venue (Super Admin only)
  - `GET /api/categories` - List event categories (Public active categories, Admin sees all)
  - `POST /api/categories` - Create category (Admin+)
  - `GET /api/categories/:id` - Get category details (Public if active, Admin sees all)
  - `PUT /api/categories/:id` - Update category (Admin+)
  - `DELETE /api/categories/:id` - Delete category (Super Admin only)
- **Event Registration API endpoints** (Production Ready):
  - `POST /api/registrations` - Register for event with eligibility checking (User)
  - `GET /api/registrations/user/:userId` - Get user registrations with filtering and pagination (User owns, Admin all)
  - `DELETE /api/registrations/:id` - Cancel registration (User owns, Admin all)
  - `GET /api/registrations/event/:eventId` - Get event registrations (Admin+ only)
  - `PUT /api/registrations/:id/approve` - Approve registration (Admin+ only)
  - `PUT /api/registrations/:id/reject` - Reject registration (Admin+ only)
  - `GET /api/registrations/stats` - Registration statistics and analytics (Admin+ only)
- **Event Media Management API endpoints** (Production Ready):
  - `POST /api/media/events/:eventId/images` - Upload event images with automatic thumbnails (Owner/Admin+)
  - `POST /api/media/events/:eventId/documents` - Upload event documents with secure storage (Owner/Admin+)
  - `GET /api/media/events/:eventId` - Get event media with optional type filtering (Public with auth-aware URLs)
  - `POST /api/media/venues/:venueId/images` - Upload venue images with multiple sizes (Admin+ only)
  - `GET /api/media/venues/:venueId` - Get venue media gallery (Public)
  - `DELETE /api/media/:mediaId` - Delete media file and R2 storage cleanup (Owner/Admin+)
  - `GET /api/media/download/:mediaId` - Generate signed URL for document download (Auth required)
  - `GET /api/media/stats` - Media storage statistics and analytics (Admin+ only)
  - `POST /api/media/cleanup` - Clean up orphaned media files (Super Admin only)
- **Admin endpoints** (Production Ready - Requires Authentication):
  - `POST /api/admin/backup` - Create manual backup to R2 (Admin+)
  - `GET /api/admin/backups` - List available R2 backups with metadata (Admin+)
  - `POST /api/admin/restore` - Restore from specific R2 backup (Super Admin only)
  - `POST /api/admin/cleanup` - Clean up old backups (Admin+)
- **Enhanced health endpoints**:
  - `GET /api/health` - Basic API health check
  - `GET /api/health/database` - Database connection and query performance
  - `GET /api/health/r2-sync` - R2 backup service status and connectivity
  - `GET /api/health/full` - Comprehensive system status with all components
- **System Health Dashboard API endpoints** (Production Ready - Admin+ only):
  - `GET /api/system/health` - Comprehensive system health dashboard data
  - `GET /api/system/health/detailed` - Detailed health check with all components
  - `GET /api/system/metrics` - System performance metrics and resource usage
  - `GET /api/system/diagnostics` - Run system diagnostics tests

## Authentication System

### Authentication Framework (Production Ready)
A comprehensive authentication system implementing enterprise-grade security patterns with JWT tokens and OAuth 2.0 integration:

#### Core Features
- **Multi-Strategy Authentication**: Email/password and Google OAuth 2.0 support
- **JWT Token Management**: Secure token generation, validation, and refresh
- **Automatic User Management**: Smart user creation and profile matching for OAuth
- **Membership Tier Assignment**: Automatic tier calculation based on financial verification
- **Session Persistence**: Seamless user session management across app reloads
- **Security Middleware**: Route protection and role-based access control

#### Frontend Integration
```typescript
// Authentication Context Usage
const { user, login, loginWithGoogle, logout, isAuthenticated } = useAuth()

// Login with email/password
const result = await login({ email, password })

// Initiate Google OAuth flow
loginWithGoogle()

// Check authentication status
if (isAuthenticated) {
  // User is logged in
}
```

#### Backend Implementation
```typescript
// Authentication Routes
POST /api/auth/register    // User registration
POST /api/auth/login       // Email/password login
GET  /api/auth/google      // Google OAuth initiation
GET  /api/auth/google/callback // OAuth callback
GET  /api/auth/profile     // Get user profile
PUT  /api/auth/profile     // Update profile
POST /api/auth/refresh     // Token refresh
```

#### OAuth Configuration
```bash
# Google OAuth Setup (Development)
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# OAuth Callback URLs
# Development: http://localhost:5000/api/auth/google/callback
# Production: https://yourdomain.com/api/auth/google/callback
```

#### Security Features
- **Password Hashing**: bcryptjs with salt rounds for secure password storage
- **JWT Security**: Configurable expiration times and refresh token mechanism
- **OAuth Security**: Secure callback handling with state validation
- **Route Protection**: Authentication middleware for protected endpoints
- **User Verification**: Multi-tier verification system for financial validation

#### User Management
- **Registration Flow**: Complete user onboarding with financial verification
- **Profile Management**: Comprehensive user profile CRUD operations
- **Role-Based Access Control**: Three-tier admin system
  - `user`: Standard members with tier-based access
  - `admin`: Event management and member operations
  - `super_admin`: Full system administration capabilities
- **Membership Tiers**: Automatic assignment based on income/net worth
  - Platinum: Default tier for new users
  - Diamond: 30M+ NTD net worth or 5M+ annual income
  - Black Card: 100M+ NTD net worth or 20M+ annual income
- **Default Accounts**: 
  - 3 Admin accounts for system administration
  - 5 Test accounts for development and testing
  - 8 Realistic sample users with complete profiles

## Default Users and Admin System

### Admin Accounts (Production Ready)
Default administrative accounts for system management:

| Email | Password | Role | Access Level |
|-------|----------|------|--------------|
| `admin@hesocial.com` | `admin123` | super_admin | Full system access |
| `superadmin@hesocial.com` | `admin123` | super_admin | Full system access |
| `events@hesocial.com` | `admin123` | admin | Event management |

### Test User Accounts
Comprehensive test accounts for development and QA:

| Email | Password | Membership | Purpose |
|-------|----------|------------|---------|
| `test.platinum@example.com` | `test123` | Platinum | Platinum tier testing |
| `test.diamond@example.com` | `test123` | Diamond | Diamond tier testing |
| `test.blackcard@example.com` | `test123` | Black Card | Black Card tier testing |
| `test.pending@example.com` | `test123` | Pending | Verification flow testing |
| `test.oauth@gmail.com` | No password | OAuth User | Google OAuth testing |

### Sample Users
8 realistic user profiles with complete backgrounds:
- **陳志明** - Tech CEO (Diamond Member)
- **王美麗** - Investment Banker (Black Card Member)
- **林建華** - Architect (Platinum Member)
- **張雅婷** - Plastic Surgeon (Diamond Member)
- **劉國強** - Real Estate Developer (Black Card Member)
- **黃淑芬** - Hedge Fund Manager (Diamond Member)
- **吳俊傑** - Serial Entrepreneur (Platinum Member)
- **李心怡** - International Lawyer (Diamond Member)

### Role-Based Access Control
```typescript
// Middleware protection levels
requireAdmin()      // Requires 'admin' or 'super_admin' role
requireSuperAdmin() // Requires 'super_admin' role only

// API endpoint protection
POST /api/admin/backup     // Admin+ required
POST /api/admin/restore    // Super Admin only
GET  /api/admin/backups    // Admin+ required
```

### Security Implementation
- **Password Hashing**: bcrypt with 12 salt rounds for all accounts
- **Role Validation**: Comprehensive middleware for admin route protection
- **Default Security**: Production-ready admin accounts with proper access control
- **Test Coverage**: Complete user scenarios for development and testing

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

#### **Phase 3: Authentication & User Management System** ✅
- **Complete Authentication Framework**: Production-ready authentication with JWT and OAuth 2.0
  - Email/password authentication with secure password hashing (bcrypt, 12 salt rounds)
  - Google OAuth 2.0 integration with Passport.js strategies
  - JWT token management with automatic refresh and validation
  - User registration with automatic membership tier assignment
  - Profile management with comprehensive CRUD operations
  - OAuth callback handling with smart user creation/matching
  - Frontend authentication context with session persistence
  - Security middleware for route protection and access control
- **Role-Based Access Control System**: Complete admin and user management
  - Three-tier role system (user, admin, super_admin)
  - Protected admin API endpoints with authentication middleware
  - Default admin accounts for system administration
  - Comprehensive test users for development and testing
  - Role validation and authorization throughout the system

#### **Phase 4: Event Content Management System** ✅
- **Complete Event Lifecycle Management**: Production-ready event management for luxury social experiences
  - Event CRUD operations with comprehensive REST API and role-based access control
  - Venue management system with luxury venue database (capacity, amenities, location data)
  - Event category system with pre-configured luxury event types and membership tier targeting
  - Event approval workflows with multi-stage process (draft→pending→approved→published)
  - Membership-based pricing and access control (Platinum, Diamond, Black Card tiers)
  - Rich data model with complete event information (scheduling, pricing, requirements, metadata)
  - Database schema with events, venues, categories, registrations, feedback, and waitlist tables
  - Admin-controlled venue and category management with public discovery interfaces
  - Event visibility controls with role-based data access and admin-only sensitive information

#### **Phase 5: Event Registration System** ✅
- **Complete Member Registration Management**: Production-ready event registration system with comprehensive user and admin interfaces
  - Member registration with eligibility checking and capacity management
  - Event registration workflow with approval processes and status tracking
  - User registration dashboard with filtering, pagination, and cancellation functionality
  - Admin registration management with bulk operations and detailed analytics
  - Registration statistics and reporting with conversion tracking
  - Membership tier-based registration validation and verification status checking
  - Integration with existing authentication and event management systems
  - Temporary server implementation for development and testing continuity

#### **Phase 6: Event Media Management System** ✅
- **Complete Media Management with R2 Storage**: Production-ready media system for scalable file handling
  - Cloudflare R2 integration for reliable, CDN-delivered media storage
  - Image processing with Sharp including automatic thumbnail generation (thumb, medium, large)
  - Document management with secure private storage and expiring signed URLs
  - Multi-file upload system with drag & drop interface and validation
  - Media gallery with lightbox, download, and management capabilities
  - Role-based permissions for upload, view, and delete operations
  - File type validation (images: JPEG, PNG, WebP, GIF; documents: PDF, DOC, XLS)
  - Size limits (10MB per file) and comprehensive error handling
  - Database schema with event_media and venue_media tables plus indexing
  - Complete REST API with admin controls and media statistics

#### **Phase 7: Admin Route Protection System** ✅
- **Enterprise-Grade Frontend Security**: Comprehensive route and component-level access control system
  - Multi-layer protection with authentication, role-based, membership-based, and verification-based access
  - ProtectedRoute component with hierarchical role validation (user → admin → super_admin)
  - Membership tier protection (Platinum → Diamond → Black Card) with automatic tier checking
  - Specialized route guards for different access levels (AdminRoute, SuperAdminRoute, VVIPRoute)
  - AccessControl component for conditional rendering with flexible fallback options
  - Permission hooks (useRoleAccess, usePermissions) for real-time access evaluation
  - Professional unauthorized access pages with clear error messages and suggestions
  - Dynamic navigation with role-based menu items and admin indicators
  - Comprehensive testing system with AccessTestPage for debugging and validation
  - Loading states and smooth transitions during authentication checks

#### **Phase 8: System Health Dashboard** ✅
- **Enterprise-Grade Monitoring Interface**: Comprehensive real-time system monitoring and diagnostics
  - Complete system health overview with overall status indicators and auto-refresh (30s intervals)
  - Detailed component health checks with environment variables, database, R2 backup, payment, and authentication validation
  - System performance metrics including memory usage, CPU time, uptime, and Node.js version information
  - System diagnostics with automated tests for database connection, R2 connectivity, and configuration validation
  - Service status monitoring for authentication (OAuth providers, JWT), payments (Stripe), and media storage (R2)
  - Database statistics with table counts, user metrics, event counts, and connection pool information
  - R2 backup monitoring with backup count, storage size, connection health, and recent backup history
  - Security status overview with HTTPS, CORS, rate limiting, and Helmet configuration
  - Migration status tracking with applied/pending counts and latest migration information
  - Interactive tab-based interface with Overview, Detailed, Metrics, and Diagnostics sections
  - Admin-only access with role-based route protection and integrated navigation
  - Visual status indicators with color-coded health states and professional error handling

#### **Phase 9: Event Media Integration** ✅
- **Seamless Workflow Integration**: Complete integration of media management into event and venue workflows
  - Enhanced EventForm with integrated media upload and gallery sections for existing events
  - Dedicated EventMediaManagement page (/events/:eventId/media) with comprehensive media interface
  - Tab-based navigation (All Media, Images, Documents) for organized media viewing and management
  - Real-time media gallery updates with automatic refresh after uploads and deletions
  - Dual upload areas with separate file type validation for images and documents
  - Event context display with event details, date, venue, and status for user orientation
  - Professional drag & drop upload interface with clear file type restrictions and size limits
  - Media preview integration in EventManagement page with compact gallery displays
  - "Media" management button for direct access to dedicated media interface
  - Venue media integration in VenueManagement page with image gallery previews
  - File type validation (images: JPEG, PNG, WebP, GIF; documents: PDF, DOC, XLS)
  - Size limits (10MB per file) with comprehensive error handling and user feedback
  - Admin-only access protection with EventManagementRoute and proper authorization
  - Responsive design optimized for both desktop and mobile media management workflows
  - Professional luxury styling consistent with platform design language

### ✅ **Recently Completed (Phase 2):**

#### **🔴 High Priority Tasks - COMPLETED:**
1. **Sales Management System**: ✅ **Complete CRM and sales pipeline API backend**
2. **UserManagement Page**: ✅ **Complete admin user management interface**  
3. **Event Category Management**: ✅ **Complete category CRUD interface**
4. **System Health Dashboard**: ✅ **Complete admin monitoring interface for system status**
5. **Event Media Integration**: ✅ **Complete seamless media workflow integration**

### 🎯 **Active Development Priorities:**

#### **📊 High Priority Tasks:**
1. **Main Server Database Fix**: Resolve DuckDB connection issues for production server
2. **Frontend Route Optimization**: Lazy loading and code splitting for better performance
3. **Event Analytics Dashboard**: Event performance metrics and member engagement tracking
4. **Sales Management Frontend**: Complete sales pipeline management interface

#### **🔧 Medium Priority Tasks:**
5. **Event Calendar Integration**: Advanced scheduling and calendar management
6. **Mobile Responsive Admin**: Mobile-optimized admin interface components
7. **Advanced Event Filters**: Enhanced filtering and search for events and users
8. **Event Registration Analytics**: Registration conversion tracking and member engagement metrics

#### **🔧 Low Priority Tasks:**
9. **Advanced Notifications**: Real-time notification system for admins and users
10. **Report Generation**: Automated reporting for events, sales, and user analytics

### 📈 **Implementation Progress:**
- **Completed**: 10/10 Major System Components (100% of Phase 1 & Phase 2)
- **Authentication System**: ✅ Production Ready
- **R2 Backup System**: ✅ Production Ready  
- **Database Migration System**: ✅ Production Ready
- **Event Content Management API**: ✅ Production Ready
- **Event Management Frontend**: ✅ Production Ready
- **Sales Management System**: ✅ Production Ready (API Backend)
- **User Management Interface**: ✅ Production Ready
- **Category Management System**: ✅ Production Ready
- **Event Registration System**: ✅ Production Ready (Frontend + Backend)
- **Event Media Management System**: ✅ Production Ready (R2 Storage + Full UI)
- **Admin Route Protection System**: ✅ Production Ready (Enterprise-Grade Security)
- **System Health Dashboard**: ✅ Production Ready (Real-time Monitoring)
- **Event Media Integration**: ✅ Production Ready (Seamless Workflow Integration)
- **Next Focus**: Frontend Route Optimization and Event Analytics Dashboard

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