#!/usr/bin/env bash
# Verifies the Worker hands each path the right SPA bundle.
#
#   scripts/verify-spa-routing.sh [base-url]     (default http://127.0.0.1:8787)
#
# The two bundles are told apart by the hashed script tag in the HTML that
# comes back, read from frontend-rust/dist-worker, so this needs the same build
# that is being served. Failure here is quiet in a browser - the wrong bundle
# renders a 404 page and rewrites the address bar - which is why it is a script
# rather than something you eyeball.
set -uo pipefail

BASE="${1:-http://127.0.0.1:8787}"
TREE="$(dirname "$0")/../frontend-rust/dist-worker"

entry_script() { grep -o 'assets/hesocial-frontend-[a-z0-9]*\.js' "$1" | head -1; }
PUBLIC=$(entry_script "$TREE/index.html")
ADMIN=$(entry_script "$TREE/admin.html")
if [ -z "$PUBLIC" ] || [ -z "$ADMIN" ] || [ "$PUBLIC" = "$ADMIN" ]; then
  echo "cannot identify the two bundles in $TREE - run npm run build:web-rust" >&2
  exit 2
fi

fail=0
check() {
  local path="$1" want="$2" code js got
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 15 "$BASE$path")
  js=$(curl -s --max-time 15 "$BASE$path" | grep -o 'assets/hesocial-frontend-[a-z0-9]*\.js' | head -1)
  case "$js" in
    "$PUBLIC") got=public ;;
    "$ADMIN")  got=admin ;;
    "")        got=none ;;
    *)         got="$js" ;;
  esac
  if [ "$code" = "200" ] && [ "$got" = "$want" ]; then
    printf '  ok   %-26s %s\n' "$path" "$got"
  else
    printf '  FAIL %-26s http=%s bundle=%s (wanted %s)\n' "$path" "$code" "$got" "$want"
    fail=$((fail + 1))
  fi
}

check_api() {
  local path="$1" ctype
  ctype=$(curl -s -o /dev/null -w '%{content_type}' --max-time 15 "$BASE$path")
  case "$ctype" in
    application/json*) printf '  ok   %-26s json\n' "$path" ;;
    *) printf '  FAIL %-26s content-type=%s (SPA fallback ate an API path)\n' "$path" "$ctype"
       fail=$((fail + 1)) ;;
  esac
}

echo "verifying $BASE"
echo "public bundle: $PUBLIC"
echo "admin bundle:  $ADMIN"
echo
echo "public routes"
for p in / /events /events/11 /vvip /profile /profile/registrations; do check "$p" public; done
echo "paths that only look like an admin prefix"
for p in /administrators /admin-backup /event-mgmt-archive; do check "$p" public; done
echo "admin routes"
for p in /admin /admin/users /admin/analytics /admin/sales /admin/system \
         /event-mgmt /event-mgmt/categories /event-mgmt/venues /event-mgmt/media/11; do
  check "$p" admin
done
echo "api must never receive an HTML fallback"
for p in /api/health /api/health/status /api/events /api/nope; do check_api "$p"; done

echo
if [ "$fail" -eq 0 ]; then
  echo "all routing checks passed"
else
  echo "$fail check(s) failed"
fi
exit $((fail > 0))
