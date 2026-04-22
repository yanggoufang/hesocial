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
npm run test                   # Run all tests (Vitest, frontend + backend)
npm run validate:all           # Docs + lint + typecheck + test (also the husky pre-commit gate)

# Database
npm run migrate:status         # Show migration state
npm run migrate:up             # Apply pending migrations
npm run migrate:rollback       # Rollback last migration
npm run migrate:create         # Scaffold new migration
npm run seed                   # Seed DuckDB with sample data

# Single test — Vitest in both workspaces: use `-t` for test-name, positional for file
cd backend  && npm run test -- -t "<name>"
cd frontend && npm run test -- <file-pattern>
```

📖 **[Complete Development Commands](docs/commands/DEVELOPMENT_COMMANDS.md)**

## Architecture

**Monorepo** using npm workspaces: `frontend/`, `backend/`, shared `database/` SQL schemas.

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS. Entry: `frontend/src/main.tsx`. Dev port: **5173**.
- **Backend**: Node.js 22 + Express + TypeScript, **ESM** (`"type": "module"`). Entry: `backend/src/server.ts`. Dev port: **5000**.
- **Database**: **DuckDB** embedded file at repo root (`hesocial.duckdb`) — no external DB server. Schema: `database/duckdb-schema.sql`. Migrations managed via `backend/src/database/MigrationService.ts` (TS migrations live in `backend/src/database/migrations/*.migration.ts`).
- **Storage**: Cloudflare R2 for media and DB backups (optional in dev).
- **Hosting target**: Render (`render.yaml`) hosts the frontend static site and backend Node service. Cloudflare is not the app runtime in the committed config.
- **Auth**: JWT + Google OAuth 2.0.

### Backend wiring gotchas
- ESM imports must use `.js` extensions on compiled TS files (e.g. `import ... from './routes/main.js'`) even in source — this is how the compiled output resolves.
- Routes are composed in `backend/src/routes/index.ts` and loaded via an async dynamic import in `backend/src/routes/main.ts`; add new route modules there, not in `server.ts`.
- `server.ts` installs a `BigInt.prototype.toJSON` monkeypatch so DuckDB `BIGINT` columns serialize to JSON — don't remove it, and prefer numbers over bigints in response shapes when safe.
- Rate limiting: admin/event-management paths get a permissive limiter mounted *before* the general `/api` limiter in `server.ts`; mount order matters.

Frontend pages in `frontend/src/pages/` are lazy-loaded via React Router; route guards live in `frontend/src/components/RouteGuards.tsx` / `ProtectedRoute.tsx`.

📖 **[API Reference](docs/api/API_REFERENCE.md)** · **[Architecture Docs](docs/architecture/)**

## Environment Setup

Before first run, create `backend/.env` from the template:

```bash
cp backend/.env.example backend/.env
```

`.env.example` ships with placeholder credentials for JWT, Google/LinkedIn OAuth, Stripe, and Cloudflare R2. The app boots with placeholders but auth/payment/R2 features require real values. See [R2 Configuration](docs/configuration/R2_CONFIGURATION.md).

## Database Modes

There is only one dev entrypoint: `npm run dev`. It always uses the local DuckDB file; R2 backup/restore activates only when R2 env vars are set in `backend/.env`. (Older docs mention `npm run dev:duckdb` — that script does not exist; see `docs/COMMAND_DISCREPANCIES.md`.)

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

## Pre-commit

A Husky `pre-commit` hook runs `validate:docs`, `lint:fix`, `typecheck`, and `npm run test -- --run` in that order. `HUSKY_SKIP_TESTS=true` skips only the test step; the others are hard gates. If `validate:docs` fails, fix the docs — don't bypass the hook.

## Deployment Reminders

- **Do not start or restart servers automatically** — ask the user to do it.
- Frontend deploys require a build (`npm run build:frontend`) before serving.
- Production hosting source of truth is `render.yaml`; see [Deployment Targets](docs/DEPLOYMENT_TARGETS.md).
- Cloudflare references in this repo mean R2 storage/backups/media unless a future Worker/Pages config is added.
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
