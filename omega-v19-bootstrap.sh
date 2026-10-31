#!/usr/bin/env bash
# Bootstrap v19 — AgentOS Omega / omega-tools sur OpenCode
# Usage : depuis la racine du dépôt (ICI : worktree de campagne odysseusAgentOS-omega) : bash omega-v19-bootstrap.sh
# N'écrase rien d'existant (sauf fichiers .opencode/*v19*). Aucun commit.
# rev8 : + garde d'idempotence V19_FORCE ; + smoke points 7/8/9 ; + scribe RAPPORT-v19.md ; + bg.sh ;
#        + master /v19-master ; + MAIN résolu par `git rev-parse --show-toplevel`
#        (le dépôt de campagne est un worktree lié, le principal est sur une autre branche) ;
#        + 2 blindages bg.sh (cd MAIN, name assaini) ; + tighten smoke 9 (status non vide).
set -euo pipefail

if [ -e .opencode/RULES-v19.md ] && [ -z "${V19_FORCE:-}" ]; then
  echo "Bundle v19 déjà installé : le script écraserait agents, commandes et scripts (modifs ultérieures perdues)." >&2
  echo "Relancer avec V19_FORCE=1 pour réinstaller volontairement." >&2; exit 1; fi
mkdir -p .opencode/agent .opencode/command scripts/v19 docs/omega/reports/evidence-v19

# ─────────────────────────── RÈGLES COMMUNES ───────────────────────────
cat > .opencode/RULES-v19.md <<'EOF'
# RÈGLES v19 (non négociables)
R1  Zéro modif de production. Écriture permise : omega/crates/omega-tools/tests/** et docs/omega/reports/** (append-only).
R2  Zéro commit/push/prune. Untracked et proptests.proptest-regressions intacts. Seuls les worktrees `v19-*` sont supprimables.
R3  Aucun chiffre sans preuve : toute mesure passe par `scripts/v19/evidence.sh` (produit un [E-nnn]). Aucun chiffre repris d'un ancien rapport.
R4  Statuts autorisés : TERMINÉ VÉRIFIÉ · ÉQUI-PROUVÉ · UNVIABLE-PROUVÉ · INATTEIGNABLE-API-PUBLIQUE · PROTÉGÉ · OUVERT(cause+action).
    Mots interdits : probablement, semble, partiel, environnement-dépendant, devrait, à peu près.
R5  Un worktree par agent qui lance cargo (`scripts/v19/wt.sh add <id>`), son CARGO_TARGET_DIR, son --output. Jamais deux cargo-mutants sur un même arbre.
R6  Aucun agent ne valide sa propre conclusion : passage obligatoire par `verifier`.
R7  ÉQUI-PROUVÉ = raisonnement écrit + recherche exhaustive citée (grep/AST) + échec documenté de l'adversaire.
R8  Un test compte seulement s'il passe sur code sain ET échoue sous le mutant visé (commande rejouable).
R9  Déterminisme : test rejoué 20 fois, en --test-threads=1 et en parallèle. 1 échec = rejeté.
R10 Max 6 agents cargo simultanés, timeout explicite sur toute commande longue.
R11 Sortie de périmètre : annuler les fichiers de l'agent, consigner l'incident (Livrable D).
R12 Invoquer les scripts v19 DIRECTEMENT (`scripts/v19/x.sh args`), sans cd, &&, pipe ni redirection : les permissions matchent la chaîne de commande, toute forme composée retombe en « ask ».
PROTÉGÉ : RECALL_LIMIT = 20 (ADR-0022). Un mutant sur cette constante n'est ni à tuer ni à déclarer équivalent.
Format de retour d'un subagent (JSON) :
{"id","perimetre","commandes":[{"cmd","exit","evidence_id"}],"chiffres":{},"conclusions":[{"enonce","statut","preuve_ids":[]}],"ouverts":[],"fichiers_modifies":[]}
EOF

# ─────────────────────────── AGENTS ───────────────────────────
cat > .opencode/agent/orchestrator.md <<'EOF'
---
description: Orchestrateur v19 — décompose, lance les subagents en parallèle, consolide. N'écrit ni code ni tests.
mode: primary
temperature: 0.1
permission:
  edit: deny
  bash:
    "git push*": deny
    "git commit*": deny
    "git worktree remove*": deny
    "scripts/v19/*": allow
    "*": ask
---
Tu es l'ORCHESTRATEUR. Lis .opencode/RULES-v19.md. Tu délègues aux subagents via l'outil de délégation d'OpenCode (task / @mention ; plusieurs appels dans un même message = parallèle).
Tu n'écris le rapport final qu'à partir de docs/omega/reports/evidence-v19/index.csv.
Après chaque vague : `scripts/v19/scope-check.sh`. Divergence → STOP + rapport d'écart.
EOF

cat > .opencode/agent/reader.md <<'EOF'
---
description: Lecteur (L) — audit lecture seule d'un fichier ou transverse
mode: subagent
temperature: 0.1
permission:
  edit: deny
  bash:
    "scripts/v19/*": allow
    "grep*": allow
    "cargo mutants --list*": allow
    "cargo test*--list*": allow
    "wc*": allow
    "*": deny
---
Lecture seule. Lis .opencode/RULES-v19.md. Livre : inventaire des fonctions (pub/privé, pure/I-O/async), cartographie fonction→tests (grep réel),
fonctions sans test, prévision de mutants, découpe en sous-incréments ≤ 12 tests (incr-J1 jail/resolve, J2 memory, J3 diff/glob/grep, J4 surface/registry/workspace),
couplages environnement. Retour JSON R-format.
EOF

cat > .opencode/agent/mutator.md <<'EOF'
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
EOF

cat > .opencode/agent/tester.md <<'EOF'
---
description: Auteur de tests (T) — un fichier de tests par sous-incrément, jamais de production
mode: subagent
temperature: 0.2
permission:
  edit: allow
  bash:
    "scripts/v19/*": allow
    "cargo test*": allow
    "cargo mutants*": allow
    "git push*": deny
    "git commit*": deny
    "*": ask
---
Lis .opencode/RULES-v19.md. Écris UNIQUEMENT omega/crates/omega-tools/tests/<incr>.rs, directement dans l'arbre principal (zone R1) ; puis `scripts/v19/wt.sh sync <id>` copie le fichier dans ton worktree, où tu prouves R8/R9 via evidence.sh. Test rejeté : supprime ton propre fichier. Pour chaque mutant visé : nom du test, preuve sain=passe,
preuve mutant=échoue, R9 (20 répétitions + parallèle). Interdits : toucher src/, #[cfg(test)] dans src/, rendre une fonction pub.
Mutant atteignable seulement via API non publique → INATTEIGNABLE-API-PUBLIQUE + chemin d'appel analysé, à soumettre au verifier.
EOF

cat > .opencode/agent/verifier.md <<'EOF'
---
description: Vérificateur adverse (V) — tente de réfuter toute conclusion, tue les "équivalents"
mode: subagent
temperature: 0.1
permission:
  edit: deny
  bash:
    "scripts/v19/*": allow
    "cargo test*": allow
    "cargo mutants*": allow
    "grep*": allow
    "*": ask
---
Lis .opencode/RULES-v19.md. Tu ne valides jamais ton propre travail. Pour chaque survivant déclaré équivalent/inatteignable : tente de le tuer (test ou entrée publique).
Succès = preuve rejetée. Dossier unique_tmp_* (omega-memory `unique_tmp_path` ×4 ET omega-tools `unique_tmp_id`, vérifier la duplication de code) :
(1) lire ce qui entre dans la clé, (2) 200 créations concurrentes sous chaque mutant, mesurer les collisions, (3) conclure tué/ÉQUI-PROUVÉ/INATTEIGNABLE/OUVERT chiffré.
Interdit de conclure sur « pas de collision observée » seul.
EOF

cat > .opencode/agent/scribe.md <<'EOF'
---
description: Scribe (S) — append-only dans MUTATION-SURVIVORS-BASELINE.md ; écrit RAPPORT-v19.md
mode: subagent
temperature: 0
permission:
  edit: allow
  bash:
    "scripts/v19/*": allow
    "*": deny
---
Lis .opencode/RULES-v19.md. Ajoute UNIQUEMENT en fin de fichier la section "## omega-tools — BASELINE ET SATURATION (v19)". Ne modifie aucune section existante.
Tu peux aussi écrire (créer ou écraser) docs/omega/reports/evidence-v19/RAPPORT-v19.md avec le texte fourni par l'orchestrateur : aucun autre fichier.
Chaque nombre porte son [E-nnn] extrait de index.csv.
EOF

# ─────────────────────────── COMMANDES ───────────────────────────
cat > .opencode/command/v19-wave0.md <<'EOF'
---
description: Vague 0 — gardien d'état (mesures réelles, aucune hypothèse)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Exécute via scripts/v19/evidence.sh : git rev-parse HEAD ; git status --porcelain ; git worktree list ; cargo check/test/fmt --check/clippy --all-targets
sur -p omega-context -p omega-memory -p omega-policy -p omega-tools (tests comptés par crate) ; cargo mutants --version ; rustc --version ; nproc ; wc -l des fichiers src d'omega-tools ;
`cargo test -p omega-tools -- --list`. Compare aux hypothèses : HEAD 7f6386f9…341 ; 271 = 112+91+68 ; omega-tools 52 tests ; total attendu 323 ; 1667 lignes.
Mesure aussi, par crate, `cargo test -p <crate> -- --list` (omega-context : tests réels ? un inventaire externe dit 0 test et 94 survivants, nos rapports disent 112 tests TERMINÉ VÉRIFIÉ : trancher par commande), liste les mutants.out-* présents et les fichiers de tests non suivis.
COMMENCE par `scripts/v19/base.sh` (fige BASE_HEAD et BASE_STATUS ; l'orchestrator n'a pas le droit d'écrire lui-même), AVANT toute autre commande. Divergence → STOP et rapport d'écart.
EOF

cat > .opencode/command/v19-run.md <<'EOF'
---
description: Campagne complète omega-tools (vagues 1 à 5)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Pré-requis : /v19-wave0 sans écart. Exécute :
V1 : 8× reader (un fichier de omega/crates/omega-tools/src chacun) + reader transverse + verifier réconciliateur (9/10 tests io_accessors, 15/16 serde_defaults, missed 15/7 vs 6+3, "7 unviable"/"4 équipotents", unique_tmp_*).
     Porte : chaque mutant de `cargo mutants --list -p omega-tools` rattaché à une fonction inventoriée.
V2 : mutator ×N (shardé selon `--list`, max 6 en parallèle), verifier contrôleur (caught+missed+unviable+timeout = total).
V3 : tester ×N (incr-J1..J4, un fichier de tests chacun, écrit dans l'arbre principal puis `wt.sh sync <id>` et preuves dans le worktree).
V4 : mutator rejoue ; reader prouveur d'équivalence ; verifier adversaire (max 2 itérations) ; dossier unique_tmp_*.
V5 : scope-check, fmt, clippy (seul warning engine.rs:248), tests par crate (attendu 323 + tests ajoutés), scribe, `scripts/v19/lint-report.sh`, `scripts/v19/wt.sh clean`.
Lance les Task d'une même vague dans un seul message. Rapport final au format v18/v19 (§1 synthèse, A, B, D, F, C+E), uniquement depuis index.csv.
EOF

cat > .opencode/command/v19-lint.md <<'EOF'
---
description: Lint de cohérence du rapport final
agent: orchestrator
---
Exécute `scripts/v19/lint-report.sh $ARGUMENTS` et corrige toute divergence avant livraison.
EOF

cat > .opencode/command/v19-smoke.md <<'EOF'
---
description: Test de fumée du dispositif (à lancer AVANT /v19-wave0 et /v19-run)
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Vérifie, avec une preuve par commande ou par résultat de Task, chaque point (OK/KO + cause) :
1. Les 5 subagents (reader, mutator, tester, verifier, scribe) sont reconnus et invocables via Task.
2. 3 Task `reader` lancées dans UN SEUL message s'exécutent bien en parallèle (relever les horodatages).
3. Les hooks gsd-write-guard / gsd-secret-read-guard laissent écrire un fichier de test jetable dans omega/crates/omega-tools/tests/ et dans docs/omega/reports/evidence-v19/ (puis le supprimer).
4. Les permissions effectives (inclure un test de composition : `scripts/v19/scope-check.sh ; git commit --help` ne doit PAS passer sans confirmation ni refus — sinon R12 seule ne protège pas, le signaler en KO) : `scripts/v19/evidence.sh <args> -- cargo --version` (avec arguments) passe sans confirmation ; le `tester` écrit sans confirmation, l'`orchestrator` ne peut pas écrire, `git commit` et `git push` sont refusés.
5. Les plugins (rtk.ts, oh-my-openagent) : (a) se chargent sans erreur (distinguer « plugin non chargé » de « sortie altérée ») ; (b) ne modifient pas la sortie de `scripts/v19/evidence.sh` : comparer le .raw avec la sortie réelle d'une commande connue (`cargo --version`). Ne renomme ni ne modifie aucun plugin global.
6. Aucun agent de oh-my-openagent n'entre en conflit de nom ou de rôle avec orchestrator/reader/mutator/tester/verifier/scribe.
7. `scripts/v19/wt.sh add smoke` crée un worktree inscriptible (sinon : le signaler en KO et demander à l'HUMAIN d'écrire le chemin d'un dossier inscriptible dans `.opencode/v19-wt-root` (une ligne) ; l'agent ne l'écrit pas lui-même, l'écriture déclencherait une confirmation. Pas de variable d'environnement. Puis relancer ce point), `evidence.sh` y tourne avec CARGO_TARGET_DIR propre, puis `wt.sh clean` le supprime.
8. Un subagent renvoie bien le JSON du format de RULES-v19.md avec le modèle effectif.
9. Limite d'un appel bash : `scripts/v19/evidence.sh orch limite-bash . 140 -- sleep 130` ; si l'appel est coupé avant la fin, noter la limite L mesurée (< 130 s). Puis `scripts/v19/bg.sh start smoke orch bg-test . 10 -- bash -c 'sleep 3; echo ok'`, `scripts/v19/bg.sh wait smoke 30`, puis `scripts/v19/bg.sh status smoke` DOIT afficher « done — E-nnn exit=... » non vide : un status « done » sans identifiant d'évidence ni exit-code est KO (tighten rev8 — un faux done est plus grave qu'un KO documenté).
Conclus : "Dispositif prêt : OUI/NON" avec la liste des KO, et enregistre le verdict : `scripts/v19/evidence.sh orch SMOKE-OK . 5 -- echo ok` si tout est OK, sinon `scripts/v19/evidence.sh orch SMOKE-KO . 5 -- echo ko`. Ne lance aucune vague.
EOF

cat > .opencode/command/v19-master.md <<'EOF'
---
description: Campagne maître v19 — omega-tools, pilotée par les outils, avec reprise, checkpoints et garde-fous
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Tu es l'orchestrateur. Objectif : mener la campagne omega-tools (audit → baseline mutation → saturation) et solder les contradictions héritées, en utilisant au maximum les agents, outils MCP, LSP et plugins RÉELLEMENT disponibles, sans jamais enfreindre R1–R12. Retours des subagents : JSON de RULES-v19.md.

## A. Portes d'entrée (dans cet ordre, STOP si échec)
A1 Reprise : lis docs/omega/reports/evidence-v19/index.csv s'il existe. Les checkpoints sont les lignes dont le sujet commence par « CP: ». Reprends après le dernier CP valide ; ne refais jamais une étape dont l'évidence existe ; une évidence incohérente ou sans résultat se refait.
A2 Smoke : index.csv doit contenir un sujet « SMOKE-OK » postérieur à tout « SMOKE-KO ». Sinon STOP et demande à l'humain de lancer /v19-smoke.
A3 Baseline : si BASE_HEAD est absent, `scripts/v19/base.sh` est ta toute première commande (R2).
A4 Éléments bloqués : tout élément marqué « bloqué / autorisation humaine » (ex. P3) le reste. Ne le démarre jamais, mentionne-le au Livrable D.

## B. Cartographie des capacités (CP:CAPS)
Inventorie par appels réels, jamais de mémoire : agents (primaires et subagents) avec leurs descriptions (dont gsd-* et ceux de oh-my-openagent) ; serveurs MCP et leurs outils, état connecté/échec (workspace-mcp, webclaw, graphify, gsd…) ; LSP rust-analyzer (références, définitions, diagnostics) si exposé comme outil ; skills et hooks actifs ; modèles (`opencode models`) et modèle effectif par agent.
Produis la table Capacité | Disponible (preuve) | Usage prévu | Limite ; enregistre-la via `scripts/v19/evidence.sh orch CP:CAPS . 10 -- echo "<table>"`. Applique ensuite cette matrice, en lisant la description réelle de chaque outil avant de t'en servir :
- LSP rust-analyzer : find-references pour la cartographie fonction → tests, deuxième source indépendante du grep. Tout désaccord LSP vs grep se tranche par une commande.
- graphify, si connecté et si sa description confirme un graphe de code : graphe d'appels d'omega-tools pour le transverse. Recouper ses nœuds avec `cargo mutants --list -p omega-tools` ; ne jamais le croire seul.
- agents gsd-* : lire la description ; si l'un est vérificateur ou planificateur adapté, l'utiliser comme deuxième vérificateur adverse indépendant (R7). Les hooks gsd-* restent actifs.
- agents oh-my-openagent : renfort read-only/exploration des readers seulement. Jamais d'orchestration ni de sortie du périmètre R1.
- webclaw : uniquement pour une documentation externe nécessaire (flags cargo-mutants, comportement proptest), source citée. Jamais pour établir un chiffre du rapport.
- workspace-mcp : hors périmètre de cette campagne, sauf description contraire. Ne pas l'utiliser.
- todo : plan vivant (une entrée par vague et par sous-incrément), mis à jour à chaque checkpoint.
Un outil absent ou en échec n'arrête pas la campagne : consigne la dégradation au Livrable D (ex. « graphify KO → transverse par grep + LSP »).
Les modèles : n'essaie pas d'en changer. Si le JSON des subagents est invalide ou si les preuves sont peu fiables, propose à l'humain un modèle plus fort pour l'orchestrateur et le verifier.

## C. Intelligence de pilotage
C1 Priorisation par risque : ordre des fichiers = criticité sécurité × nombre de mutants × centralité. resolve() (jail) d'abord, puis memory.rs, puis surface/registry, puis diff/glob/grep/workspace. Sous-incréments J1 à J4 sauf si le lecteur justifie mieux.
C2 Sharding adaptatif : `cargo mutants --list -p omega-tools` donne l'effectif par fichier ; un pilote de 10 mutants mesure les secondes par mutant ; calcule le nombre de shards pour que chaque appel reste sous la limite L d'un appel bash (mesurée au smoke point 9). Toute commande attendue plus longue que L passe par `scripts/v19/bg.sh start|status|wait` : sondage court, jamais d'attente bloquante au-delà de L. Max 6 cargo simultanés (R10).
C3 Triage avant écriture : pour chaque missed, un reader tranche TUABLE-API-PUBLIQUE / INATTEIGNABLE-API-PUBLIQUE / PROTÉGÉ / ÉQUIVALENT-À-PROUVER avant qu'un tester ne s'y attaque. Aucun tester sur un mutant non trié.
C4 Techniques de test imposées au tester : comparaisons (==, <, <=, >, >=) → valeurs aux bornes b−1, b, b+1 ; retour par défaut remplacé → assertion sur la valeur exacte, pas seulement Ok/Err ; opérateurs logiques → cas où chaque opérande décide seul ; arithmétique → valeurs asymétriques (éviter 0, 1 et les symétries) ; I/O → tempdir unique, jamais de chemin partagé. Les 40 proptests existants ne tuent pas forcément les bornes exactes.
C5 Budgets d'itération : ≤ 2 tentatives de test par mutant, puis OUVERT avec cause ; JSON de subagent invalide : 1 relance puis OUVERT ; boucle du verifier ≤ 2.
C6 Hygiène de contexte : un subagent reçoit son périmètre (chemins, mutants, règles), pas l'historique. Tu ne gardes que index.csv et les verdicts. Après chaque vague : checkpoint, scope-check, mise à jour du todo.
C7 Deux sources indépendantes pour tout fait structurel (LSP + grep, graphify + --list). Pour toute équivalence : prouveur et adversaire distincts, plus un agent gsd vérificateur si disponible.
C8 Propositions de production : si un mutant n'est atteignable qu'en changeant la production (visibilité, refactor), ne modifie RIEN. Consigne une PROPOSITION (fichier:ligne, changement minimal, mutants débloqués) pour décision humaine.

## D. Dossiers hérités à solder (par commandes)
D1 omega-context : « 0 test / 94 survivants » (inventaire) contre « 112 tests TERMINÉ VÉRIFIÉ » (rapports) → `cargo test -p omega-context -- --list`, comptes par binaire. La valeur mesurée gagne, l'autre est consignée comme erreur documentaire.
D2 omega-memory : survivants réels contre « 0 survivant » ; dossier unique_tmp_path ×4, et unique_tmp_id d'omega-tools (duplication de code ?) par l'expérience en 3 volets (contenu de la clé, 200 créations concurrentes par mutant, conclusion R4).
D3 Comptes : io_accessors 9 ou 10 tests, serde_defaults 15 ou 16, « missed restants » 15/7 contre 6+3.
D4 Fichiers non suivis (tests omega-memory, PLAN-MAITRE.md, mutants.out-*) : les lister, ne jamais les toucher, signaler le risque qu'ils ne soient pas dans git.

## E. Vagues (une vague = ses Task lancées dans UN SEUL message)
Checkpoint après chaque vague : `scripts/v19/evidence.sh orch CP:W<n> . 10 -- echo "CP W<n> OK <résumé chiffré>"`.
W0 : mesures d'état (cargo check/test/fmt/clippy par crate, wc -l, `--list` des tests, outils). Divergence avec les hypothèses → STOP + rapport d'écart.
W1 : 8 readers (un fichier chacun) + reader transverse (LSP/graphify) + verifier réconciliateur (D1 à D4) + contre-auditeur (chaque mutant listé rattaché à une fonction inventoriée).
W2 : mutators par fichier ou shard dans des worktrees v19-*, bg.sh si > L ; verifier contrôleur : caught + missed + unviable + timeout = total listé.
W3 : triage C3, puis testers J1..Jn (un fichier de tests chacun, écrit dans l'arbre principal, `wt.sh sync <id>`, preuves R8/R9 dans le worktree).
W4 : rejeu mutation ; prouveurs d'équivalence ; adversaires (+ gsd) ; dossier unique_tmp_*.
W5 : scope-check, fmt, clippy (seul warning engine.rs:248), tests recomptés par crate ; scribe : section baseline append-only + rapport final ; lint ; `scripts/v19/wt.sh clean`.

## F. Rapport final
Compose le corps (sections 1 à 6 du format v18/v19) UNIQUEMENT depuis index.csv et les .raw. Le scribe l'écrit dans docs/omega/reports/evidence-v19/RAPPORT-v19.md. Lance `scripts/v19/lint-report.sh docs/omega/reports/evidence-v19/RAPPORT-v19.md` et corrige jusqu'à « lint OK » (≤ 3 tours). Livrable E : état des crates suivantes (omega-router, omega-server, omega-app) avec `cargo mutants --list` mesuré ; dernière ligne « Prochaine étape : votre signal pour auditer <crate> ». Ne démarre aucune autre crate sans ce signal.

## G. Points d'arrêt humains
Smoke KO · divergence en W0 · mutant PROTÉGÉ (RECALL_LIMIT = 20, ADR-0022) · élément bloqué · PROPOSITION de production · toute demande de confirmation d'outil · résultat contradictoire que ne tranche aucune commande.
EOF

# ─────────────────────────── SCRIPTS ───────────────────────────
cat > scripts/v19/base.sh <<'EOF'
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
EOF

cat > scripts/v19/evidence.sh <<'EOF'
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
EOF

cat > scripts/v19/wt.sh <<'EOF'
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
EOF

cat > scripts/v19/scope-check.sh <<'EOF'
#!/usr/bin/env bash
# Vérifie R1/R2 : aucun commit depuis BASE_HEAD, aucune modif hors zones autorisées (hors baseline).
set -euo pipefail
MAIN=$(git rev-parse --show-toplevel); cd "$MAIN"
EV=docs/omega/reports/evidence-v19
BASE=$(cat "$EV/BASE_HEAD" 2>/dev/null || true)
[ -n "$BASE" ] && [ -f "$EV/BASE_STATUS" ] || { echo "baseline absente : lancer scripts/v19/base.sh (via /v19-wave0)"; exit 2; }
fail=0
if [ -n "$(git rev-list "$BASE"..HEAD)" ]; then echo "VIOLATION R2 : nouveau commit"; fail=1; fi
ALLOWED='^(omega/crates/omega-tools/tests/|docs/omega/reports/)'
while IFS= read -r line; do
  grep -qxF -- "$line" "$EV/BASE_STATUS" && continue
  path=${line:3}; path=${path##* -> }
  case "$path" in *proptest-regressions) continue;; esac
  echo "$path" | grep -Eq "$ALLOWED" || { echo "VIOLATION R1 : $line"; fail=1; }
done < <(git status --porcelain)
[ $fail -eq 0 ] && echo "scope OK"; exit $fail
EOF

cat > scripts/v19/lint-report.sh <<'EOF'
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
EOF

cat > scripts/v19/bg.sh <<'EOF'
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
EOF

chmod +x scripts/v19/*.sh

# ─────────────────────────── CONFIG OPENCODE ───────────────────────────
# Pas de fichier fantôme : on n'écrit opencode.json que s'il n'existe aucune config projet.
if [ -f opencode.json ] || [ -f opencode.jsonc ]; then
  echo "Config projet existante : NON modifiée. Pour des deny globaux, ajouter à la main :"
  echo '  "permission": { "bash": { "git push*": "deny", "git commit*": "deny" } }'
  echo "(les agents v19 portent déjà ces deny dans leur frontmatter ; /v19-smoke point 4 le vérifie)"
else
  cat > opencode.json <<'EOF'
{
  "$schema": "https://opencode.ai/config.json",
  "permission": {
    "bash": { "git push*": "deny", "git commit*": "deny" }
  }
}
EOF
  echo "opencode.json créé (aucune config projet n'existait)."
fi

echo "Bundle v19 installé (rev8, MAIN=worktree d'exécution)."
echo "Étapes : 1) opencode dans $(git rev-parse --show-toplevel 2>/dev/null || echo .)  2) agent orchestrator  3) /v19-smoke  4) /v19-master (reprise auto)"
echo "Variantes : /v19-wave0 puis /v19-run (pas à pas) ; /v19-lint <rapport.md>"
