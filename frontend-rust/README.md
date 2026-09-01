# hesocial-frontend (Dioxus CSR scaffold)

Rust/WASM replacement **scaffold** for `frontend/`. It proves the toolchain
(Dioxus 0.7 CSR → static assets, Tailwind tokens, three Rust test layers).
It does **not** port application pages.

Pinned crate versions are in `Cargo.toml` / `Cargo.lock`.

| Crate | Version | Why |
|---|---|---|
| `dioxus` (`web` + `router`) | **0.7.10** | Current stable CSR + first-party router. Edition 2024, MSRV 1.83. Matches the 0.7 CLI. |
| `dioxus-router` | **0.7.10** | Pulled via `dioxus` feature `router`, not a direct dep. |
| `dioxus-ssr` | **0.7.10** | Host/wasm component tests render `VirtualDom` to HTML. |
| `thirtyfour` | **0.37.5** | W3C WebDriver client with `WebDriver::managed()`: downloads a matching chromedriver, spawns it, and tears it down on `quit()`. Chosen over `fantoccini` for that lifecycle and Chrome capability helpers (`set_headless`, `set_no_sandbox`). Requires rustc **1.88+**. |
| `wasm-bindgen-test` | **0.3.77** | Matches installed `wasm-bindgen` 0.2.127. |
| `tiny_http` | **0.12.0** | In-test static file server. No long-lived `dx serve`. |

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

Covers `toggle_label` / `next_toggled` in `src/logic.rs`. No browser, no wasm.

### 2. Component (`wasm-bindgen-test`)

```bash
cd frontend-rust
cargo test --target wasm32-unknown-unknown --test wasm
```

Uses the runner set in `.cargo/config.toml`. Renders `Home` through
`VirtualDom` + `dioxus-ssr` inside Node via `wasm-bindgen-test-runner`.
Asserts the heading and toggle button appear in the markup.

### 3. WebDriver E2E (`thirtyfour`)

```bash
cd frontend-rust
dx bundle --web --release --out-dir dist   # once, produces dist/public
cargo test --test e2e -- --nocapture
```

**Lifecycle (all inside the test, nothing left running):**

1. Locate `dist/public/index.html` (or the `target/dx/.../web/public` fallback).
2. Bind `tiny_http` on `127.0.0.1:0` in a background thread; serve the bundle
   with `application/wasm` for `.wasm`.
3. `WebDriver::managed(chrome)` downloads a chromedriver matching the installed
   Chrome, spawns it, and launches headless Chrome (`--headless`, `--no-sandbox`,
   `--disable-gpu`, `--disable-dev-shm-usage`).
4. Load `/`, wait for `#scaffold-heading` == `HeSocial`, click `#toggle-btn`
   (`Off` → `On`).
5. `driver.quit()` closes the browser and the chromedriver subprocess. The
   HTTP thread ends when the `cargo test` process exits.

`cargo test` (no filters) on the host also runs `--test logic` and `--test e2e`.
The wasm layer is a different target and must be invoked separately.

## Layout

```
frontend-rust/
  Cargo.toml          # standalone crate; not in backend-rust workspace
  Dioxus.toml         # out_dir = dist, HTML title
  tailwind.config.js  # reused luxury tokens; content globs → src/**/*.rs
  tailwind.css        # Tailwind v3 input + luxury component classes
  assets/tailwind.css # generated CSS consumed by asset!("/assets/tailwind.css")
  src/logic.rs        # pure toggle helpers
  src/ui.rs           # Route + App + Home (heading + button)
  src/main.rs         # dioxus::launch(App)
  tests/logic.rs
  tests/wasm.rs
  tests/e2e.rs
```

The trivial page is a heading plus a button that flips `Off`/`On`. Router is
wired (`Route::Home` at `/`) so later page ports have a place to land.
