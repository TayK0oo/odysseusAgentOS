#!/usr/bin/env bash
# Vérifie R1/R2 : aucun commit depuis BASE_HEAD, aucune modif hors zones autorisées (hors baseline).
set -euo pipefail
MAIN=$(git rev-parse --show-toplevel); cd "$MAIN"
EV=docs/omega/reports/evidence-v19
BASE=$(cat "$EV/BASE_HEAD" 2>/dev/null || true)
[ -n "$BASE" ] && [ -f "$EV/BASE_STATUS" ] || { echo "baseline absente : lancer scripts/v19/base.sh (via /v19-wave0)"; exit 2; }
fail=0
if [ -n "$(git rev-list "$BASE"..HEAD)" ]; then echo "VIOLATION R2 : nouveau commit"; fail=1; fi
ALLOWED='^(omega/crates/omega-(tools|router|server|providers|search|types|registry|core|sandbox|app|protocol|plugin-api|mcp|adapters|config|otel|python-bridge|harness)/tests/|docs/omega/(reports|adr)/)'
while IFS= read -r line; do
  grep -qxF -- "$line" "$EV/BASE_STATUS" && continue
  path=${line:3}; path=${path##* -> }
  case "$path" in *proptest-regressions) continue;; esac
  echo "$path" | grep -Eq "$ALLOWED" || { echo "VIOLATION R1 : $line"; fail=1; }
done < <(git status --porcelain)
[ $fail -eq 0 ] && echo "scope OK"; exit $fail
