# ADR-0015 — G1.a.s1 : S1 passe au vert, le critère « S1 rouge » (G0.c.s1) est retiré

**Date** : 2026-10-22 · **Statut** : Accepté · **Décidé par** : chef de projet
**Travail** : P1-1 (B1) · **Ferme** : critère G1.a.s1 de la porte G1
**Ne renverse pas** : ADR-0013 (G1 exécutable), ADR-0006 (wire tool_calls)

## Contexte

La porte G1 est outillée (ADR-0013) mais **rouge sur trois critères** : `G1.a.s1`
(S1), `G1.b.s2` (S2), `G1.c.sequence` (S0→S1→S2). Le premier obstacle n'était pas
la boucle d'outils — elle existait, ses tests passaient — mais le **chemin réel** :
`POST /api/run` enchaînait `assemble` + `infer`, c'est-à-dire **un seul appel
LLM**, sans surface d'outils. La boucle existait, personne ne l'appelait.

Deux constats de lecture ont fait le reste (leçon L2 : vérifier le flux, pas
supposer) :

1. **Le canary ne pouvait structurellement pas être vu.** S1 écrit 7 fichiers de
   noms aléatoires dans le `workdir` du scénario et attend que la réponse les
   contienne. Or le `workdir` n'était **jamais transmis au serveur** : la surface
   d'outils, même branchée, n'aurait pas pu lire ce répertoire. Le scénario
   exigeait donc l'impossible, pas « l'encore-absent ».
2. **Le modèle scripté ne demandait aucun outil.** S1 scriptait un écho : le
   modèle ne pouvait pas appeler d'outil, donc aucune observation ne pouvait
   revenir. Le scénario ne mesurait pas la boucle — il mesurait son absence.

## Décision

**D1 — La boucle d'outils est branchée sur le chemin réel.** `execute_run`
enchaîne `assemble` + `tool_loop` quand une surface d'outils est configurée.
Détection du tool-call (forme native ou repli ReAct, ADR-0006) → exécution →
observation réinjectée → multi-tours, arrêté par un budget strict et monotone.
La boucle publie `infer.completed` : le contrat du journal (`last_answer`, UI)
ne dépend pas du chemin pris.

**D2 — La surface d'outils locale est choisie explicitement.** `mcp:rea` occupe
déjà la capacité `tool-surface` au registre et lance `npx rea-agents --mcp` à
chaque appel (indisponible en CI, et sans rapport avec « lister un
répertoire »). La surface FS locale est donc **résolue depuis la configuration**
(`ServerConfig.tool_surface`, `OMEGA_TOOL_SURFACE` / `OMEGA_WORKSPACE_ROOT`),
pas depuis le registre. Ce n'est pas une exception à « everything is a plugin » :
la surface reste une implémentation du port `ToolSurface`, la boucle ne connaît
que le port.

**D3 — Le scénario S1 est rendu exécutable, et son contrôle négatif est
implémenté.** Le provider scripté gagne une séquence `outil` (tour 1 demande
`ls`, tour 2 reprend l'observation), le `workdir` est transmis au serveur
(comme racine FS), et le contrôle négatif `desactiver: [tool_surface]` est
**implémenté** : le même scénario rejoué sur un moteur *sans* surface d'outils
doit produire une réponse **sans** les canaries. C'est le contrôle du contrôle —
sans lui, « les canaries sont dans la réponse » ne prouverait rien.

**D4 — Le critère `G0.c.s1` est retiré.** Il affirmait « S1 rouge sur l'état
actuel » ; S1 n'est plus rouge, l'affirmation est fausse. Le critère de phase 0
qui le couvrait est remplacé par le critère de phase 1 `G1.a.s1` (5 runs, PASS
sec). La phase 0 garde `G0.c.s0` (démarrage propre), qui reste vrai.

**D5 — Un seul chemin d'écriture du journal.** La boucle écrivait
`tool.call.completed` **deux fois** dans l'App (via `ctx.publish`, bus adossé au
journal, **et** via le champ `trace`, append direct) : mesuré, 2 occurrences au
lieu d'une. Dans l'App, `trace` reste à `None` — le bus est l'unique source.

## Conséquences

- `G1.a.s1` devient vert (S1 PASS, 5 runs, contrôle négatif inclus).
- `G1.b.s2` et `G1.c.sequence` restent rouges : les budgets ne sont pas encore
  appliqués dans la boucle (P1-3, travail B2). Le rouge reste **sincère**.
- Le comportement par défaut du serveur change : une surface d'outils locale est
  active par défaut (`OMEGA_TOOL_SURFACE=fs`), la racine étant le répertoire
  courant. `OMEGA_TOOL_SURFACE=off` restaure le comportement « un seul appel
  LLM ».
- La surface locale doit connaître sa racine : un déploiement doit fixer
  `OMEGA_WORKSPACE_ROOT` explicitement, sinon la racine est le répertoire
  courant du processus. Point de vigilance documenté (pas de Fail-Closed sur ce
  point : la jail de `omega-tools` borne toujours les chemins résolus sous la
  racine).

## Alternatives écartées

- **Attendre l'extension du wire (`tool_calls` natif)** : le repli ReAct
  d'ADR-0006 est déjà le chemin canonique et fonctionne ; attendre le schéma
  étendu retarderait la porte sans rien prouver de plus.
- **Enregistrer la surface FS dans le registre** : la capacité `tool-surface`
  est en régime *ensemble* mais `mcp:rea` l'occupe déjà et le premier
  enregistrement gagnerait la résolution — la surface MCP deviendrait
  inexploitable et la liste nominative de `G1.d` (13 couples) devrait être
  refaite. Décision inverse : configuration explicite, registre inchangé.
- **Déclarer S1 vert sans son contrôle négatif** : aurait produit un vert
  « observé » mais non « prouvé » — exactement ce que la leçon L4 interdit.
