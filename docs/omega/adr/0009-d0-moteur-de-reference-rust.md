# ADR-0009 — D0 : le moteur de référence est Rust (option O2)

**Date** : 2026-10-20 · **Statut** : Accepté · **Décidé par** : chef de projet
**Travail** : P0-9 (spike D0) · **Ferme** : porte G0.d · **Ne renverse pas** : ADR-0001

## Contexte

D0 décide **où vivent les priorités 1 à 5** (outils, mémoire, skills,
sous-agents, MCP) : elles existent dans la boucle d'exécution. Les construire
avant de savoir où les construire, c'est risquer de les écrire deux fois
(PLAN-MAITRE §5).

Le plan donne trois options — **O1** Python moteur / Rust en bordure,
**O2** Rust moteur / Python oracle, **O3** deux moteurs (interdit au-delà de
P0) — et une règle de décision **par les données** : spike borné à 2 sessions,
faire passer S0 à S2 sur **chaque** moteur, puis (1) nombre de scénarios verts
au bout de la boîte de temps, (2) effort mesuré, (3) à égalité on ne renverse
pas ADR-0001 sans preuve — la charge de la preuve pèse sur le changement,
(4) si le Python est retenu pour une zone, une raison **mesurée** est
consignée.

## Faits mesurés (observed — 2026-10-20)

| Moteur | S0 | S1 | S2 | Preuve |
|---|---|---|---|---|
| **Rust** (`omega-server`) | **PASS** | **XFAIL** (3 causes nommées : wire sans `tool_calls`, boucle absente, scripting 2 formes) | **scénario inexistant** | `omega/acceptance/runner.py --scenario S0\|S1 --engine rust` |
| **Python** (socle) | **SKIPPED** | SKIPPED | SKIPPED | `--engine python` → `SKIPPED(python_engine_absent)` |

Matériel du moteur Python, mesuré :

- séquence de démarrage **découverte** : `python3 app.py` — FastAPI/uvicorn,
  `APP_BIND` (127.0.0.1) et `APP_PORT` (7000) par défaut (`app.py:370-378`) ;
- **dépendances absentes** dans tous les environnements disponibles sur cette
  machine (`fastapi`, `sqlalchemy`) : le socle ne démarre pas ici, et sa suite
  de tests n'est pas exécutable (ADR-0011) ;
- le pont Rust↔Python reste un **stub** : `omega/bridge/handler.py` répond
  `{"status": "executed"}` sans exécuter d'outil.

Rust : 22 crates, 370 tests, `cargo test --workspace` **vert**,
`clippy --workspace -D warnings` **vert**, 0 cycle de dépendances, run réel
observé = 8 événements / 2 steps.

## Décision

**O2 — Rust moteur.** ADR-0001 est **confirmé**, pas renversé.

1. La boucle, les outils, la mémoire, les skills et les sous-agents s'écrivent
   **en Rust** (P1-1, déjà rescopé `[L]` → `[XL]` par ADR-0006).
2. Le socle Python devient l'**oracle de comportement** d'O2 — pas le moteur.
   Il ne peut tenir ce rôle qu'après **P1-0** (venv dédié avec
   `requirements.txt`) : tant que sa suite n'est pas exécutable, l'oracle
   n'existe pas. C'est une dette explicite, pas une supposition.
3. **O1** (Python moteur) est écarté **à ce jour** : le moteur Python ne
   démarre dans aucun environnement reproductible, le pont est un stub, et il
   n'aurait **aucun scénario vert** à opposer à ceux du Rust.
4. **O3** (deux moteurs) reste interdit.
5. Toute zone portée en Rust passe **la même suite de conformité (A1)** que
   son équivalent Python ; le pont ne compte pas comme zone portée tant
   qu'il est un stub.

## Limites assumées

Le spike n'a **pas** pu être mené à son terme (S0 → S2 sur chaque moteur) :

- **S1** exige des outils réels dans la boucle (le wire ne transporte pas
  `tool_calls` — ADR-0006) et **S2** (limites, politique, budget) n'existe
  pas encore : c'est du travail **P1**, pas un spike ;
- le **moteur Python n'est pas démarrable** ici (dépendances absentes).

D0 est donc une **ratification d'ADR-0001 appuyée sur des données mesurées**,
pas une expérience complète. **Condition de réexamen** : dès que S1 et S2
existent sur les deux moteurs, rejouer le critère (1) du plan ; si le Python
devient plus vert à effort égal, rouvrir D0 (cet ADR est révisable).

## Conséquences

- **P1-1** (la boucle) s'écrit en Rust ; ADR-0006 (transport des `tool_calls`
  dans le wire) en est le préalable.
- Le creux reste un fait (4 425 fonctions de test Python / 370 tests Rust) :
  D0 ne le comble pas, il dit **où on écrit**.
- **P1-0** (venv du socle) devient la condition de l'oracle Python : sans lui,
  aucune comparaison de parité n'est possible.
- La porte **G0.d** passe : l'ADR existe et il est Accepté.

## Alternatives rejetées

- **O1 — Python moteur** : 0 scénario vert mesuré, démarrage non
  reproductible, pont stub.
- **O3 — deux moteurs** : interdit au-delà de P0 par le plan.
- **Reporter D0** : G0.d resterait rouge et P1-1 n'aurait pas de lieu
  d'écriture — c'est précisément le reproche que le plan adresse aux
  sprints antérieurs.

## Provenance

- [observed] Verdicts du banc : `omega/acceptance/runner.py --scenario S0|S1
  --engine rust|python` (2026-10-20) ; rapport de porte
  `docs/omega/plan/gates/phase-0-20261020-*.json`.
- [observed] Démarrage du socle : `app.py:370-378`.
- [observed] `cargo test --workspace` vert et `clippy -D warnings` vert :
  rapport `/tmp/verify-*.json` du 2026-10-20.
- [inferred] Le socle Python « marche » : **non établi**. Le plan (§5) met
  lui-même en garde : « ne pas présumer que le Python marche, c'est ce que S1
  doit établir, pour les deux ».
