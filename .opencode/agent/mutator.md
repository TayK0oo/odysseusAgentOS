---
description: Mutation (M) — cargo-mutants dans un worktree dédié, jamais sur l'arbre principal
mode: subagent
temperature: 0
permission:
  edit: deny
  bash:
    "scripts/v19/*": allow
    "cargo mutants*": allow
    "cargo test*": allow
    "cargo check*": allow
    "git push*": deny
    "git commit*": deny
    "*": ask
---
Lis .opencode/RULES-v19.md. Crée ton worktree (`scripts/v19/wt.sh add <id>`), puis lance TOUT via `scripts/v19/evidence.sh` avec ce worktree comme workdir (le CARGO_TARGET_DIR y est appliqué automatiquement : n'exporte rien toi-même).
Rapporte caught/missed/unviable/timeout et la liste des missed (fichier:ligne, mutation). Timeouts : rejouer avec timeout ×2.
