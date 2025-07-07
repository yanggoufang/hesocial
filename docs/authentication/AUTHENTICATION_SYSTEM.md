# Authentication System

## Authentication Framework (Production Ready)

A comprehensive authentication system implementing enterprise-grade security patterns with JWT tokens and OAuth 2.0 integration.

## Core Features

- **Multi-Strategy Authentication**: Email/password and Google OAuth 2.0 support
- **JWT Token Management**: Secure token generation, validation, and refresh
- **Automatic User Management**: Smart user creation and profile matching for OAuth
- **Membership Tier Assignment**: Automatic tier calculation based on financial verification
- **Session Persistence**: Seamless user session management across app reloads
- **Security Middleware**: Route protection and role-based access control

## Frontend Integration

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

## Backend Implementation

### Authentication Routes
```typescript
POST /api/auth/register    // User registration
POST /api/auth/login       // Email/password login
GET  /api/auth/google      // Google OAuth initiation
GET  /api/auth/google/callback // OAuth callback
GET  /api/auth/profile     // Get user profile
PUT  /api/auth/profile     // Update profile
POST /api/auth/refresh     // Token refresh
```

### OAuth Configuration
```bash
# Google OAuth Setup (Development)
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# OAuth Callback URLs
# Development: http://localhost:5000/api/auth/google/callback
# Production: https://yourdomain.com/api/auth/google/callback
```

## Security Features

- **Password Hashing**: bcryptjs with salt rounds for secure password storage
- **JWT Security**: Configurable expiration times and refresh token mechanism
- **OAuth Security**: Secure callback handling with state validation
- **Route Protection**: Authentication middleware for protected endpoints
- **User Verification**: Multi-tier verification system for financial validation

## User Management

### Registration Flow
Complete user onboarding with financial verification

### Profile Management
Comprehensive user profile CRUD operations

### Role-Based Access Control
Three-tier admin system:
- `user`: Standard members with tier-based access
- `admin`: Event management and member operations
- `super_admin`: Full system administration capabilities

### Membership Tiers
Automatic assignment based on income/net worth:
- **Platinum**: Default tier for new users
- **Diamond**: 30M+ NTD net worth or 5M+ annual income
- **Black Card**: 100M+ NTD net worth or 20M+ annual income

### Default Accounts
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

## Role-Based Access Control

```typescript
// Middleware protection levels
requireAdmin()      // Requires 'admin' or 'super_admin' role
requireSuperAdmin() // Requires 'super_admin' role only

// API endpoint protection
POST /api/admin/backup     // Admin+ required
POST /api/admin/restore    // Super Admin only
GET  /api/admin/backups    // Admin+ required
```

## Security Implementation

- **Password Hashing**: bcrypt with 12 salt rounds for all accounts
- **Role Validation**: Comprehensive middleware for admin route protection
- **Default Security**: Production-ready admin accounts with proper access control
- **Test Coverage**: Complete user scenarios for development and testing

## Related Documentation
- [Default Users](../DEFAULT_USERS.md)
- [Authentication Implementation](../AUTHENTICATION_IMPLEMENTATION.md)
- [API Reference](../api/API_REFERENCE.md)