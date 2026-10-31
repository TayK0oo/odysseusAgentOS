#!/usr/bin/env bash
# bg.sh start <nom> <agent> <sujet> <workdir> <timeout_s> -- <cmd...> | status [nom] | wait <nom> <max_s>
# Lance evidence.sh en arrière-plan (commande plus longue que la limite d'un appel bash), survit à la fin de l'appel.
set -uo pipefail
MAIN=$(git rev-parse --show-toplevel)
# rev8 blindage 1 : invariant au cwd de l'agent (evidence.sh et D ci-dessous résolvent après cd).
cd "$MAIN" || { echo "bg.sh: cd $MAIN impossible" >&2; exit 1; }
D="$MAIN/docs/omega/reports/evidence-v19/bg"; mkdir -p "$D"
state() { # $1=nom
  local pid; pid=$(cat "$D/$1.pid" 2>/dev/null || echo 0)
  if [ "$pid" -gt 0 ] && kill -0 "$pid" 2>/dev/null; then echo "$1 : running (pid $pid)"
  else echo "$1 : done — $(tail -n1 "$D/$1.out" 2>/dev/null || echo 'sortie absente')"; fi; }
case "${1:-}" in
  start) name=${2:?nom}
    # rev8 blindage 2 : name assaini (il colle dans les chemins $D/$name.*) — pas de slash ni de chaîne vide.
    case "$name" in */*|"") echo "bg.sh: name invalide (slash ou vide) : '$name'" >&2; exit 2 ;; esac
    shift 2
    [ ! -e "$D/$name.pid" ] || { echo "déjà lancé : $name (status $name)" >&2; exit 1; }
    ( setsid nohup "$MAIN/scripts/v19/evidence.sh" "$@" > "$D/$name.out" 2>&1 & echo $! > "$D/$name.pid" )
    sleep 0.3; echo "lancé : $name (pid $(cat "$D/$name.pid"))" ;;
  status) if [ -n "${2:-}" ]; then state "$2"; else for f in "$D"/*.pid; do [ -e "$f" ] && state "$(basename "$f" .pid)"; done; fi ;;
  wait) name=${2:?nom}; max=${3:-60}; t=0
    while [ $t -lt "$max" ]; do state "$name" | grep -q running || break; sleep 2; t=$((t+2)); done; state "$name" ;;
  *) echo "usage: bg.sh start <nom> <agent> <sujet> <wd> <to> -- <cmd...> | status [nom] | wait <nom> <max_s>"; exit 2 ;;
esac
