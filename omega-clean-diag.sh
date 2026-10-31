#!/usr/bin/env bash
# omega-clean-diag.sh [--tests]
# Diagnostic LECTURE SEULE : tranche les contradictions du rapport /clean-master.
# N'écrit rien dans le projet, n'affiche AUCUNE valeur de secret (chemins et comptes seulement).
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2
h() { printf '\n== %s\n' "$*"; }

h "1. GIT : commits, branches, worktrees"
echo "HEAD $(git rev-parse --short HEAD) ; commits (toutes refs) : $(git rev-list --all --count)"
echo "branches locales : $(git for-each-ref refs/heads | wc -l) ; distantes : $(git for-each-ref refs/remotes | wc -l) ; tags : $(git tag | wc -l)"
echo "-- commits présents sur une branche mais ABSENTS de HEAD (travail hors HEAD) :"
git for-each-ref --format='%(refname:short)' refs/heads | while read -r b; do
  n=$(git rev-list --count HEAD.."$b" 2>/dev/null || echo "?"); [ "$n" != "0" ] && echo "   $b : +$n"; done | head -30
echo "-- worktrees :"; git worktree list | sed 's/^/   /'
echo "-- ce que 'git worktree prune' retirerait (simulation, rien n'est fait) :"; git worktree prune --dry-run -v 2>&1 | sed 's/^/   /'

h "2. NON SUIVIS : entrées 'git status' contre fichiers réels"
echo "entrées ?? (dossiers regroupés) : $(git status --porcelain | grep -c '^??')"
echo "fichiers non suivis et non ignorés : $(git ls-files -o --exclude-standard | wc -l)"
echo "-- par dossier (2 niveaux), en nombre de fichiers :"
git ls-files -o --exclude-standard | awk -F/ '{k=(NF>1)?$1"/"$2:$1; c[k]++} END{for(k in c) print "   " c[k], k}' | sort -rn | head -25
echo "-- extensions les plus fréquentes :"
git ls-files -o --exclude-standard | awk -F. 'NF>1{print $NF}' | sort | uniq -c | sort -rn | head -10 | sed 's/^/   /'
echo "-- les entrées ?? elles-mêmes :"; git status --porcelain | grep '^??' | cut -c4- | head -90 | sed 's/^/   /'

h "3. .gitignore et fichiers d'environnement (contenu JAMAIS lu)"
for p in omega/target omega/ui/node_modules omega/ui/dist target node_modules; do
  [ -e "$p" ] || continue
  r=$(git check-ignore -v "$p" 2>/dev/null)
  echo "   $p : $([ -n "$r" ] && echo 'IGNORÉ' || echo 'NON IGNORÉ') ; fichiers suivis dedans : $(git ls-files "$p" | wc -l)"
done
find . \( -name .git -o -name node_modules -o -name target \) -prune -o -name '.env*' -print 2>/dev/null | while read -r f; do
  echo "   $f : suivi=$(git ls-files --error-unmatch "$f" >/dev/null 2>&1 && echo OUI || echo non) ; ignoré=$(git check-ignore -q "$f" && echo OUI || echo non) ; commits touchant ce chemin=$(git log --all --oneline -- "$f" | wc -l)"
done
echo "-- fichiers contenant un motif de secret (CHEMINS seulement, hors target/node_modules) :"
grep -rIlE '(AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----|ghp_[A-Za-z0-9]{36}|sk-[A-Za-z0-9]{20,}|xox[baprs]-[A-Za-z0-9-]{10,})' \
  --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules --exclude-dir='mutants.out*' . 2>/dev/null | head -20 | sed 's/^/   /'

h "4. Dispositif v19 : suivi ou non ?"
echo "   scripts/v19 suivis : $(git ls-files scripts/v19 | wc -l) ; non suivis : $(git ls-files -o --exclude-standard scripts/v19 | wc -l)"
echo "   .opencode suivis : $(git ls-files .opencode | wc -l) ; non suivis : $(git ls-files -o --exclude-standard .opencode | wc -l)"
echo "   manifest.sh : $([ -x scripts/v19/manifest.sh ] && echo présent || echo ABSENT) ; agent auditor : $([ -e .opencode/agent/auditor.md ] && echo présent || echo ABSENT)"

h "5. mutants.out* : qui existe, taille, et la baseline qui s'y réfère"
find . \( -name .git -o -name node_modules -o -name target \) -prune -o -maxdepth 3 -type d -name 'mutants.out*' -print 2>/dev/null | head -60 | while read -r d; do
  echo "   $d : $(du -sk "$d" 2>/dev/null | cut -f1) Ko ; outcomes.json=$([ -e "$d/outcomes.json" ] && echo oui || echo non) ; missed.txt=$([ -e "$d/mutants.out/missed.txt" -o -e "$d/missed.txt" ] && echo oui || echo non)"; done
echo "   total dossiers : $(find . \( -name .git -o -name node_modules -o -name target \) -prune -o -type d -name 'mutants.out*' -print 2>/dev/null | wc -l)"

h "6. ADR : numérotation et IMPACT d'une renumérotation"
ls docs/omega/adr 2>/dev/null | sed 's/^/   /'
for n in 0017 0018 0022 0024; do
  echo "   références à ADR-$n hors du dossier adr : $(grep -rIl --exclude-dir=.git --exclude-dir=target --exclude-dir=node_modules --exclude-dir=adr -e "ADR-$n" . 2>/dev/null | wc -l) fichiers"; done

h "7. Agents OpenCode : 'doublons' réels ou ensembles distincts ?"
a=$(ls .opencode/agent 2>/dev/null | sort); b=$(ls .opencode/agents 2>/dev/null | sort)
echo "   .opencode/agent  : $(echo "$a" | grep -c .) fichiers ; .opencode/agents : $(echo "$b" | grep -c .) fichiers"
echo "   noms communs aux deux dossiers : $(comm -12 <(echo "$a") <(echo "$b") | wc -l)"
comm -12 <(echo "$a") <(echo "$b") | while read -r f; do
  s1=$(sha1sum ".opencode/agent/$f" | cut -c1-10); s2=$(sha1sum ".opencode/agents/$f" | cut -c1-10)
  echo "      $f : $([ "$s1" = "$s2" ] && echo identique || echo CONTENU DIFFÉRENT)"; done | head -20
G=${XDG_CONFIG_HOME:-$HOME/.config}/opencode
echo "   global agent : $(ls "$G/agent" 2>/dev/null | wc -l) ; global agents : $(ls "$G/agents" 2>/dev/null | wc -l) ; auth.json présent : $([ -e "$G/auth.json" ] && echo oui || echo non) (droits : $(stat -c %a "$G/auth.json" 2>/dev/null || echo n/a))"

h "8. Dossiers inconnus du rapport (contenu non détaillé : noms et types seulement)"
for d in obsidian-vault companion archive; do
  [ -d "$d" ] || continue
  echo "   $d : $(du -sk "$d" | cut -f1) Ko ; fichiers : $(find "$d" -type f | wc -l) ; suivis git : $(git ls-files "$d" | wc -l) ; modifié le : $(date -r "$d" +%F)"
  find "$d" -maxdepth 1 | head -12 | sed 's/^/      /'; done

h "9. Rapports RAPPORT-v*.md"
for f in docs/omega/reports/RAPPORT-v*.md; do [ -e "$f" ] || continue
  echo "   $f : $(wc -l < "$f") lignes ; suivi=$(git ls-files --error-unmatch "$f" >/dev/null 2>&1 && echo OUI || echo non) ; cité ailleurs : $(grep -rIl --exclude-dir=.git --exclude-dir=target -F "$(basename "$f")" . 2>/dev/null | grep -vxF "./$f" | wc -l) fichiers"; done

if [ "${1:-}" = "--tests" ]; then
  h "10. Tests par binaire (compilation avec CARGO_TARGET_DIR=/tmp/omega-clean-target)"
  CARGO_TARGET_DIR=/tmp/omega-clean-target cargo test --workspace -- --list 2>&1 \
    | awk '/Running|Doc-tests/{name=$0} /: test$/{c[name]++; t++} END{for(n in c) print "   " c[n], n; print "   TOTAL listé :", t+0}' | sort -rn | head -60
  echo "   tests #[ignore] dans le code : $(grep -rIn --include=*.rs -e '#\[ignore' . 2>/dev/null | grep -v '/target/' | wc -l)"
fi
echo; echo "Diagnostic terminé : rien n'a été modifié."
