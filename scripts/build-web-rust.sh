#!/usr/bin/env bash
set -euo pipefail

# Builds the Rust SPA as two wasm bundles so an ordinary visitor never
# downloads the admin console. Route membership is decided by the
# `admin-bundle` cargo feature in frontend-rust/src/ui.rs.
#
#   frontend-rust/dist         public + member routes (also what the e2e suite serves)
#   frontend-rust/dist-admin   /admin* and /event-mgmt* routes
#   frontend-rust/dist-worker  both, merged into the tree wrangler uploads
#
# The merge works because dx hashes every asset filename, so the two bundles'
# JS and wasm never collide, and their tailwind CSS is byte-identical. The
# public entry stays at /index.html for Cloudflare's SPA fallback; the admin
# entry becomes /admin.html, which the Worker returns for the admin prefixes
# (ADMIN_BUNDLE_PREFIXES in backend-rust/crates/worker/src/lib.rs).

cd "$(dirname "$0")/../frontend-rust"

DX_FLAGS=(--release --platform web --debug-symbols false)

rm -rf dist dist-admin dist-worker target/dx
dx bundle "${DX_FLAGS[@]}" --features admin-bundle
mv dist dist-admin

rm -rf target/dx
dx bundle "${DX_FLAGS[@]}"

mkdir -p dist-worker
cp -r dist/public/. dist-worker/
cp -r dist-admin/public/assets/. dist-worker/assets/
cp dist-admin/public/index.html dist-worker/admin.html

for entry in dist-worker/index.html dist-worker/admin.html; do
  script=$(grep -o '/\./assets/[^"]*\.js' "$entry" | head -1)
  test -f "dist-worker${script#/.}" \
    || { echo "merged tree is missing $script for $entry" >&2; exit 1; }
done

for label in dist dist-admin; do
  for f in "$label"/public/assets/*.wasm; do
    printf '%-11s %-52s raw=%8d gzip=%8d\n' "$label" "$(basename "$f")" \
      "$(stat -c%s "$f")" "$(gzip -9c "$f" | wc -c)"
  done
done
printf '%-11s %d files, %d bytes on disk\n' dist-worker \
  "$(find dist-worker -type f | wc -l)" \
  "$(du -sb dist-worker | cut -f1)"
