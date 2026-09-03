# Development Commands

**Last Updated**: September 3, 2026 — Express backend archived
**Scope**: every script in the root `package.json`, plus the Rust toolchain commands
**Source of truth**: the root `package.json`. `npm run validate:docs` fails if this file names a script that does not exist.

The Node/Express backend and its DuckDB database were archived to
`archive/backend/` — see `archive/README.md`. The API is the Rust Worker in
`backend-rust/`, and the SPA is being replaced by the Rust one in
`frontend-rust/`. Commands that used to drive Express (`npm start`, the
migration scripts, the seed script, and every `*:backend` variant) are gone from
the root package rather than left pointing at an archived directory.

---

## Setup

```bash
npm run setup          # root + contract dependencies
npm run clean          # remove frontend-rust/dist*
```

Rust needs its own toolchain: a stable `cargo` with the `wasm32-unknown-unknown`
target, `dx` (the Dioxus CLI) for the frontend, and `npx wrangler` for the
Worker. `~/.turso/turso` is required by the contract tests.

## Run

```bash
npm run dev            # Dioxus dev server (dx serve)
```

There is no backend dev server script. Run the Worker yourself when you need it:

```bash
cd backend-rust && npx wrangler dev
```

Frontend-rust runs under the Dioxus CLI:

```bash
cd frontend-rust && dx serve
```

## Build

```bash
npm run build          # Worker wasm + Rust SPA bundles
npm run build:worker   # Rust Worker -> wasm32
npm run build:web-rust # Rust SPA -> two bundles + the merged dist-worker tree
```

`build:web-rust` is the two-bundle build described in
[Deployment Targets](../DEPLOYMENT_TARGETS.md): a public bundle and an admin
bundle, merged into `frontend-rust/dist-worker` for wrangler.

## Test

```bash
npm run test               # the pre-commit gate: cargo test -p core
npm run test:rust          # backend-rust: cargo test -p core
npm run test:web-rust      # frontend-rust: cargo test (logic + SSR + e2e)
npm run test:contract:rust # the Rust Worker under workerd against a local sqld
```

`test:contract:rust` lives in the `contract/` workspace. It needs a built Worker
shim, so run `cd backend-rust && npx wrangler deploy --dry-run` first — the
config fails loudly if the shim is missing or older than the Rust sources.

The `frontend-rust` e2e suite drives a real Chrome through WebDriver and serves
`frontend-rust/dist`, so build it first with `npm run build:web-rust`.

**`npm run test` is the pre-commit gate.** `archive/frontend/` has no tests; the suites that
all — archiving the Express backend took the only Vitest suites the root `test`
script ran, which is why `test:rust` is now part of it. The suites that actually
carry the project (`test:web-rust`, 293 logic + 22 e2e; `test:contract:rust`,
49) each need a build first, so they are not in the pre-commit gate. Run them
before anything that matters.

Single tests:

```bash
cd frontend-rust  && cargo test --test <suite> <test-name>
cd backend-rust   && cargo test -p core <test-name>
```

## Lint and typecheck

```bash
npm run lint           # clippy over backend-rust
npm run lint:rust      # same thing
```

`frontend-rust` has its own gates: `cargo fmt --check` and
`cargo clippy --all-targets`.

## Documentation

```bash
npm run validate:docs    # the pre-commit gate; checks scripts, routes, schema
npm run generate:api-docs
npm run monitor:docs
npm run validate:all     # validate:docs + lint + typecheck + test
```

`validate:docs` reads the Rust Worker's route table in
`backend-rust/crates/worker/src/`, the schema in `backend-rust/sql/schema.sql`,
and the script list in this file.

## Database

The live database is Turso/libSQL, reached over Hrana HTTP by
`backend-rust/crates/worker/src/db.rs`. Schema and seed data are plain SQL in
`backend-rust/sql/`, applied by whoever provisions the database — there is no
migration runner in the Rust stack. The DuckDB migration CLI and its
migration and seed scripts went to `archive/backend/`.

For a local database the contract tests already stand one up:

```bash
~/.turso/turso dev --port 8481
```
