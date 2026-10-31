#!/usr/bin/env bash
# omega-commit-preview.sh — APERÇU LECTURE SEULE du plan de commit. Ne modifie ni l'index ni le working tree.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
OUT=${1:-/tmp/omega-commit-lists}; mkdir -p "$OUT"; rm -f "$OUT"/*.txt

VIGN='(^|/)(mutants\.out[^/]*|node_modules|target|dist)/|(^|/)\.env($|\.local$)|auth\.json$|\.opencode/v19-wt-root$'
PLAN=(
 "C0-gitignore|.gitignore"
 "C1-bundle|omega-v19-bootstrap.sh omega-clean-setup.sh omega-clean-diag.sh omega-commit-preview.sh .opencode scripts hooks plugins"
 "C2-adr|docs/omega/adr"
 "C3-production|:(glob)omega/crates/*/src/**"
 "C4-tests|:(glob)omega/crates/*/tests/**"
 "C5-rapports|:(glob)docs/omega/reports/RAPPORT-*.md docs/omega/reports/MUTATION-SURVIVORS-BASELINE.md"
 "C6-evidence|docs/omega/reports/evidence-v19 docs/omega/reports/clean"
 "C7-docs-plan|PLAN-MAITRE.md docs"
)
SECRETRE='(AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----|ghp_[A-Za-z0-9]{36}|sk-[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{10,})'

echo "HEAD $(git rev-parse --short HEAD) sur $(git branch --show-current) ; index déjà stagé : $(git diff --cached --name-only | wc -l) fichiers"
mapfile -t DELETED < <(git ls-files -d)
mapfile -t ALL < <(git ls-files -m -o --exclude-standard | sort -u)
declare -A taken; for f in "${DELETED[@]}"; do taken["$f"]=del; done
IGN=(); for f in "${ALL[@]}"; do [ -n "${taken[$f]:-}" ] && continue; if [[ "$f" =~ $VIGN ]]; then IGN+=("$f"); taken["$f"]=ign; fi; done

echo; echo "== À NE PAS COMMITER (régénérable ou sensible) : ${#IGN[@]} fichiers → ajouter au .gitignore"
printf '%s\n' "${IGN[@]}" | sed -E 's#^(.*(mutants\.out[^/]*|node_modules|target|dist)/).*#\1#; s#^(.*/)?(\.env[^/]*|auth\.json|v19-wt-root)$#\1\2#' | sort | uniq -c | sort -rn | head -12 | sed 's/^/   /'
echo "== FICHIERS SUIVIS SUPPRIMÉS SUR DISQUE (jamais ajoutés automatiquement) : ${#DELETED[@]}"; printf '%s\n' "${DELETED[@]}" | head -10 | sed 's/^/   /'

total=0; IGNALL=0
for g in "${PLAN[@]}"; do
  name=${g%%|*}; specs=${g#*|}
  mapfile -t M < <(git ls-files -m -o --exclude-standard -- $specs 2>/dev/null | sort -u)
  sel=(); for f in "${M[@]}"; do [ -n "$f" ] && [ -z "${taken[$f]:-}" ] && { sel+=("$f"); taken["$f"]=$name; }; done
  n=${#sel[@]}; total=$((total+n))
  mapfile -t IG < <(git ls-files -o -i --exclude-standard -- $specs 2>/dev/null | sort -u)
  igs=(); for f in "${IG[@]}"; do [ -n "$f" ] && [ -z "${taken[$f]:-}" ] && ! [[ "$f" =~ $VIGN ]] && { igs+=("$f"); taken["$f"]=ignoré; }; done
  echo; echo "== $name : $n fichiers"
  if [ "${#igs[@]}" -gt 0 ]; then
    IGNALL=$((IGNALL+${#igs[@]}))
    echo "   !! ${#igs[@]} fichiers IGNORÉS par .gitignore dans ce périmètre (jamais commités tel quel) :"
    git check-ignore -v -- "${igs[0]}" 2>/dev/null | sed 's/^/      règle : /'
    printf '%s\n' "${igs[@]}" | head -4 | sed 's/^/      · /'
    echo "      → corriger la règle (exception du type !/chemin/), pas git add -f"
  fi
  [ "$n" -eq 0 ] && continue
  printf ':(literal)%s\n' "${sel[@]}" > "$OUT/$name.txt"
  sz=$(printf '%s\0' "${sel[@]}" | xargs -0 stat -c '%s' 2>/dev/null | awk '{s+=$1} END{printf "%.1f", s/1048576}')
  echo "   taille : ${sz} Mo ; modifiés (suivis) : $(printf '%s\n' "${sel[@]}" | xargs -d '\n' git ls-files -m -- 2>/dev/null | wc -l) ; nouveaux : $(printf '%s\n' "${sel[@]}" | xargs -d '\n' git ls-files -o --exclude-standard -- 2>/dev/null | wc -l)"
  big=$(printf '%s\0' "${sel[@]}" | xargs -0 stat -c '%s %n' 2>/dev/null | awk '$1>1048576{print "   > 1 Mo : " $2 " (" int($1/1048576) " Mo)"}' | head -5); [ -n "$big" ] && echo "$big"
  sec=$(printf '%s\0' "${sel[@]}" | xargs -0 grep -IlE "$SECRETRE" 2>/dev/null | head -5); [ -n "$sec" ] && echo "$sec" | sed 's/^/   MOTIF DE SECRET (chemin seulement) : /'
  printf '%s\n' "${sel[@]}" | head -6 | sed 's/^/   · /'; [ "$n" -gt 6 ] && echo "   · … (liste complète : $OUT/$name.txt)"
done

UNC=(); for f in "${ALL[@]}"; do [ -z "${taken[$f]:-}" ] && UNC+=("$f"); done
echo; echo "== NON COUVERTS par aucun groupe : ${#UNC[@]} fichiers (à classer avant de commiter)"
printf '%s\n' "${UNC[@]}" | head -30 | sed 's/^/   /'
echo; echo "Bilan : ${#ALL[@]} fichiers modifiés/non suivis ; répartis : $total ; à ignorer : ${#IGN[@]} ; non couverts : ${#UNC[@]} ; supprimés : ${#DELETED[@]} ; PRÉCIEUX MAIS IGNORÉS : $IGNALL"
echo "Listes exactes écrites dans $OUT (hors dépôt). Commit d'un groupe : git add --pathspec-from-file=$OUT/<groupe>.txt && git status --short | head"
echo "Rien n'a été modifié dans le dépôt."
