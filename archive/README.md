# archive/

Code that is no longer part of the build, kept for reference rather than use.
Nothing here is installed, linted, typechecked, tested, or deployed: `archive/`
is not an npm workspace and no root script points into it.

## backend/ — the Node/Express + DuckDB API

Archived 2026-09-03. It was replaced by the Rust Worker in `backend-rust/`,
which has served production since Render was decommissioned; by then Express was
running nowhere but a developer's laptop.

What it still contains, and where the live equivalent is:

| Archived | Live equivalent |
| --- | --- |
| `src/routes/`, `src/controllers/` — the Express API | `backend-rust/crates/worker/src/` |
| `src/database/migrations/` — the DuckDB migration runner and its migrations | `backend-rust/sql/schema.sql`, applied when the Turso database is provisioned |
| `src/database/seed.ts` | `backend-rust/sql/seed.sql` |
| `test/contract/express.contract.test.ts` | — the Express half of the dual-target contract suite; nothing replaces it, because there is no second implementation left to compare against |

**The Rust half of that contract suite was not archived.** It moved to the
`contract/` workspace at the repo root, still runs as
`npm run test:contract:rust`, and still passes 49/49. Archiving it along with
Express would have left the Rust API with no integration tests at all.

The DuckDB file (`hesocial.duckdb`) and the schema in `database/` are likewise
inert. They are the shape the Turso schema was derived from, so they are worth
reading when a column's history is in question, and worth nothing when running
anything.

### If you need to run it

Don't, unless you are reading history. It expects its own `npm install`, a
`.env` with the old variable names, and a DuckDB file that no longer matches the
live schema.
