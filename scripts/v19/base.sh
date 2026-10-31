#!/usr/bin/env bash
# base.sh — fige la baseline R2 (HEAD + status). Appelé une seule fois, en tête de /v19-wave0.
# MAIN : racine du worktree D'EXÉCUTION (pas le worktree principal du dépôt — la campagne vit dans un worktree lié).
set -euo pipefail
MAIN=$(git rev-parse --show-toplevel); cd "$MAIN"
EV=docs/omega/reports/evidence-v19; mkdir -p "$EV"
# One-shot : ré-figer en cours de campagne rendrait scope-check aveugle (les fichiers des agents entreraient dans la baseline).
if [ -e "$EV/BASE_HEAD" ] || [ -e "$EV/BASE_STATUS" ]; then
  echo "baseline déjà figée : $(cat "$EV/BASE_HEAD" 2>/dev/null || echo "BASE_HEAD absent, BASE_STATUS présent") — base.sh est one-shot." >&2
  echo "Nouvelle campagne : un humain supprime BASE_HEAD et BASE_STATUS à la main." >&2; exit 1; fi
git rev-parse HEAD > "$EV/BASE_HEAD"
git status --porcelain > "$EV/BASE_STATUS"
echo "baseline figée : $(cat "$EV/BASE_HEAD") ($(wc -l < "$EV/BASE_STATUS") lignes de status)"
