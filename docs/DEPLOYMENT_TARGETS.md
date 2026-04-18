# Deployment Targets

This document defines the current production hosting model for HeSocial. It is the source of truth for distinguishing application hosting from Cloudflare infrastructure services.

## Current Production Targets

The committed production deployment configuration is `render.yaml`.

| Component | Production target | Evidence |
| --- | --- | --- |
| Frontend | Render static site, `hesocial-frontend` | `render.yaml` static service with `staticPublishPath: ./frontend/dist` |
| Backend API | Render Node web service, `hesocial-api` | `render.yaml` web service with `runtime: node` and `startCommand: cd backend && npm start` |
| Database runtime | Local DuckDB file used by the backend process | `backend/src/database/duckdb-connection.ts` |
| Durable DB backup/persistence | Cloudflare R2 | `R2BackupService.ts` and R2 configuration docs |
| Media storage | Cloudflare R2 | `MediaService.ts` |
| Domain/DNS | Cloudflare may be used externally, but no DNS config is committed | No repo-local Cloudflare DNS config |

## What Cloudflare Means In This Repo

Cloudflare currently means **R2 storage and possibly external DNS/domain routing**. It does not currently mean that the frontend or backend runtime is deployed to Cloudflare.

The repo does not contain:

- `wrangler.toml`
- Cloudflare Worker entrypoint
- Cloudflare Pages configuration
- Pages Functions directory
- D1 database configuration
- KV, Durable Objects, Queues, or Hyperdrive bindings

## Local Testing Model

Local runtime testing should match the current production shape:

- Frontend: Vite dev server or built static assets from `frontend/dist`
- Backend: Node/Express server from `backend/src/server.ts` or `backend/dist/server.js`
- Database: local `hesocial.duckdb`
- R2: disabled unless real R2 environment variables are provided

Do not describe local testing as Cloudflare Worker or Pages simulation unless Cloudflare runtime config is added to the repo.

## If We Move Runtime To Cloudflare Later

A Cloudflare runtime migration is a separate architecture project. It should add a new design and implementation plan before code changes begin. At minimum, it needs:

- `wrangler.toml`
- a Worker or Pages Functions entrypoint
- a replacement for Express middleware patterns or an adapter/router such as Hono
- a database decision, likely D1 or an external database, because the current backend depends on a local DuckDB file
- updated local simulation commands using `wrangler dev` or `wrangler pages dev`
- updated production deployment instructions

Until those files and commands exist, `render.yaml` remains the production hosting source of truth.
