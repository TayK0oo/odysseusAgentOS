#!/usr/bin/env bash
# wt.sh add <id> | sync <id> | clean   — uniquement les worktrees v19-*
set -euo pipefail
# MAIN : racine du worktree d'exécution (worktree lié de campagne, pas le worktree principal du dépôt).
MAIN=$(git rev-parse --show-toplevel); cd "$MAIN"
SCOPE=omega/crates/omega-tools
# Racine des worktrees, par ordre de priorité : $V19_WT_ROOT, puis le fichier .opencode/v19-wt-root (une ligne : le chemin),
# puis le dossier parent du worktree d'exécution. Le fichier évite toute variable d'env, qui ne persiste pas entre deux commandes d'agent.
wtroot() { if [ -n "${V19_WT_ROOT:-}" ]; then printf '%s' "$V19_WT_ROOT"
  elif [ -s "$MAIN/.opencode/v19-wt-root" ]; then head -n1 "$MAIN/.opencode/v19-wt-root"
  else dirname "$MAIN"; fi; }
wtdir() { echo "$(wtroot)/v19-$1"; }
# Copie les non-suivis d'omega-tools du principal vers le worktree ET retire du worktree ceux qui n'existent plus dans le principal
# (un test rejeté puis supprimé ne doit plus polluer les preuves R8/R9).
sync_files() {
  local d=$1 main_list f
  main_list=$(git ls-files --others --exclude-standard -- "$SCOPE")
  while IFS= read -r f; do [ -n "$f" ] || continue
    mkdir -p "$d/$(dirname "$f")"; cp "$f" "$d/$f"; done <<< "$main_list"
  while IFS= read -r f; do [ -n "$f" ] || continue
    printf '%s\n' "$main_list" | grep -qxF -- "$f" || { rm -f "$d/$f"; echo "retiré du worktree (absent du principal) : $f"; }
  done < <(git -C "$d" ls-files --others --exclude-standard -- "$SCOPE")
}
case "${1:-}" in
  add)  d=$(wtdir "${2:?id}"); git worktree add --detach "$d" HEAD >/dev/null; sync_files "$d"
        echo "worktree : $d (CARGO_TARGET_DIR appliqué automatiquement par evidence.sh)" ;;
  sync) d=$(wtdir "${2:?id}"); [ -d "$d" ] || { echo "worktree absent : $d" >&2; exit 2; }
        sync_files "$d"; echo "synchronisé : $d" ;;
  clean)
    bad=0
    # Seuls les worktrees dont le NOM DE DOSSIER commence par v19- (pas un parent), jamais le worktree d'exécution.
    mapfile -t WTS < <(git worktree list --porcelain | sed -n 's/^worktree //p' | awk -F/ '$NF ~ /^v19-/' | grep -vxF -- "$MAIN" || true)
    for w in "${WTS[@]}"; do
      # Garde : un test présent dans le worktree mais absent/différent dans l'arbre principal = non publié.
      while read -r f; do
        [ -f "$MAIN/$f" ] && cmp -s "$w/$f" "$MAIN/$f" || { echo "NON PUBLIÉ : $w/$f" >&2; bad=1; }
      done < <(git -C "$w" ls-files --others --exclude-standard -- "$SCOPE" )
    done
    [ $bad -eq 0 ] || { echo "clean annulé : copier ces fichiers dans l'arbre principal ou les écarter explicitement" >&2; exit 1; }
    for w in "${WTS[@]}"; do git worktree remove --force "$w"; done; echo "worktrees v19-* supprimés" ;;
  *) echo "usage: wt.sh add <id> | sync <id> | clean"; exit 2 ;;
esac
