#!/usr/bin/env bash
# evidence.sh <agent> <sujet> <workdir> <timeout_s> -- <commande...>
# Exécute la commande, archive la sortie brute complète (.raw) + résumé (.md), imprime E-nnn.
set -uo pipefail
if [ $# -lt 6 ] || [ "$5" != "--" ]; then
  echo "usage: evidence.sh <agent> <sujet> <workdir> <timeout_s> -- <cmd...>" >&2; exit 2; fi
agent=$1; sujet=$(printf '%s' "$2" | tr -d '"\n,'); to=$4
wd=$(cd "$3" 2>/dev/null && pwd) || { echo "workdir introuvable : $3" >&2; exit 2; }
shift 5
MAIN=$(git rev-parse --show-toplevel)
EV="$MAIN/docs/omega/reports/evidence-v19"; IDX="$EV/index.csv"; mkdir -p "$EV"
# Identifiant unique SANS verrou long : création atomique du fichier (noclobber).
n=$(ls "$EV"/E-*.md 2>/dev/null | wc -l)
while :; do n=$((n+1)); id=$(printf "E-%03d" "$n"); ( set -o noclobber; : > "$EV/$id.md" ) 2>/dev/null && break; done
# Isolation cargo : tout workdir v19-* a son propre target.
case "$(basename "$wd")" in v19-*) export CARGO_TARGET_DIR="$wd/target-v19";; esac
start=$(date -u +%FT%TZ)
( cd "$wd" && timeout "$to" "$@" ) >"$EV/$id.raw" 2>&1; code=$?
kv=$(grep -E "test result|caught|missed|unviable|timeout|^[0-9a-f]{40}$" "$EV/$id.raw" | tail -5 | tr '\n' ';' | tr -d ',"')
{
  echo "# $id"; echo "- agent: $agent"; echo "- sujet: $sujet"; echo "- workdir: $wd"
  echo "- commande: $*"; echo "- horodatage: $start"; echo "- exit: $code"
  echo "- sortie complète: $id.raw"
  echo '```'; tail -n 200 "$EV/$id.raw"; echo '```'
} > "$EV/$id.md"
# Verrou bref : uniquement l'ajout de la ligne d'index.
( flock 9; [ -s "$IDX" ] || echo "id,agent,sujet,exit,cle_valeur" > "$IDX"
  echo "$id,$agent,\"$sujet\",$code,\"$kv\"" >> "$IDX" ) 9>"$EV/.index.lock"
echo "$id exit=$code"
