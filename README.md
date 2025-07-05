# HeSocial - High-End Social Event Platform

A luxury social event platform designed for affluent individuals aged 45-65, facilitating premium networking events such as private dinners, yacht parties, and art appreciation gatherings.

## Quick Start

```bash
# Install all dependencies
npm run setup

# Start development servers (frontend + backend)
npm run dev

# Build for production
npm run build

# Run tests
npm run test

# Run linting
npm run lint

# Type checking
npm run typecheck
```

## Project Structure

```
hesocial/
├── frontend/          # React + TypeScript frontend
├── backend/           # Node.js + Express backend
├── database/          # Database schemas and migrations
├── docs/              # Documentation
├── CLAUDE.md          # AI assistant guidance
└── package.json       # Root package configuration
```

## Technology Stack

- **Frontend**: React 18, TypeScript, Tailwind CSS, Vite
- **Backend**: Node.js 22.16.0, Express, TypeScript with ESM modules
- **Database**: DuckDB with Cloudflare R2 backup system
- **Backup System**: AWS SDK v3 with smart restore logic and automatic backups
- **Authentication**: ✅ **Production Ready** - JWT + Google OAuth 2.0 with Passport.js
- **Payments**: Stripe integration
- **Infrastructure**: Cloudflare R2 for database persistence and backup management

## Documentation

- **[Development Guide](./CLAUDE.md)** - AI assistant guidance and architecture information
- **[Authentication Implementation](./docs/AUTHENTICATION_IMPLEMENTATION.md)** - Complete authentication system documentation
- **[Default Users Guide](./docs/DEFAULT_USERS.md)** - Admin accounts and test users documentation
- **[Setup Documentation](./docs/setup/)** - Database and service setup guides
- **[Specifications](./docs/specifications/)** - Platform specifications and requirements

## Database & Backup

The platform uses DuckDB as the primary database with Cloudflare R2 for production-ready backup and persistence:

### ✅ **Production-Ready R2 Integration**
- **Automatic Backups**: Created on graceful server shutdown with SIGTERM/SIGINT handling
- **Smart Restore Logic**: Timestamp comparison to prevent data loss during startup
- **Manual Operations**: Full admin API for backup management and restoration
- **Health Monitoring**: Real-time status of database and R2 sync connectivity
- **Environment Separation**: Isolated development and production R2 buckets

### **Admin API Endpoints**
- `POST /api/admin/backup` - Create manual backup to R2
- `GET /api/admin/backups` - List available R2 backups with metadata
- `POST /api/admin/restore` - Restore database from specific R2 backup

### **Authentication & User Management System** ✅ **Production Ready**
Complete authentication system with JWT tokens, Google OAuth 2.0, role-based access control, and full frontend integration:

#### **Authentication Endpoints**
- `POST /api/auth/register` - User registration with automatic membership tier assignment
- `POST /api/auth/login` - Email/password authentication
- `GET /api/auth/google` - Google OAuth 2.0 authentication flow
- `GET /api/auth/profile` - User profile management
- `PUT /api/auth/profile` - Update user profile
- `POST /api/auth/refresh` - JWT token refresh

#### **Admin System**
- **Role-Based Access**: Three-tier system (user, admin, super_admin)
- **Default Admin Accounts**: Ready-to-use administrative accounts
- **Protected Admin APIs**: Secure admin-only endpoints with authentication
- **Admin Endpoints**:
  - `POST /api/admin/backup` - Create manual backup (Admin+)
  - `GET /api/admin/backups` - List available backups (Admin+)
  - `POST /api/admin/restore` - Restore database (Super Admin only)

#### **Default Users**
- **3 Admin Accounts**: System administration (admin@hesocial.com, etc.)
- **5 Test Accounts**: Development and testing (test.platinum@example.com, etc.)
- **8 Sample Users**: Realistic profiles for development

#### **Key Features**
- **Multi-Strategy Authentication**: Email/password and Google OAuth 2.0
- **Complete Registration Flow**: 3-step registration form with real-time validation
- **JWT Token Management**: Secure generation, validation, and refresh
- **Role-Based Security**: Admin and super admin access control
- **Automatic User Management**: Smart user creation for OAuth
- **Membership Tiers**: Automatic assignment (Platinum, Diamond, Black Card)
- **Session Persistence**: Seamless authentication across app reloads
- **Frontend Integration**: Full React component integration with authentication state

### **Health Check Endpoints**
- `GET /api/health/database` - Database connection and query performance
- `GET /api/health/r2-sync` - R2 backup service status and connectivity
- `GET /api/health/full` - Comprehensive system status with all components

## Security

This platform handles sensitive financial and personal data. All development must follow strict security protocols including:
- AES-256 encryption for data at rest
- TLS 1.3 for data in transit
- Multi-factor authentication
- GDPR/CCPA compliance