# Remediation Plan — Codex Code Review Findings

**Date**: 2026-04-18
**Source**: External code review (Codex) cross-validated against current code, independently reviewed by Gemini (via code inspection — Gemini CLI quota exhausted at review time)
**Scope**: 8 Codex findings + 1 mock-data issue discovered during validation
**Branch strategy**: Single feature branch `fix/codex-remediation`, phase-by-phase commits

---

## Findings Summary

| ID | Severity | Area | Summary |
|----|----------|------|---------|
| F1 | Critical | Auth | Hardcoded admin login (`admin@hesocial.com`/`admin123` → static `dev-token-12345`) mounted in `server.ts` startup, not env-gated |
| F2 | Critical | Routes | Unauthenticated `/api/debug/*` endpoints including `DELETE FROM events` |
| F3 | High | Build | `npm run typecheck` fails on both workspaces — missing types, invalid imports, missing exports |
| F4 | High | API | Frontend `POST /registrations/events/:eventId` hits non-existent backend route (BE only has `POST /registrations/`) |
| F5 | High | Auth | Token stored under `hesocial_token` by authService; read as `token` by 4 other services → `Bearer null` on protected calls |
| F6 | Medium | Routes | admin/users/events/venues/categories/media routers commented out; frontend still calls them (falls to 501) |
| F7 | Medium | Routes | `router.use('*')` catch-all at `routes/index.ts:376` makes lines 385-672 unreachable (duplicated block) |
| F8 | Medium | Express | Global error handler registered before route mounting — `next(err)` skips it |
| F9 | Medium | Data | `SalesManagement.tsx:122-268` uses mock data + `setTimeout` despite working backend routes existing |

---

## Execution Order (revised after Gemini review)

**A (security) → B (broken flows + mocks) → D1 (shared types) → C (re-enable routers) → D2–D3 (frontend types, verify)**

Rationale: D1 (shared `getDuckDBConnection` export + missing `@types/*`) almost certainly blocks multiple disabled routers simultaneously. Doing C before D1 means fighting the same type error across 6 routers.

---

## Phase A — Security Critical (stop the bleeding)

**Goal**: Eliminate auth bypass and destructive unauthenticated endpoints.
**Estimated effort**: 30–60 min
**Risk**: Low.

### A1. Remove hardcoded auth from `backend/src/server.ts`
- **Pre-check (already verified)**: `backend/src/routes/health.ts` already defines `GET /database` and `GET /r2-sync`, mounted at `/api/health` via `routes/index.ts:34`. The temp copies at `server.ts:355-393` are duplicates — safe to delete.
- Delete lines 265–395 (temp auth routes + duplicate health routes).
- Confirm `authRoutes.ts` handles `POST /login`, `POST /logout`, `POST /validate`, `GET /validate`. If any are missing (especially `GET /validate` used by frontend), add to `authRoutes.ts` backed by real JWT + DB user lookup.

### A2. Remove duplicated debug block in `backend/src/routes/index.ts`
- Delete lines 385–672 (everything after `router.use('*')` catch-all).
- Remove any imports that become unused.

### A3. Gate remaining `/debug/*` routes
- Move all `/debug/*` handlers into a new `backend/src/routes/debugRoutes.ts` file.
- In `routes/index.ts`, mount only when `NODE_ENV !== 'production'`:
  ```ts
  if (process.env.NODE_ENV !== 'production') {
    const debugRoutes = (await import('./debugRoutes.js')).default
    router.use('/debug', protect, requireAdmin, debugRoutes)
  }
  ```
- In production, the paths will fall to the `router.use('*')` 501 handler (acceptable — better than 401 signal leakage).

### A4. Verification
```bash
# Development mode
curl -X POST localhost:5000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@hesocial.com","password":"admin123"}' \
  | jq -r .data.token

# ASSERT: returned token is a signed JWT (3 dot-separated base64 segments),
# NOT the literal string "dev-token-12345".
# Decode header to confirm: echo "$TOKEN" | cut -d. -f1 | base64 -d

# Debug routes require admin in dev
curl -X POST localhost:5000/api/debug/expand-seed          # 401 without auth
curl -X POST localhost:5000/api/debug/expand-seed \
  -H "Authorization: Bearer $ADMIN_TOKEN"                  # 200 with admin

# Production mode (NODE_ENV=production node dist/server.js)
curl -X POST localhost:5000/api/debug/expand-seed          # 501 (debug router not mounted)
```

### A5. Commit
`fix(security): remove hardcoded auth backdoor and gate debug routes`

---

## Phase B — Broken User Flows

**Goal**: Login → protected API → registration working end-to-end; replace SalesManagement mocks.
**Estimated effort**: 1.5–2.5 hr
**Risk**: Medium (cross-cutting auth token change).

### B1. Unify token storage key (F5)
- Canonical key: **`hesocial_token`** (matches authService, more specific than `token`).
- **Audit confirmed** (verified during plan revision): only 4 services use the wrong `'token'` key — `adminService.ts`, `mediaService.ts`, `systemHealthService.ts` have no localStorage usage.
- **Required edits** (both read AND remove-on-401 paths):
  - `registrationService.ts:70` — `getItem('token')`
  - `analyticsService.ts:12` — `getItem('token')`
  - `analyticsService.ts:28` — `removeItem('token')` + `removeItem('user')` (check canonical user key in authService)
  - `eventService.ts:12` — `getItem('token')`
  - `eventService.ts:28` — `removeItem('token')` + `removeItem('user')`
  - `participantService.ts:13` — `getItem('token')`
- **No shared helper this round** — a `tokenStorage.ts` refactor is nice-to-have but adds churn. Direct replacement first; extract helper later if bugs recur.

### B2. Fix registration route mismatch (F4)
- Change backend to match frontend: `router.post('/events/:eventId', protect, registerForEvent)` in `registrationRoutes.ts:52`.
- **Note**: `registrationController.ts:9` already reads `req.params.eventId`, but since the current route has no `:eventId` in the path, it has **never received a real value**. After the fix, smoke-test registration end-to-end — do not assume the controller works.
- Update `docs/api/API_REFERENCE.md` and CLAUDE.md API summary if the old path is referenced.

### B3. Fix Express error handler ordering (F8)
- Move the error middleware (`server.ts:237-253`) to **after** API routes mount (currently at line 406).
- Final order: route mounts → 404 handler (if any) → error handler.

### B4. Replace SalesManagement mock data (F9 — pulled from original Phase C3)
- `frontend/src/pages/SalesManagement.tsx:122-268`: remove `fetchLeads`, `fetchOpportunities`, `fetchMetrics` mock arrays + `setTimeout` fakes.
- Create `frontend/src/services/salesService.ts` following the `eventService.ts` pattern (uses unified `hesocial_token` per B1).
- Wire to real endpoints: `GET /api/sales/leads`, `GET /api/sales/opportunities`, `GET /api/sales/metrics`.

### B5. Verification
```bash
# 1. Curl-level smoke tests
TOKEN=$(curl -sX POST localhost:5000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@hesocial.com","password":"admin123"}' | jq -r .data.token)

curl localhost:5000/api/registrations/user -H "Authorization: Bearer $TOKEN"
curl -X POST localhost:5000/api/registrations/events/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"specialRequests":"test"}'

# 2. Browser-level regression check (curl bypasses localStorage, can't catch Bearer null)
#    - Log in at http://localhost:5173/login
#    - Open DevTools Network tab
#    - Navigate to /my-registrations
#    - ASSERT: Authorization header is "Bearer <real-jwt>", NOT "Bearer null"
#    - Navigate to /sales
#    - ASSERT: data loads from network request, not from a hardcoded array
#    - Trigger 401 (manually delete hesocial_token in DevTools Application tab)
#    - ASSERT: after next protected call, user is logged out and user object is cleared
```

### B6. Commit
`fix(auth,api): unify token key, align registration route, fix error ordering, replace sales mocks`

---

## Phase D1 — Shared Type Infrastructure (moved ahead of C)

**Goal**: Make typecheck green for the shared backend types blocking disabled routers.
**Estimated effort**: 30–60 min
**Risk**: Low.

### D1a. Backend dependencies and exports
- Add dev dep: `@types/cookie-parser`
- Replace `@aws-sdk/node-http-handler` imports with `@smithy/node-http-handler` (AWS SDK v3 moved this package).
- Locate the real export in `backend/src/database/duckdb-connection.ts` and either:
  - Add a named export `getDuckDBConnection` alias, OR
  - Fix all importers to use the correct existing name.

### D1b. Verification
```bash
cd backend && npx tsc --noEmit
# Fix any remaining cross-cutting errors. Router-specific errors can wait for Phase C.
```

### D1c. Commit
`fix(types): add missing backend types and unify duckdb connection export`

---

## Phase C — Restore Disabled Routers

**Goal**: Re-enable admin/event/venue/category/user/media CRUD.
**Estimated effort**: 1.5–3 hr (depends on residual per-router errors).
**Risk**: Medium.

### C1. For each router (`admin.ts`, `userManagement.ts`, `eventManagement.ts`, `venueManagement.ts`, `categoryManagement.ts`, `mediaRoutes.ts`):
- Uncomment the import line in `routes/index.ts:12-21`
- Uncomment the `router.use(...)` mount at `routes/index.ts:134-140`
- Run `npx tsc --noEmit` — fix any per-router errors
- Hit the router from curl or UI to confirm runtime integrity
- Commit per router: `fix(routes): re-enable <name> router`

### C2. Verification
```bash
npm run typecheck                     # both workspaces green
npm run dev
# UI smoke:
# - /admin/users loads user list
# - /admin/events shows create/edit/delete buttons that work
# - /admin/venues, /admin/categories, /admin/media load real data
```

---

## Phase D2–D3 — Frontend Types + Final Verify

**Goal**: Frontend typecheck green, build green.
**Estimated effort**: 30–60 min
**Risk**: Low (mechanical).

### D2. lucide-react icon imports
- `lucide-react@^0.279.0` does not export: `MessageCircle`, `TrendingDown`, `PieChart`, `Target`, `FileText`, `Building`, `Loader2`, `Ticket`.
- Recommend **upgrade** to latest (check for breaking changes to icons already in use). Fallback: replace each missing icon with an existing equivalent in 0.279.

### D3. Final verification
```bash
npm run typecheck            # 0 exit on both workspaces
npm run build                # 0 exit on both workspaces
npm run lint                 # 0 exit or documented remaining warnings
npm run validate:all         # full gate (docs + lint + typecheck + test)
```

### D4. Commit
`fix(types): upgrade lucide-react and restore frontend build`

---

## Out of Scope (deferred)

- Refresh token / rate limiting / OAuth provider expansion (TODO.md medium)
- Test coverage targets (TODO.md technical debt)
- CI/CD pipeline (TODO.md technical debt)
- Stripe payment integration (TODO.md medium)
- `tokenStorage.ts` helper extraction (revisit if token-key bugs recur post-B1)

---

## Rollback Strategy

- Each phase/sub-step = separate commit on `fix/codex-remediation`
- `git revert <sha>` for targeted rollback; no force-push
- Merge to `master` only after all phases verified locally + user confirmation

---

## Success Criteria

- [ ] `/api/auth/login` returns a real signed JWT (not `dev-token-12345`)
- [ ] `/api/debug/*` mounted only when `NODE_ENV !== 'production'`, admin-gated
- [ ] Browser network tab shows `Authorization: Bearer <jwt>` on all protected calls (no `Bearer null`)
- [ ] Triggering 401 clears both token and user from localStorage
- [ ] Registering for an event succeeds end-to-end
- [ ] `SalesManagement.tsx` renders backend data (or empty state) — no hardcoded arrays
- [ ] `npm run typecheck` exits 0 on both workspaces
- [ ] All admin CRUD pages in frontend work against real backend
