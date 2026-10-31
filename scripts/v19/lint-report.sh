#!/usr/bin/env bash
# lint-report.sh <rapport.md> : mots interdits (mots entiers), [E-nnn] valides.
set -uo pipefail
MAIN=$(git rev-parse --show-toplevel)
R=${1:?rapport.md}; IDX="$MAIN/docs/omega/reports/evidence-v19/index.csv"; fail=0
B='(^|[^[:alpha:]])'; E='([^[:alpha:]]|$)'
if grep -inE "${B}(probablement|semble|semblent|semblait|partiel(s|le|les|lement)?|environnement-dépendant|devrai(t|ent)|(à|À|a) peu (près|pres|PRÈS))${E}" "$R"; then
  echo "R4 : mot interdit"; fail=1; fi
for e in $(grep -oE "E-[0-9]{3}" "$R" | sort -u); do grep -q "^$e," "$IDX" || { echo "[$e] absent de index.csv"; fail=1; }; done
nums=$(grep -vE "^\s*$" "$R" | grep -E "[0-9]{2,}" | grep -vE "E-[0-9]{3}" | wc -l)
echo "lignes numériques sans [E-nnn] : $nums (à justifier ou sourcer)"
[ $fail -eq 0 ] && echo "lint OK"; exit $fail
