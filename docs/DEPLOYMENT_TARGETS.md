# Deployment Targets

This document defines the current production hosting model for HeSocial. It is
the source of truth for where each piece actually runs.

## Current Production Targets

Production is a single Cloudflare Worker serving both the API and the SPA from
one origin. Its committed configuration is `backend-rust/wrangler.toml`.

| Component | Production target | Evidence |
| --- | --- | --- |
| Site + API | Cloudflare Worker `hesocial-backend-rust` at `https://hesocial.ahexagram.com` | `backend-rust/wrangler.toml` — `routes` custom domains and `[assets]` |
| API implementation | Rust (`workers-rs` + axum), compiled to wasm32 | `backend-rust/crates/worker` |
| Frontend | React SPA served as Worker static assets (the Rust SPA is built but not yet flipped on — see below) | `[assets]` with `not_found_handling = "single-page-application"` |
| Database | Turso / libSQL over Hrana HTTP v2 | `backend-rust/crates/worker/src/db.rs`, `TURSO_URL` + `TURSO_AUTH_TOKEN` |
| Media storage | Cloudflare R2, bucket `hesocial-media` | `MEDIA` binding |
| Rate limiting | Cloudflare Rate Limit binding | `RATE_LIMITER` binding |
| Domain/DNS | Cloudflare zone `ahexagram.com` | Workers Custom Domains, provisioned by `wrangler deploy` |

`hesocial-api.ahexagram.com` stays routed to the same Worker so anything
already pointed at the API-only hostname keeps working.

## Request Routing

Static assets are matched before the Worker runs. `/api/*` is exempted via
`run_worker_first`, so API paths always reach the Worker — without that, the
SPA fallback would answer an unknown API path with `index.html` instead of the
JSON error clients expect.

## Rust SPA: two bundles (not yet wired to the Worker)

`frontend-rust/` is the 22-route Rust/Dioxus SPA that replaces the React one.
**Nothing in `wrangler.toml` or `package.json` points at it yet** — the Worker
still serves `frontend/dist`. Wiring it is a deliberate, separate cutover.

It builds as **two** wasm bundles, because wasm-split is unusable
(see `docs/rust-migration/ROADMAP.md`) and one bundle is 765,738 bytes gzipped:

```bash
npm run build:web-rust     # scripts/build-web-rust.sh
```

| Output | Routes | wasm gzip |
| --- | --- | --- |
| `frontend-rust/dist` | public + member (`/`, `/events*`, `/vvip`, `/profile*`, …) | 477,723 |
| `frontend-rust/dist-admin` | `/admin*`, `/event-mgmt*` | 613,400 |

Membership is the `admin-bundle` cargo feature gating the `Route` enum in
`frontend-rust/src/ui.rs`. Both bundles carry `/`, `/login` and `/profile` so a
cross-bundle jump always lands somewhere real, and `Shell`'s navigation falls
back to a full page load (`shell::hard_navigate`) when a path does not parse in
the running bundle — that is how a member clicking 管理後台 crosses over.

`build:web-rust` also merges the two into `frontend-rust/dist-worker`, which is
the tree wrangler uploads. The merge is safe because dx hashes every asset
filename, so the bundles' JS and wasm never collide and their tailwind CSS is
byte-identical:

```
dist-worker/index.html                      public entry (Cloudflare's SPA fallback)
dist-worker/admin.html                      admin entry
dist-worker/assets/…-<publichash>.js|wasm
dist-worker/assets/…-<adminhash>.js|wasm
dist-worker/assets/tailwind-….css           shared
```

One `[assets]` directory cannot route two entry points, so the Worker does it:
`run_worker_first` covers `/admin`, `/admin/*`, `/event-mgmt` and
`/event-mgmt/*`, and the fetch handler answers those with `/admin.html` from
the `ASSETS` binding. The prefix test lives in `hesocial_core::spa` so it is
covered by `cargo test -p core` rather than only in wasm — `/administrators`
and `/event-mgmt-archive` must not match, and they are asserted not to.

**The cutover itself is one line** — `directory = "../frontend-rust/dist-worker"`
— and is deliberately not taken yet. The Rust SPA has only ever been served by
the e2e harness, and a backend-only deploy must not silently replace the whole
frontend. Until it is flipped, the admin prefixes reach the Worker, `/admin.html`
misses in the React tree, and the handler falls through, so the behaviour is
unchanged.

After flipping, verify both prefixes. Serving the wrong bundle for a path fails
quietly rather than loudly: the router renders its `RouteMatchError` page *and*
rewrites the address bar to `/`, so the URL and the content disagree.

## Deploy

```bash
VITE_API_URL=/api npm run build:frontend    # frontend/dist is gitignored
npx wrangler --cwd backend-rust deploy      # builds the wasm and uploads both
```

`VITE_API_URL=/api` makes the bundle call its own origin. Rolling back is
`npx wrangler --cwd backend-rust rollback`, or removing the custom domain.

## Secrets

Set with `npx wrangler --cwd backend-rust secret put <NAME>`:

| Secret | Required | Notes |
| --- | --- | --- |
| `JWT_SECRET` | yes | Token-issuing endpoints 500 fail-closed without it. Read per request, so rotating needs no code change or redeploy. |
| `TURSO_AUTH_TOKEN` | yes | Per-database, non-expiring, from `turso db tokens create hesocial`. A group token would cover every database in the group and expire in a day, and a Workers isolate cannot refresh one. Rotate with `turso db tokens invalidate hesocial`. |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | for Google login | Currently unset. The callback is derived from the request origin, so register `https://hesocial.ahexagram.com/api/auth/google/callback`. |

## Local Testing Model

- Contract tests run the real wasm Worker under workerd against a local
  `turso dev`: `npm run test:contract:rust` (see `backend/vitest.workers.config.ts`).
- The legacy Node/Express stack in `backend/` still runs on DuckDB for the
  Express side of the dual-target contract suite.

## Decommissioned

Render (`render.yaml`) hosted the Express API and the static frontend. Both are
retired: Render's free disk is ephemeral, the DuckDB file did not survive a
restart, and no usable backup existed in R2 to restore from, so every data
endpoint on `hesocial-api.onrender.com` returned 500. There was no production
data to migrate.
