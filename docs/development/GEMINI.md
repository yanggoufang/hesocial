# Project Guidance

This file provides guidance to AI assistants when working with code in this repository.

## Project Overview

This is a high-end social event platform targeting affluent individuals aged 45-65 with high net worth (NT$5M+ annual income or NT$30M+ net assets). The platform facilitates luxury social events like private dinners, yacht parties, and art appreciation gatherings.

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
- **Runtime**: Node.js with Express framework
- **Language**: TypeScript with ESM modules
- **Database**: DuckDB as primary database with Cloudflare R2 persistence
- **Authentication**: JWT with Passport.js (Google OAuth, LinkedIn OAuth)
- **Security**: Helmet, CORS, rate limiting, compression
- **Logging**: Winston with Morgan middleware
- **File Processing**: Multer with Sharp for image processing
- **Payments**: Stripe integration

### Database Options
- **Primary**: DuckDB with schema defined in `database/duckdb-schema.sql`
- **Production Persistence**: Cloudflare R2 for DuckDB database file storage
- **Development**: Local DuckDB file for development
- **Key Tables**: Users, Events, Registrations, Financial verification

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
npm run migrate      # Run database migrations
npm run seed         # Seed development data
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
npm run migrate      # Run database migrations
npm run seed         # Seed database with sample data
```

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
│   │   ├── middleware/    # Express middleware
│   │   ├── routes/        # API route definitions
│   │   ├── types/         # Shared TypeScript types
│   │   └── utils/         # Configuration and logging
│   ├── server.ts          # Main PostgreSQL server
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
1. **Production Server** (`server.ts`): DuckDB with Cloudflare R2 persistence
2. **DuckDB Server** (`server-duckdb.ts`): Local DuckDB file for development
3. **Demo Server** (`server-demo.ts`): Mock data for quick prototyping

### Database Architecture
- **User Management**: Multi-tier membership system (Platinum, Diamond, Black Card)
- **Event System**: Premium events with pricing, exclusivity levels, and venue management
- **Registration System**: Event registration with approval workflows
- **Financial Verification**: Required for platform access

### Authentication & Security
- JWT-based authentication with refresh tokens
- OAuth 2.0 integration (Google, LinkedIn)
- Rate limiting and request validation
- Helmet.js for security headers
- CORS configuration for cross-origin requests

## Development Workflow

### Getting Started
1. **Initial Setup**: `npm run setup` - installs all dependencies
2. **Development**: `npm run dev` - starts both frontend (port 3000) and backend (port 5000)
3. **Database Setup**: DuckDB runs automatically - see docs/setup/ for R2 configuration

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