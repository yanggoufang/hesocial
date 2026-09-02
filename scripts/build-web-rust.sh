#!/usr/bin/env bash
set -euo pipefail

# Builds the Rust SPA as two wasm bundles so an ordinary visitor never
# downloads the admin console. Route membership is decided by the
# `admin-bundle` cargo feature in frontend-rust/src/ui.rs.
#
#   frontend-rust/dist        public + member routes (also what the e2e suite serves)
#   frontend-rust/dist-admin  /admin* and /event-mgmt* routes
#
# Both carry /, /login and /profile so a cross-bundle hard navigation lands
# somewhere real. Serving them needs the Worker to hand /admin* and
# /event-mgmt* the admin index.html; see docs/DEPLOYMENT_TARGETS.md.

cd "$(dirname "$0")/../frontend-rust"

DX_FLAGS=(--release --platform web --debug-symbols false)

rm -rf dist dist-admin target/dx
dx bundle "${DX_FLAGS[@]}" --features admin-bundle
mv dist dist-admin

rm -rf target/dx
dx bundle "${DX_FLAGS[@]}"

for label in dist dist-admin; do
  for f in "$label"/public/assets/*.wasm; do
    printf '%-11s %-52s raw=%8d gzip=%8d\n' "$label" "$(basename "$f")" \
      "$(stat -c%s "$f")" "$(gzip -9c "$f" | wc -c)"
  done
done
