# hesocial-frontend (Dioxus CSR)

Rust/WASM replacement for `frontend/`. Round 1 proved the toolchain (Dioxus
0.7 CSR → static assets, Tailwind tokens, three Rust test layers). Round 2
ports `/login`. Round 3 ports the `/events` list page (filters, cards,
pagination, `GET /api/events`). Round 4 ports the shared Navbar/Footer shell
around every route, including the user-menu exit animation (CSS keyframes plus
a Rust presence state so the node stays mounted while it animates out). Other
application pages are not ported; `/`, `/register`, `/forgot-password`,
`/profile`, `/profile/registrations`, `/events/:id`, `/vvip`, `/admin`,
`/event-mgmt`, `/admin/sales`, and `/admin/system` exist as stubs so login
links, the OAuth-callback regression, event-detail links, and the signed-in
dropdown can be exercised.

Pinned crate versions are in `Cargo.toml` / `Cargo.lock`.

| Crate | Version | Why |
|---|---|---|
| `dioxus` (`web` + `router`) | **0.7.10** | Current stable CSR + first-party router. Edition 2024, MSRV 1.83. Matches the 0.7 CLI. |
| `dioxus-router` | **0.7.10** | Pulled via `dioxus` feature `router`, not a direct dep. |
| `dioxus-ssr` | **0.7.10** | Host/wasm component tests render `VirtualDom` to HTML. |
| `thirtyfour` | **0.37.5** | W3C WebDriver client with `WebDriver::managed()`: downloads a matching chromedriver, spawns it, and tears it down on `quit()`. Chosen over `fantoccini` for that lifecycle and Chrome capability helpers (`set_headless`, `set_no_sandbox`). Requires rustc **1.88+**. |
| `wasm-bindgen-test` | **0.3.77** | Matches installed `wasm-bindgen` 0.2.127. |
| `tiny_http` | **0.12.0** | In-test static file server. No long-lived `dx serve`. |
| `gloo-net` | **0.6** | Wasm-only `fetch` for `POST /api/auth/login` and `GET /api/events`. |
| `serde` / `serde_json` | **1** | Login and events request/response JSON. |
| `web-sys` | **0.3** | Wasm `window.location` + `localStorage`. |
| `js-sys` | **0.3** | Wasm-only `Date` for `zh-TW` event timestamps. |

## Prerequisites

```bash
# Rust + wasm32 (this machine already had both)
rustup toolchain install stable
rustup target add wasm32-unknown-unknown

# Dioxus CLI 0.7.10 — prefer the prebuilt binary.
# `cargo install dioxus-cli --version 0.7.10 --locked` needs pkg-config + libssl-dev
# and failed here without them. Prebuilt:
curl -fsSL -o /tmp/dx.tar.gz \
  https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-unknown-linux-gnu.tar.gz
tar -xzf /tmp/dx.tar.gz -C /tmp
install -m 0755 /tmp/dx "$HOME/.cargo/bin/dx"
dx --version   # dioxus 0.7.10 (57d6794)

# Alternative (downloads a prebuilt as well):
# curl -fsSL https://dioxuslabs.com/install.sh | bash

# wasm-bindgen-test-runner (Node.js executes the wasm tests)
cargo install wasm-bindgen-cli --version 0.2.127 --locked
# node is required on PATH (tested with v24.3.0)

# E2E: Google Chrome on PATH. thirtyfour downloads chromedriver itself.
# google-chrome --version   # tested with 150.0.7871.128
```

Do **not** leave `dx serve` running. Builds and tests below are one-shot.

## Build (static assets)

```bash
cd frontend-rust
dx bundle --web --release --out-dir dist
```

Output (Cloudflare Worker `[assets]` compatible — `index.html` at the directory root):

```
frontend-rust/dist/public/index.html
frontend-rust/dist/public/assets/*.js      # wasm-bindgen glue (generated)
frontend-rust/dist/public/assets/*.wasm
frontend-rust/dist/public/assets/*.css     # Tailwind output
```

When this crate replaces the React SPA, point `backend-rust/wrangler.toml` at:

```toml
[assets]
directory = "../frontend-rust/dist/public"
not_found_handling = "single-page-application"
run_worker_first = ["/api/*"]
```

Do **not** change `wrangler.toml` in this scaffold.

`dx` auto-detects `tailwind.config.js` (Tailwind v3) and `tailwind.css`. Design
tokens (`luxury-*`, Playfair/Inter, `.luxury-button`) are copied from
`frontend/tailwind.config.js` / `frontend/src/styles/index.css`; only the
`content` globs were pointed at `./src/**/*.{rs,html,css}`.

Known build warning: `wasm-opt` aborted here (`SIGABRT`, "unsupported version of
DWARF"). `dx` still copied the unoptimized `.wasm` (~1.8 MiB) and the page
loads. Workaround if you need `wasm-opt`:

```bash
dx bundle --web --release --out-dir dist --debug-symbols false
```

## Tests

### 1. Pure logic (`cargo test`)

```bash
cd frontend-rust
cargo test --test logic
```

Covers toggle helpers plus login error selection, `POST /api/auth/login`
JSON parsing, Bearer header formatting, OAuth `?token=` extraction, the
claim-before-`/complete-profile`→`/profile` redirect ordering, events query
strings, filter→page-1 reset, pagination range, the
`success: false` / malformed-body collapse to `total: 0, totalPages: 1`,
exact-path nav highlighting, session entries for signed-out / signed-in /
admin, the `viewAdmin` gate (`admin` | `super_admin`), and dropdown
presence (mounted through exit).
No browser, no wasm.

### 2. Component (`wasm-bindgen-test`)

```bash
cd frontend-rust
cargo test --target wasm32-unknown-unknown --test wasm
```

Uses the runner set in `.cargo/config.toml`. Renders `Home`, `/login`, and
`/events` through `VirtualDom` + `dioxus-ssr` inside Node via
`wasm-bindgen-test-runner`. Asserts Traditional Chinese copy, password
masking, LinkedIn permanently disabled, submit disabled while in flight,
events loading/empty states, card fields, exclusivity badge/star/diamond
selection (including `exclusivityLevel: null`), signed-out vs signed-in vs
admin navbar markup, dropdown open/closed/exiting, mobile Menu/X toggle, and
footer copy.

### 3. WebDriver E2E (`thirtyfour`)

```bash
cd frontend-rust
dx bundle --web --release --out-dir dist   # once, produces dist/public
cargo test --test e2e -- --nocapture
```

**Lifecycle (all inside the test, nothing left running):**

1. Locate `dist/public/index.html` (or the `target/dx/.../web/public` fallback).
2. Bind `tiny_http` on `127.0.0.1:0`. The **thread owns the `Server`**. A
   An `AtomicBool` stop flag plus `recv_timeout(50ms)` lets `shutdown()` join
   the thread; the listener is then gone (proven by `harness_starts_and_stops_twice`).
   `/api/auth/login`, `/api/auth/google`, and `GET /api/events` are stubbed
   in-process — the test never calls a real backend.
3. `WebDriver::managed(chrome)` downloads a matching chromedriver, spawns it,
   and launches headless Chrome (`--headless`, `--no-sandbox`, `--disable-gpu`,
   `--disable-dev-shm-usage`).
4. Covers the home toggle, login copy, 401 error string, success →
   `localStorage.hesocial_token` + navigate `/`, in-flight submit disable,
   Google full-page navigation, password reveal, OAuth
   `/complete-profile?token=` claimed **before** the `/profile` redirect,
   the `/events` list, search filtering, pagination, an API 500 that
   yields the empty state rather than a crash, the signed-out shell,
   user-menu open/close including the exit class, mobile toggle, admin
   entries only for `role: admin`, and logout back to the signed-out shell.
5. `driver.quit()` closes the browser and chromedriver. `StaticHarness::shutdown`
   joins the HTTP thread. Nothing is left running.

`cargo test` (no filters) on the host also runs `--test logic` and `--test e2e`.
The wasm layer is a different target and must be invoked separately.

## Layout

```
frontend-rust/
  Cargo.toml          # standalone crate; not in backend-rust workspace
  Dioxus.toml         # out_dir = dist, HTML title
  tailwind.config.js  # reused luxury tokens; content globs → src/**/*.rs
  tailwind.css        # Tailwind v3 input + luxury classes + hs-enter keyframes
  assets/tailwind.css # generated; gitignored — do not commit
  src/auth.rs         # login parse, token key, OAuth claim-before-redirect
  src/events.rs       # query string, parse, badge/price/image helpers, GET
  src/icons.rs        # Lucide SVG Icon enum (add a variant to scale)
  src/logic.rs        # pure toggle helpers
  src/permissions.rs  # AuthSnapshot + Can flags (navbar wires view_admin)
  src/shell.rs        # nav items, active-path, session entries, Presence
  src/pages.rs        # page modules; Events is the first extracted page
  src/pages/events.rs # Events container + EventsScreen + EventCard
  src/pages/shell.rs  # NavbarScreen + Footer
  src/ui.rs           # Route + App + Shell layout + Login + remaining stubs
  src/main.rs         # claim_oauth_token_on_boot(); dioxus::launch(App)
  tests/logic.rs
  tests/wasm.rs
  tests/e2e.rs
```

`/login` is a public route. OAuth tokens are claimed from `window.location`
in `main` and at the start of `App`, **before** `Router` or the `/profile`
guard run. That ordering is load-bearing: `/complete-profile` redirects to
`/profile` and drops `?token=`.
