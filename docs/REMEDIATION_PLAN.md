# Remediation Plan — Codex Code Review Findings

**Date**: 2026-04-18
**Source**: External review (Codex), cross-validated against code, reviewed by Gemini (v1), corroborated by second Codex pass (v2)
**Scope**: 8 Codex findings + 1 mock-data issue discovered during validation
**Branch**: `fix/codex-remediation` (create fresh, phase-by-phase commits)

---

## Findings Summary

| ID | Severity | Summary |
|----|----------|---------|
| F1 | Critical | Hardcoded admin login in `server.ts`, not env-gated |
| F2 | Critical | Unauthenticated `/api/debug/*` endpoints (incl. `DELETE FROM events`) |
| F3 | High | `npm run typecheck` fails on both workspaces |
| F4 | High | Frontend `POST /registrations/events/:eventId` hits non-existent BE route |
| F5 | High | Token key mismatch: `hesocial_token` (auth) vs `token` (4 other services) |
| F6 | Medium | admin/users/events/venues/categories/media routers commented out |
| F7 | Medium | Catch-all shadows duplicated debug block (lines 385-672) |
| F8 | Medium | Error handler registered before route mounting |
| F9 | Medium | `SalesManagement.tsx` mock data bypasses working backend |
| F10 | High | `BlueGreenDatabaseManager` hardcoded absolute dev paths break on deploy |

---

## Execution Order

**Phase 0 (preflight) → A (security) → B1-B3 (tokens, registration, error order) → D1 (backend types) → B4 (SalesManagement) → C (re-enable routers, one at a time) → D2 (frontend icons) → D3 (final verify)**

Rationale: D1 before B4 so SalesManagement rewrite can be smoke-tested against a type-clean backend. D1 before C so shared export/type issues aren't fought across 6 routers.

---

## Phase 0 — Preflight

**Goal**: Baseline current state before any change.

```bash
git checkout -b fix/codex-remediation
git status --short                              # must be clean
npm ls --workspaces --depth=0                   # verify declared deps are installed
npm run typecheck:backend 2>&1 | tee baseline-backend-typecheck.txt
npm run typecheck:frontend 2>&1 | tee baseline-frontend-typecheck.txt
```

Inspect baselines — understand the current error set before fixing. These files are local-only, not committed.

---

## Phase A — Security Critical

**Goal**: Remove auth bypass and destructive unauthenticated endpoints.
**Effort**: 30–60 min.

### A0. Admin login pre-check (new, per corroborator)

Before deleting the hardcoded auth, verify real admin login will work:

```bash
# Confirm the real bcrypt hash is seeded (not the fake '$2b$10$hash' from /debug/manual-seed)
# The real hash lives in database/duckdb-admin-users.sql.
# If a debug seed was run, it will have overwritten the hash.

# Query DuckDB:
# SELECT email, substring(password_hash, 1, 10) FROM users WHERE email = 'admin@hesocial.com';
# Expected: starts with $2b$10$ and is NOT literally "$2b$10$hash"

# If the hash is fake, re-seed admin users first:
# psql-equivalent: apply database/duckdb-admin-users.sql before deleting the backdoor
```

If the real hash isn't in place, re-seed admin users before proceeding. Otherwise A1 will leave you with no working admin login.

### A1. Remove hardcoded auth from `backend/src/server.ts`

- **Verified during plan revision**: `backend/src/routes/health.ts` defines both `GET /database` and `GET /r2-sync`, mounted at `/api/health` via `routes/index.ts:34`. The duplicates at `server.ts:355-393` are safe to delete.
- Delete lines 265–395 (temp auth + duplicate health).
- Confirm `authRoutes.ts` covers `POST /login`, `POST /logout`, `POST /validate`, `GET /validate`. If `GET /validate` is missing (frontend uses it), add to `authRoutes.ts` backed by real JWT verify + DB user lookup.

### A2. Remove duplicated debug block in `backend/src/routes/index.ts`

- Delete lines 385–672 (post-catch-all duplicates).
- Remove imports that become unused.

### A3. Extract and gate debug routes (static import, per corroborator)

Create `backend/src/routes/debugRoutes.ts` containing the 7 `/debug/*` handlers currently in `routes/index.ts:153-373`.

In `routes/index.ts`, add **static** import and conditional mount:

```ts
import debugRoutes from './debugRoutes.js'
import { protect, requireAdmin } from '../middleware/auth.js'

if (process.env.NODE_ENV !== 'production') {
  router.use('/debug', protect, requireAdmin, debugRoutes)
}
```

Remove the 7 handler bodies from `routes/index.ts:153-373` after the move.

### A4. Verification

```bash
# 1. Real JWT returned (not dev-token-12345)
TOKEN=$(curl -sX POST localhost:5000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@hesocial.com","password":"admin123"}' | jq -r .data.token)
# Assert TOKEN has 3 dot-separated base64 parts (JWT), not literal "dev-token-12345"
echo "$TOKEN" | awk -F. '{exit (NF==3 && $0 != "dev-token-12345") ? 0 : 1}'

# 2. Debug routes gated
curl -X POST localhost:5000/api/debug/expand-seed                  # 401 (no auth)
curl -X POST localhost:5000/api/debug/expand-seed \
  -H "Authorization: Bearer $TOKEN"                                # 200 (admin)

# 3. Production gate
NODE_ENV=production node backend/dist/server.js &
curl localhost:5000/api/debug/expand-seed                          # 501 or 404
```

### A5. Commit
`fix(security): remove auth backdoor and gate debug routes`

---

## Phase B1-B3 — Auth token + routing correctness

**Goal**: Unify token key, align registration route, fix error handler order.
**Effort**: 1–1.5 hr.

### B1. Unify token storage key (F5)

- Canonical key: **`hesocial_token`** (authService already uses this).
- **Audit confirmed**: only 4 services use `'token'`. `adminService.ts`, `mediaService.ts`, `systemHealthService.ts` have zero localStorage usage.
- **authService never stores a `user` object** — so any `removeItem('user')` elsewhere is a no-op. **Delete those lines** rather than renaming them to a non-existent key.

Required edits:
- `registrationService.ts:70` — change `getItem('token')` → `getItem('hesocial_token')`
- `analyticsService.ts:12` — change `getItem('token')` → `getItem('hesocial_token')`
- `analyticsService.ts:28-29` — change `removeItem('token')` → `removeItem('hesocial_token')`; **delete** `removeItem('user')`
- `eventService.ts:12` — change `getItem('token')` → `getItem('hesocial_token')`
- `eventService.ts:28-29` — change `removeItem('token')` → `removeItem('hesocial_token')`; **delete** `removeItem('user')`
- `participantService.ts:13` — change `getItem('token')` → `getItem('hesocial_token')`

### B2. Fix registration route (F4)

In `backend/src/routes/registrationRoutes.ts:52`:

```ts
router.post('/events/:eventId', protect, registerForEvent)
```

No compatibility alias for `POST /` — no known caller uses it, and the controller requires `eventId` anyway (currently always `undefined`).

**Smoke-test after the change** — the controller code path reading `req.params.eventId` has never actually received a real value.

Update `docs/api/API_REFERENCE.md` if it references the old path.

### B3. Fix Express error handler order (F8)

Move `server.ts:237-253` error middleware to **after** API route mount (currently `server.ts:406`).
Final order: mounts → (optional 404) → error handler.

### B4. Verification

```bash
# Curl smoke
curl localhost:5000/api/registrations/user -H "Authorization: Bearer $TOKEN"
curl -X POST localhost:5000/api/registrations/events/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"specialRequests":"test"}'

# Browser (curl can't catch Bearer null):
# - Log in at http://localhost:5173/login
# - DevTools Network tab → navigate /my-registrations
# - ASSERT: Authorization: Bearer <jwt>, not Bearer null
```

### B5. Commits
- `fix(auth): unify frontend token key to hesocial_token`
- `fix(api): align registration route with frontend, fix error handler order`

---

## Phase D1 — Backend Shared Types

**Goal**: Fix cross-cutting backend type/export issues before touching disabled routers.
**Effort**: 30–90 min.
**Risk**: Low.

### D1a. Verify, don't blindly add deps (corrected per corroborator)

`@types/cookie-parser`, `cookie-parser`, and `@aws-sdk/node-http-handler` are **already in backend/package.json**. Do not re-add.

```bash
cd backend
npm ls @types/cookie-parser
npm ls cookie-parser
npm ls @aws-sdk/node-http-handler
# If missing from node_modules: npm install (root cause is likely not "missing dep" but install state)
```

If the AWS SDK moved `NodeHttpHandler` to `@smithy/node-http-handler`, add `@smithy/node-http-handler` as an **explicit backend dependency** rather than relying on a transitive one. Update the single import site.

### D1b. Fix `getDuckDBConnection` export

Locate the real export in `backend/src/database/duckdb-connection.ts`. Either:
- Add a named alias: `export const getDuckDBConnection = <real-name>`, OR
- Update all importers to use the existing export name (grep for `getDuckDBConnection` callers first).

### D1c. Verification
```bash
cd backend && npx tsc --noEmit
# Should resolve the cross-cutting errors. Per-router errors are Phase C.
```

### D1d. Commit
`fix(types): restore backend shared dependencies and duckdb export`

---

## Phase B4 — SalesManagement Real Data

**Goal**: Replace mock data with real API calls (after D1 so backend sales types are stable).
**Effort**: 45–90 min.

### B4a. Create `frontend/src/services/salesService.ts`

Follow the `eventService.ts` pattern. Use `hesocial_token` (unified in B1). Wire to:
- `GET /api/sales/leads` (with filter query params)
- `GET /api/sales/opportunities`
- `GET /api/sales/metrics`

### B4b. Rewrite `frontend/src/pages/SalesManagement.tsx:122-268`

Delete `fetchLeads`, `fetchOpportunities`, `fetchMetrics` mock arrays and `setTimeout` stubs. Call the new service; render real data or an empty state. No fallback to mocks.

### B4c. Verification

```bash
# Backend sales routes already mounted; confirm UI:
# - /sales shows backend data (or empty state), not hardcoded 張志明/李美華
# - Lead/opp filter changes trigger new network requests
```

### B4d. Commit
`fix(sales): load SalesManagement from API, remove mock data`

---

## Phase C — Re-enable Routers (one at a time)

**Goal**: Uncomment disabled CRUD routers with verification between each.
**Effort**: 1.5–3 hr.
**Risk**: Medium.

### C order (lowest friction first, per corroborator)

1. `categoryManagement.ts`
2. `venueManagement.ts`
3. `eventManagement.ts`
4. `userManagement.ts`
5. `admin.ts`
6. `mediaRoutes.ts` (last — heaviest R2/AWS coupling)

### C per-router workflow

For each:
- Uncomment import + `router.use(...)` in `routes/index.ts`
- `cd backend && npx tsc --noEmit` — fix per-router errors
- Hit the router endpoints from curl or UI — confirm runtime
- Commit: `fix(routes): re-enable <name> router`

### C Verification (at end)

```bash
npm run typecheck                     # both workspaces green
npm run dev
# UI smoke: /admin/categories, /admin/venues, /admin/events, /admin/users, /admin, media endpoints
```

---

## Phase D2 — Frontend lucide-react

**Goal**: Fix missing icon imports.
**Effort**: 20–40 min.

`lucide-react@^0.279.0` doesn't export: `MessageCircle`, `TrendingDown`, `PieChart`, `Target`, `FileText`, `Building`, `Loader2`, `Ticket`.

**Prefer replacement over upgrade** (per corroborator — lower risk):
- For each missing icon, check `node_modules/lucide-react/dist/lucide-react.d.ts` at pinned version and substitute with the closest existing alternative.
- Only upgrade the package if ≥ 3 icons have no substitute — then verify all other icon imports still resolve post-upgrade.

### D2 Commit
`fix(frontend): replace unavailable lucide-react icons`

---

## Phase D3 — Final Verification

```bash
npm run typecheck            # 0 exit both workspaces
npm run build                # 0 exit both workspaces
npm run lint                 # 0 exit or document remaining warnings
npm run validate:all         # full gate
```

---

## Commit Breakdown (final, per corroborator)

1. `fix(security): remove auth backdoor and gate debug routes`          (A)
2. `fix(auth): unify frontend token key to hesocial_token`              (B1)
3. `fix(api): align registration route and error middleware order`      (B2+B3)
4. `fix(types): restore backend shared deps and duckdb export`          (D1)
5. `fix(sales): load SalesManagement from API`                          (B4)
6. `fix(routes): re-enable category and venue routers`                  (C1+C2)
7. `fix(routes): re-enable event management router`                     (C3)
8. `fix(routes): re-enable user and admin routers`                      (C4+C5)
9. `fix(routes): re-enable media router`                                (C6)
10. `fix(frontend): restore frontend typecheck`                         (D2)

---

## Phase E — F10: Portable Database Paths

**Goal**: Remove hardcoded absolute dev paths that break deployment on Render / any non-local host.
**Effort**: 5 min.
**Risk**: Low.

### E1. `backend/src/database/BlueGreenDatabaseManager.ts:50-53`

Before:

```ts
constructor(
  private baseDbPath: string = '/home/yanggf/a/hesocial/hesocial.duckdb',
  private schemaPath: string = '/home/yanggf/a/hesocial/database/duckdb-schema.sql'
) {}
```

After:

```ts
constructor(
  private baseDbPath: string = path.join(process.cwd(), 'hesocial.duckdb'),
  private schemaPath: string = path.join(process.cwd(), 'database/duckdb-schema.sql')
) {}
```

`path` is already imported at line 8. Matches the resolution pattern used by `duckdb-connection.ts`.

### E2. Commit

`fix(db): use cwd-relative paths in BlueGreenDatabaseManager`

---

## Out of Scope

- Refresh tokens, rate limiting, expanded OAuth (TODO.md medium)
- Test coverage targets (TODO.md tech debt)
- CI/CD pipeline (TODO.md tech debt)
- Stripe payment integration (TODO.md medium)
- `tokenStorage.ts` helper — revisit if key bugs recur post-B1

---

## Rollback

Each numbered commit on `fix/codex-remediation` is independently revertible. No force-push. Merge to `master` only after local verification + user confirmation.

---

## Success Criteria

- [ ] `/api/auth/login` returns signed JWT, not `dev-token-12345`
- [ ] Admin login works with seeded bcrypt hash (not debug-seed fake `$2b$10$hash`)
- [ ] `/api/debug/*` mounted only in non-production, admin-gated
- [ ] No `Bearer null` in browser network tab on protected calls
- [ ] Triggering 401 clears token from localStorage (no stale `user` key references)
- [ ] Event registration succeeds end-to-end
- [ ] SalesManagement loads from API (or empty state), no hardcoded arrays
- [ ] `npm run typecheck` exits 0 on both workspaces
- [ ] All admin CRUD pages work against real backend
