# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

High-end social event platform for affluent individuals (NT$5M+ income, NT$30M+ assets) facilitating luxury events (private dinners, yacht parties, art appreciation).

📖 **[Full Project Overview](docs/PROJECT_OVERVIEW.md)** · **[Current Status](docs/systems/DEVELOPMENT_STATUS.md)** · **[Completed Systems](docs/systems/COMPLETED_SYSTEMS.md)**

## Commands

```bash
# Setup & development
npm run setup                  # Install all workspace dependencies
npm run dev                    # Start the React SPA (5173)
npm run dev:frontend           # same thing

# Build & quality gates
npm run build                  # Build the React SPA
npm run build:worker           # Build the Rust Worker (wasm32)
npm run build:web-rust         # Build the Rust SPA as two bundles + the merged tree
npm run lint                   # ESLint the React SPA
npm run lint:rust              # clippy over backend-rust
npm run typecheck              # TS typecheck the React SPA
npm run test                   # Pre-commit gate: React SPA (no test files) + cargo test -p core
npm run test:rust              # backend-rust: cargo test -p core
npm run test:web-rust          # frontend-rust: cargo test
npm run test:contract:rust     # the Rust Worker under workerd against a local sqld
npm run validate:all           # Docs + lint + typecheck + test (also the husky pre-commit gate)
# NOTE: `frontend/` has no tests. The real suites need a build first and are NOT
# in the gate: test:web-rust (293 logic + 22 e2e) and test:contract:rust (49).

# Single test
cd frontend      && npm run test -- <file-pattern>
cd frontend-rust && cargo test --test <suite> <test-name>
cd backend-rust  && cargo test -p core <test-name>
```

📖 **[Complete Development Commands](docs/commands/DEVELOPMENT_COMMANDS.md)**

## Architecture

**Monorepo** using npm workspaces: `frontend/` (React SPA) and `contract/` (Worker contract tests). The Rust crates `backend-rust/` and `frontend-rust/` are cargo, not npm.

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS. Entry: `frontend/src/main.tsx`. Dev port: **5173**. Being replaced by `frontend-rust/` (Dioxus, 22/22 routes ported, not yet deployed).
- **Backend**: Rust (`workers-rs` + axum) compiled to wasm32, in `backend-rust/crates/worker`. Domain logic that can be tested on the host lives in `backend-rust/crates/core`.
- **Database**: **Turso / libSQL** over Hrana HTTP (`backend-rust/crates/worker/src/db.rs`). Schema and seed are plain SQL in `backend-rust/sql/`; there is no migration runner.
- **Archived**: the Node/Express API and its DuckDB database are in `archive/backend/` and are not built, tested, or deployed. See `archive/README.md`.
- **Storage**: Cloudflare R2 for media and DB backups (optional in dev).
- **Hosting target**: a single Cloudflare Worker (`backend-rust/wrangler.toml`) serves the Rust API and the React SPA from one origin at `hesocial.ahexagram.com`. Render is decommissioned.
- **Auth**: JWT + Google OAuth 2.0.

### Backend wiring gotchas
- Routes are composed in `router()` in `backend-rust/crates/worker/src/lib.rs`; handlers live in the `*_handlers.rs` modules beside it.
- `crates/worker` only compiles for wasm32. Anything worth unit-testing belongs in `crates/core`, which `cargo test -p core` covers.
- The contract suite in `contract/` runs the real Worker under workerd against a local `turso dev`. It refuses to run against a stale build, so run `cd backend-rust && npx wrangler deploy --dry-run` first.

Frontend pages in `frontend/src/pages/` are lazy-loaded via React Router; route guards live in `frontend/src/components/RouteGuards.tsx` / `ProtectedRoute.tsx`.

📖 **[API Reference](docs/api/API_REFERENCE.md)** · **[Architecture Docs](docs/architecture/)**

## Environment Setup

Worker configuration lives in `backend-rust/wrangler.toml`; secrets are set with `npx wrangler --cwd backend-rust secret put <NAME>` and are listed in [Deployment Targets](docs/DEPLOYMENT_TARGETS.md). Nothing in the live stack reads a `.env` file.

## Database

Turso/libSQL, reached over Hrana HTTP. `TURSO_URL` is a var in `wrangler.toml`; `TURSO_AUTH_TOKEN` is a secret. For local work the contract tests stand up `~/.turso/turso dev --port 8481` and apply `backend-rust/sql/schema.sql` and `seed.sql` themselves.

📖 **[Database System](docs/database/DATABASE_SYSTEM.md)**

## Test Accounts

Seed accounts come from `backend-rust/sql/seed.sql`, applied when a database is provisioned. A fresh database has no users, and no working login, until it is applied.

- **Admin**: `admin@hesocial.com` / `admin123`
- **Test User**: `test.platinum@example.com` / `test123`

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
- Production hosting source of truth is `backend-rust/wrangler.toml`; see [Deployment Targets](docs/DEPLOYMENT_TARGETS.md). Deploying needs `VITE_API_URL=/api npm run build:frontend` first — `frontend/dist` is gitignored.
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
