---
description: Campagne maître v19 — omega-tools, pilotée par les outils, avec reprise, checkpoints et garde-fous
agent: orchestrator
---
Lis @.opencode/RULES-v19.md. Tu es l'orchestrateur. Objectif : mener la campagne omega-tools (audit → baseline mutation → saturation) et solder les contradictions héritées, en utilisant au maximum les agents, outils MCP, LSP et plugins RÉELLEMENT disponibles, sans jamais enfreindre R1–R12. Retours des subagents : JSON de RULES-v19.md.
IMPORTANT (rev8-armée, preuve en main) : le workspace Cargo vit dans omega/ — la racine du dépôt n'a AUCUN Cargo.toml. Toute commande cargo passe workdir=omega dans evidence.sh (`scripts/v19/evidence.sh <agent> <sujet> omega <to> -- cargo ...`) ; Livrable E de la section F compris. Les echo/git restent wd=".".

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
