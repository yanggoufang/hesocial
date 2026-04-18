# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

High-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets) facilitating luxury events (private dinners, yacht parties, art appreciation).

📖 **[Full Project Overview](docs/PROJECT_OVERVIEW.md)** · **[Current Status](docs/systems/DEVELOPMENT_STATUS.md)** · **[Completed Systems](docs/systems/COMPLETED_SYSTEMS.md)**

## Commands

```bash
# Setup & development
npm run setup                  # Install all workspace dependencies
npm run dev                    # Start frontend (5173) + backend (5000) concurrently
npm run dev:frontend           # Frontend only
npm run dev:backend            # Backend only

# Build & quality gates
npm run build                  # Build both workspaces
npm run lint                   # ESLint frontend + backend
npm run lint:fix               # Auto-fix lint issues
npm run typecheck              # TS typecheck both workspaces
npm run test                   # Run all tests (Vitest frontend, Jest backend)
npm run validate:all           # Docs + lint + typecheck + test (pre-commit gate)

# Database
npm run migrate:status         # Show migration state
npm run migrate:up             # Apply pending migrations
npm run migrate:rollback       # Rollback last migration
npm run migrate:create         # Scaffold new migration
npm run seed                   # Seed DuckDB with sample data

# Single test (backend)
cd backend && npm run test -- --testNamePattern="<name>"
# Single test (frontend)
cd frontend && npm run test -- <file-pattern>
```

📖 **[Complete Development Commands](docs/commands/DEVELOPMENT_COMMANDS.md)**

## Architecture

**Monorepo** using npm workspaces: `frontend/`, `backend/`, shared `database/` SQL schemas.

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS. Entry: `frontend/src/main.tsx`. Dev port: **5173**.
- **Backend**: Node.js 22 + Express + TypeScript. Entry: `backend/src/server.ts`. Dev port: **5000**.
- **Database**: **DuckDB** embedded file at repo root (`hesocial.duckdb`) — no external DB server. Schema: `database/duckdb-schema.sql`. Migrations managed via `backend/src/database/MigrationService.ts`.
- **Storage**: Cloudflare R2 for media and DB backups (optional in dev).
- **Auth**: JWT + Google OAuth 2.0.

Backend routes live in `backend/src/routes/` and are registered in `backend/src/server.ts`. Frontend pages in `frontend/src/pages/` are lazy-loaded via React Router.

📖 **[API Reference](docs/api/API_REFERENCE.md)** · **[Architecture Docs](docs/architecture/)**

## Environment Setup

Before first run, create `backend/.env` from the template:

```bash
cp backend/.env.example backend/.env
```

`.env.example` ships with placeholder credentials for JWT, Google/LinkedIn OAuth, Stripe, and Cloudflare R2. The app boots with placeholders but auth/payment/R2 features require real values. See [R2 Configuration](docs/configuration/R2_CONFIGURATION.md).

## Database Modes

- **Full Mode**: `npm run dev` — DuckDB with R2 backup integration
- **DuckDB-only Mode**: `npm run dev:duckdb` — local DuckDB, no R2

📖 **[Database System](docs/database/DATABASE_SYSTEM.md)**

## Test Accounts

- **Admin**: `admin@hesocial.com` / `admin123`
- **Test User**: `test.platinum@example.com` / `test123`
- **Dev Token**: `dev-token-12345` (bypass auth for local API testing)

📖 **[Authentication System](docs/authentication/AUTHENTICATION_SYSTEM.md)**

## Code Style

- **No comments** unless explicitly requested
- TypeScript strict mode
- Follow existing patterns; prefer editing over creating files
- Role-based access control on all protected routes
- Validate inputs at route handlers

## Deployment Reminders

- **Do not start or restart servers automatically** — ask the user to do it.
- Frontend deploys require a build (`npm run build:frontend`) before serving.
- Never commit `.env`, `.credentials.json`, or the `statsig/` directory.

## Documentation Structure

| Path | Purpose |
|------|---------|
| `docs/PROJECT_OVERVIEW.md` | High-level project context |
| `docs/systems/` | Per-system documentation & status |
| `docs/commands/` | Development commands reference |
| `docs/authentication/` | Auth system details |
| `docs/database/` | Database, migrations, schema |
| `docs/configuration/` | Setup guides (R2, env) |
| `docs/api/` | API endpoint reference |
| `docs/architecture/` | System architecture diagrams |
| `docs/development/` | Development workflows |
