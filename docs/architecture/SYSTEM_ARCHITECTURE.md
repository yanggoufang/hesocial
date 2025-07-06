# System Architecture

## Monorepo Structure

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

## Backend Server Variants

The backend supports three different server configurations:

1. **Production Server** (`server.ts`): DuckDB with Cloudflare R2 persistence, enhanced startup sequence
2. **DuckDB Server** (`server-duckdb.ts`): Local DuckDB file for development
3. **Demo Server** (`server-demo.ts`): Mock data for quick prototyping

## Enhanced Server Features

- **9-Step Startup Sequence**: Configuration validation → Service initialization → Smart restore → Database connection → Migration checks → Health validation → Route loading → Server start
- **Health Monitoring**: Comprehensive startup health checks with visual indicators
- **Server State Tracking**: Operational metrics including uptime, session duration, and startup counts
- **Smart Backup/Restore**: Automatic timestamp-based restoration with data loss prevention
- **Periodic Backups**: Configurable automatic backups with interval scheduling (disabled by default, manual preferred)
- **Graceful Shutdown**: Clean shutdown with automatic backup creation and state recording

## Database Architecture

### Core Tables

- **User Management**: Multi-tier membership system (Platinum, Diamond, Black Card)
  - Role-based access control (user, admin, super_admin)
  - Default admin accounts for system administration
  - Comprehensive test users for development and testing
  - Financial verification and automatic tier assignment

- **Event Content Management System**: Complete luxury event lifecycle management
  - **Events Table**: Complete event information with approval workflows and membership-based pricing
  - **Venues Table**: Luxury venue database with capacity, amenities, and location data
  - **Event Categories Table**: Pre-configured luxury event types with membership tier targeting
  - **Event Registrations**: Member participation tracking with payment and status management
  - **Event Feedback**: Rating and review system for event quality control
  - **Event Waitlist**: Queue management for popular events

- **Registration System**: Event registration with approval workflows
- **Migration System**: Version-controlled schema changes with rollback capabilities
- **Operational Tracking**: Server state, migration history, and audit trails

## Frontend Architecture

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

## Backend Architecture

- Express.js with TypeScript and ESM modules
- Modular route structure with controllers
- Database abstraction layer supporting multiple databases
- Comprehensive logging with Winston
- File upload handling with Multer and Sharp

## Security Architecture

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

## Deployment Architecture

### Database Development Modes
- **Demo Mode**: `npm run dev:demo` - uses mock data, no database required
- **DuckDB Mode**: `npm run dev:duckdb` - local DuckDB file for development
- **Full Mode**: `npm run dev` - DuckDB with Cloudflare R2 for production

### Environment Separation
- Development: `hesocial-duckdb-dev`
- Production: `hesocial-duckdb`
- R2 bucket separation for media storage
- Environment-specific OAuth callback URLs