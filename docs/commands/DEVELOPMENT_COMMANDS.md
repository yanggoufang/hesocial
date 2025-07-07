# Development Commands

## Root Level Commands

### Project Setup
```bash
# Setup project (install all dependencies)
npm run setup

# Start both frontend and backend
npm run dev

# Build entire project
npm run build
```

### Testing and Quality
```bash
# Run all tests
npm run test

# Lint all code
npm run lint

# Type check all code
npm run typecheck
```

### Database Operations
```bash
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

## Frontend Commands (cd frontend/)

### Development
```bash
npm run dev          # Start Vite dev server
npm run build        # Build for production
npm run preview      # Preview production build
```

### Quality Control
```bash
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
npm run test:coverage # Run tests with coverage
```

## Backend Commands (cd backend/)

### Development Servers
```bash
npm run dev          # Start with DuckDB (tsx watch)
npm run dev:demo     # Start demo server with mock data
npm run dev:duckdb   # Start with DuckDB embedded database
```

### Production
```bash
npm run build        # Compile TypeScript
npm run start        # Start production server
```

### Database Management
```bash
npm run seed         # Seed database with sample data

# Migration management (same as root commands)
npm run migrate:status    # Show migration status
npm run migrate:up        # Apply pending migrations
npm run migrate:rollback  # Rollback to version
npm run migrate:create    # Generate migration template
npm run migrate:validate  # Validate migration integrity
```

### Quality Control
```bash
npm run lint         # ESLint with auto-fix
npm run typecheck    # TypeScript type checking
npm run test         # Run Vitest tests
```

## Development Workflow

### Getting Started
1. **Initial Setup**: `npm run setup` - installs all dependencies
2. **Development**: `npm run dev` - starts both frontend (port 3000) and backend (port 5000)
3. **Database Setup**: See `docs/setup/` for database configuration options

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