# Development Workflow

## Getting Started

### Initial Setup
1. **Initial Setup**: `npm run setup` - installs all dependencies
2. **Development**: `npm run dev` - starts both frontend (port 3000) and backend (port 5000)
3. **Database Setup**: See `docs/setup/` for database configuration options

### Development Commands

#### Root Level Commands
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

#### Frontend Commands (cd frontend/)
```bash
npm run dev          # Start Vite dev server
npm run build        # Build for production
npm run preview      # Preview production build
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run test:coverage # Run tests with coverage
```

#### Backend Commands (cd backend/)
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

## Testing Commands

- `npm run test` - run all tests (frontend + backend)
- `npm run test:frontend` - frontend tests only
- `npm run test:backend` - backend tests only
- Individual test suites use Vitest

## Code Quality Standards

### Code Quality Commands
- `npm run lint` - ESLint for both frontend and backend
- `npm run typecheck` - TypeScript compilation check
- Strict TypeScript configuration with no implicit any

### Development Guidelines

#### Code Quality Requirements
- **NEVER** add comments unless explicitly requested
- Follow existing code conventions and patterns
- Check for existing libraries before adding new dependencies
- Run `npm run lint` and `npm run typecheck` before completion
- **NEVER** commit changes unless explicitly requested

#### Security Requirements
- Only defensive security tasks allowed
- Never expose or log secrets/keys
- Follow role-based access control patterns
- Use existing authentication middleware

## Development Environment Options

### Database Development Modes
- **Demo Mode**: `npm run dev:demo` - uses mock data, no database required
- **DuckDB Mode**: `npm run dev:duckdb` - local DuckDB file for development
- **Full Mode**: `npm run dev` - DuckDB with Cloudflare R2 for production

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

## Environment Configuration

### Required Environment Variables

#### R2 Storage Configuration
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

#### OAuth Configuration
```bash
# Google OAuth Setup (Development)
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# OAuth Callback URLs
# Development: http://localhost:5000/api/auth/google/callback
# Production: https://yourdomain.com/api/auth/google/callback
```

## Task Management Workflow

### Using Todo System
- Use TodoWrite/TodoRead tools frequently for complex tasks
- Mark todos as completed immediately after finishing
- Reference implementation status in documentation before starting work
- Search for existing patterns and conventions in codebase

### Complex Task Workflow
1. **Plan**: Use TodoWrite to break down complex tasks into steps
2. **Research**: Use Agent tool for searching/understanding codebase
3. **Implement**: Follow established patterns and conventions
4. **Validate**: Run lint, typecheck, and tests
5. **Complete**: Mark todos as completed and document any changes

## Implementation Approach

### Enterprise-Grade Practices
Based on production-ready patterns, this implementation follows enterprise-grade practices:
- **Comprehensive Health Checks**: Visual status indicators and detailed component validation
- **Smart Operational Logic**: Intelligent decision-making (e.g., smart restore, auto-migration)
- **Operational Visibility**: Server state tracking and detailed logging with structured data
- **Production Safety**: Configuration validation, error handling, and graceful degradation
- **Developer Experience**: CLI tools, comprehensive documentation, and clear patterns

### Following Conventions
When making changes to files:
1. First understand the file's code conventions
2. Mimic code style, use existing libraries and utilities
3. Follow existing patterns
4. **NEVER** assume that a given library is available - check package.json first
5. When creating new components, look at existing components for patterns
6. When editing code, look at surrounding context (especially imports)
7. Always follow security best practices
8. Never introduce code that exposes or logs secrets and keys

## Debugging and Troubleshooting

### Common Issues
1. **DuckDB Connection Issues**: Use temporary server for development continuity
2. **R2 Storage Problems**: Check environment variables and connectivity
3. **Authentication Issues**: Verify OAuth configuration and JWT settings
4. **Migration Problems**: Use migration CLI tools for diagnosis and rollback

### Development Tools
- **Health Monitoring**: `/api/health` endpoints for system status
- **Migration CLI**: Comprehensive migration management tools
- **Backup System**: Manual backup/restore capabilities
- **Logging**: Winston logging with structured data