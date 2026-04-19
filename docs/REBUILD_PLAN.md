# HeSocial Rebuild Plan - Turso/libSQL Direction

**Date**: 2026-04-19
**Decision**: Move the production transactional database off DuckDB and rebuild the backend around Turso/libSQL.
**Status**: Planned follow-up after the current Render production hotfix is deployed.

## Why Rebuild The Data Layer

DuckDB is a strong analytical database, but it is a poor fit as the primary transactional database for this app on Render. HeSocial's production workload is mostly users, auth, roles, events, registrations, visitor tracking, and admin CRUD. Those are ordinary web-application transactions, not local analytical scans.

The current DuckDB setup has already exposed operational risk:

- Local file and WAL state can prevent the Node process from opening the database.
- Render persistence depends on deployment/runtime disk behavior and backup restore timing.
- R2 database backup/restore is custom application logic rather than a managed persistence layer.
- Schema drift, such as missing `users.role`, can break core auth endpoints at runtime.
- Concurrent request writes require careful handling around IDs, file locks, and migrations.

Turso/libSQL is a better fit for the production app because it keeps the SQLite programming model while moving persistence, replication, and operational ownership outside the Render container.

## Target Stack

| Layer | Choice |
|-------|--------|
| Frontend | Keep current Vite + React + TypeScript + Tailwind |
| API | Keep Express temporarily, then optionally migrate to Hono/Workers later |
| Primary database | **Turso/libSQL** |
| Object storage | Cloudflare R2 for media assets |
| Backups | Turso platform backups/export plus optional R2 logical exports |
| Auth | Existing JWT/email-password flow, then Google OAuth once env vars/routes are verified |
| Hosting | Short term: Render. Later: Cloudflare Pages/Workers only if it reduces ops burden |

## Strategic Choice

This is a **database-first rebuild**, not a full product rewrite.

Do not rebuild the frontend, routing, or product surface just to change the database. The immediate target is:

1. Keep current React frontend.
2. Keep current API route contracts.
3. Replace DuckDB adapters and schema with Turso/libSQL.
4. Move R2 back to its correct role: media storage and optional exports, not primary database recovery.

## Carry Forward

- Current React pages/components and Tailwind theme.
- API route shape where practical, especially `/api/auth/*`, `/api/events`, `/api/categories`, `/api/venues`, `/api/registrations/*`.
- Existing bcrypt password hashes if compatibility is verified.
- Existing role model: `user`, `admin`, `super_admin`.
- Existing R2 media bucket and media references.

## Replace

- `backend/src/database/duckdb-connection.ts`
- `backend/src/database/duckdb-pool.ts`
- DuckDB migration scripts and file-based restore assumptions.
- Runtime DB backup/restore dependency on R2.
- Any `MAX(id) + 1` ID generation still present after the hotfix.

## Proposed Phases

### P0 - Ship Current Production Hotfix

- Push the current Render/API base URL/router/OAuth/visitor tracking fix.
- Verify Render dashboard `VITE_API_URL`.
- Verify production `/api/test`, `/api/auth/google`, and `/api/auth/login`.
- Do not start the Turso migration until the current production API is no longer returning route-level 404s.

### P1 - Turso Project And Adapter

- Create Turso database.
- Add Render env vars:
  - `TURSO_DATABASE_URL`
  - `TURSO_AUTH_TOKEN`
- Add a small database adapter with the same query shape currently used by controllers.
- Keep the old DuckDB adapter available only for one-time export scripts.

### P2 - Schema Translation

- Translate the current DuckDB schema to SQLite/libSQL-compatible SQL.
- Prefer explicit text IDs where existing app code already uses UUIDs.
- Avoid sequence-style integer IDs unless the table truly needs numeric ordering.
- Add migrations for:
  - users and roles
  - events, categories, venues
  - registrations
  - media metadata
  - visitor sessions/page views/events

### P3 - Data Export And Import

- Export DuckDB tables to CSV or SQL.
- Import into Turso.
- Verify row counts table-by-table.
- Spot-check:
  - one admin user
  - one normal user
  - one event
  - one venue
  - one registration
- Verify password hash compatibility before declaring auth migrated.

### P4 - Runtime Switch

- Point backend at Turso adapter in staging.
- Run route smoke tests:
  - `GET /api/health`
  - `GET /api/events`
  - `GET /api/categories`
  - `GET /api/venues`
  - `POST /api/auth/login`
  - `GET /api/auth/profile`
- Deploy to production only after admin login and user login are confirmed.

### P5 - Simplify R2 Backup Role

- Keep R2 for media.
- Remove automatic database restore from normal startup.
- Keep optional logical database export to R2 if useful, but do not depend on it for every boot.

## Unresolved Issues

These are known issues that should stay open until verified in production or fixed in the Turso migration.

1. **Production admin login is unverified.**
   Local smoke no longer crashes on `/api/auth/login`, but `admin@hesocial.com / admin123` returned `401` locally. This may be local seed drift. Production must be tested after deploy.

2. **Render dashboard env overrides are unverified.**
   `render.yaml` now sets `VITE_API_URL=https://hesocial-api.onrender.com/api`, and frontend normalization tolerates either host or host-plus-`/api`. A stale dashboard override could still point elsewhere.

3. **Google OAuth is intentionally disabled unless credentials exist.**
   `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` must be set manually in Render. OAuth should remain a follow-up until callback URLs and frontend route behavior are tested end-to-end.

4. **Current local DuckDB file appears stale/fragile.**
   Direct DuckDB open against `hesocial.duckdb` failed after local watcher restarts, while a copied DB at `/tmp/hesocial-smoke.duckdb` opened successfully. This reinforces the Turso decision.

5. **R2 restore exists but has not been used in this fix.**
   The code can list/download/restore R2 backups when R2 env vars are configured, but the current hotfix did not pull a backup from R2.

6. **Visitor tracking IDs are now sequence-backed at startup, but this is transitional.**
   The Turso schema should use safer ID defaults instead of recreating DuckDB sequences at boot.

## Current Hotfix Relationship

The current hotfix is still worth deploying before the rebuild because it:

- stops silent API route mount failures;
- prevents missing Google OAuth credentials from killing the router;
- fixes production API base URL construction;
- removes frontend raw `/api/...` calls that would hit the static frontend host;
- repairs schema drift for `users.role`;
- stabilizes visitor tracking enough for the current Render app.

The Turso rebuild should start only after this is deployed and production behavior is measured.
