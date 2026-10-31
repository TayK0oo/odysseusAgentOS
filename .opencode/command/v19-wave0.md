---
description: Vague 0 — gardien d'état (mesures réelles, aucune hypothèse)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Exécute via scripts/v19/evidence.sh : git rev-parse HEAD ; git status --porcelain ; git worktree list ; cargo check/test/fmt --check/clippy --all-targets
IMPORTANT (rev8-armée, preuve en main) : le workspace Cargo vit dans omega/ — la racine du dépôt n'a AUCUN Cargo.toml (`could not find Cargo.toml`). Toute commande cargo passe workdir=omega : `scripts/v19/evidence.sh <agent> <sujet> omega <to> -- cargo ...` (idem wc -l sur les src). Les echo/git restent wd=".".
sur -p omega-context -p omega-memory -p omega-policy -p omega-tools (tests comptés par crate) ; cargo mutants --version ; rustc --version ; nproc ; wc -l des fichiers src d'omega-tools ;
`cargo test -p omega-tools -- --list`. Compare aux hypothèses : HEAD 7f6386f9…341 ; 271 = 112+91+68 ; omega-tools 52 tests ; total attendu 323 ; 1667 lignes.
Mesure aussi, par crate, `cargo test -p <crate> -- --list` (omega-context : tests réels ? un inventaire externe dit 0 test et 94 survivants, nos rapports disent 112 tests TERMINÉ VÉRIFIÉ : trancher par commande), liste les mutants.out-* présents et les fichiers de tests non suivis.
COMMENCE par `scripts/v19/base.sh` (fige BASE_HEAD et BASE_STATUS ; l'orchestrator n'a pas le droit d'écrire lui-même), AVANT toute autre commande. Divergence → STOP et rapport d'écart.
