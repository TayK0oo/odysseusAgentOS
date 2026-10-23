# AGENTOS OMEGA — Le projet, la vision, l'état réel

> **Document unique de référence.** Tout ce qu'il faut savoir pour comprendre le projet :
> ce qu'il est, ce qu'il contient (chaque fichier et à quoi il sert), ce qui marche
> vraiment, ce qui est vide, et où on va.
>
> Écrit le 2026-10-19 · branche `feat/agentos-omega` · PR #1 mergée dans `dev`

---

## PARTIE 1 — LE PROJET

### 1.1 Ce que c'est, en une phrase

**Un agent de travail autonome** : il reçoit un objectif, il utilise de vrais outils sur
votre machine, il se souvient entre les sessions, il apprend de ses erreurs, et vous
pouvez le débrancher ou le modifier sans le recompiler.

### 1.2 D'où il vient

Le projet est né d'un constat : les agents IA modernes sont **(a)** powerful mais fermés,
**(b)** ou ouverts mais sans mémoire. odysseusAgentOS veut être **les deux** :
auto-hébergé, sceptique, outillé, et qui s'améliore tout seul.

### 1.3 Les deux socles

| Socle | Langage | Rôle | État |
|---|---|---|---|
| **odysseusAgentOS** | Python | le système qui **tourne** : 4 440 <!--m:socles.python.tests=4440--> fonctions de test recensées (méthode : AST — pas une exécution) | ⚠️ suite non exécutée ici (dépendances absentes, ADR-0011) |
| **omega** | Rust + React | le noyau **nouveau** : routeur, ports, sandbox | ⚠️ châssis réel, moteur absent |

> **Point crucial, détaillé en Partie 4** : ces deux socles se sont malencontreusement
> chevauchés. Le Python fait le travail ; le Rust refait le travail sans le faire encore.

---

## PARTIE 2 — LA VISION

### 2.1 Le North Star (repris de `docs/omega/00-VISION-OMEGA.md`)

> Un agent reçoit un objectif, l'exécute jusqu'au bout sans dériver, **apprend de chaque
> run**, et ne dépasse jamais ses limites — sans intervention humaine constante.

### 2.2 Les 6 piliers (tels que définis dans `00-VISION-OMEGA.md` §2)

| # | Pilier | Ce que ça veut dire concrètement |
|---|---|---|
| 1 | **Routing intelligent** | Chaque modèle a un rôle, un coût, une vitesse. Le système choisit selon la tâche, bascule si ça échoue, sait tourner en local. |
| 2 | **Reverse-engineering** | Capacité reproductible d'extraire les capacités des meilleurs harnais du marché et de les ré-écrire proprement chez soi. |
| 3 | **Everything is a plugin** | Aucun composant en dur. Tout passe par un port + un adaptateur. Configuration > code. |
| 4 | **Rust / React-TS** | Noyau sûr et rapide, interface claire et observable. |
| 5 | **Économie & traçabilité** | Le déterministe va au système (scripts), pas à l'agent. Tout est mesuré, daté, rejouable. |
| 6 | **Adaptatif** | Le système apprend de chaque run et ne dépend d'aucun fournisseur unique. |

### 2.3 Où le marché va (votre veille, pas la mienne)

`~/Téléchargements/veille_systemes_ia.md` — la référence 2026 est **Hermes Agent** (Nous
Research, MIT) : l'agent auto-améliorant qui **crée ses propres skills à partir de son
expérience**, garde une mémoire persistante, fait tourner des sous-agents en parallèle,
parle MCP, sur 20+ plateformes, en continu sur votre serveur.

> **Le standard vers lequel tout converge** : *mémoire persistante + skills + MCP + multi-plateforme*.

C'est exactement nos piliers 3 et 6, mais formulés par le marché au lieu de nous.

### 2.4 Ce que nous voulons vraiment

Six priorités, **dans cet ordre**. Chacune a une démonstration qui prouve qu'elle est
faite — pas un statut, pas un rapport.

| # | Priorité | Démonstration qui prouve que c'est fini |
|---|---|---|
| 1 | **Mémoire qui survit** | Vous parlez à l'agent, vous coupez le processus, vous relancez : il se souvient. |
| 2 | **Outils branchés** | « liste les fichiers » → il appelle `ls` → il répond avec le vrai contenu du dossier. |
| 3 | **Skills auto-générés** | L'agent échoue 3 fois sur une tâche → il **écrit lui-même** un `SKILL.md` → il réussit la 4ᵉ. |
| 4 | **Sous-agents** | Une tâche longue se découpe, 3 agents en parallèle, la trace montre l'arbre. |
| 5 | **MCP + plateformes** | Le même cerveau Piloté depuis Telegram et depuis l'IDE. |
| 6 | **Adaptatif** | Chaque run est scoré → le routeur change de modèle tout seul, sans qu'on l'y force. |

---

## PARTIE 3 — CE QUI EXISTE : INVENTAIRE ET RÔLE DE CHAQUE FICHIER

### 3.1 Documents de référence

| Fichier | Ce que c'est | À quoi ça sert |
|---|---|---|
| `docs/omega/00-VISION-OMEGA.md` | Les 6 piliers, l'axe hexagonal, le routeur, les non-objectifs | **La source de vérité de la vision.** À lire avant tout |
| `SFD.md` (racine) | Spécification fonctionnelle de référence | Le cahier des charges fonctionnel |
| `AGENTS.md` (racine) | Orientation pour les agents | Les 6 règlesglobals + le modèle de travail |
| `docs/omega/02-WORKSTREAMS.md` | Découpage W1→W10 | Comment le projet a été découpé |
| `docs/omega/MEMORY.md` | Mémoire auto-générée | État courant du corpus ; **généré par script, ne pas éditer** |
| `docs/omega/INDEX.md` | Carte de l'espace Omega | Point d'entrée dans les docs |
| `docs/omega/RECAP-COMPLET.md` | Synthèse de session | Historique condensé |

### 3.2 Les règles (non négociables)

| Fichier | Contenu |
|---|---|
| `docs/omega/rules/RULES-OMEGA.md` | Règles de session : autocontrôle, traçabilité, pas de code non testé |
| `docs/omega/rules/MODEL-POLICY.md` | **Sous-agents sur modèles gratuits**, payant = exception justifiée |
| `.opencode/rules/omega.md` | Version courte chargée automatiquement |
| `model-policy.md` (racine) | Politique de modèles |
| `SECURITY.md` / `THREAT_MODEL.md` (racine) | Menaces et mitigations |

### 3.3 Les décisions d'architecture (ADR)

| Fichier | Décision |
|---|---|
| `adr/0001-moteur-rust-reecriture.md` | Le noyau sera en Rust |
| `adr/0002-ui-react-ts-app-separee.md` | L'UI est séparée (React/TS) |
| `adr/0003-architecture-plugin-capacites.md` | Ports + adaptateurs + registre |
| `adr/0010-plugin-convergence.md` | Convergence vers « everything is a plugin » |

### 3.4 Le reverse-engineering (notre advantage concurrentielle)

| Fichier | Ce que c'est | Rôle |
|---|---|---|
| `docs/omega/reverse-eng/dissections/*.json` | **51 fichiers** <!--m:avancee.dissections=51--> — dissectés de 51 harnais du marché | **Source de vérité** de ce que le marché sait faire. 910 <!--m:avancee.fonctionnalites_total=910--> fonctionnalités |
| `docs/omega/reverse-eng/dissections/*.md` | Fiches lisibles générées des JSON | Pour les humains |
| `docs/omega/reverse-eng/OMEGA-COMPLET.md` | Le corpus fusionné | 102 <!--m:avancee.capacites_omega=102--> capacités (`omega_capability` fait foi), consensus pondéré par popularité |
| `docs/omega/reverse-eng/HARNESS-CENSUS.md` | 67 <!--m:avancee.census_recenses=67--> harnais recensés | Quels sont disséqués, lesquels sont écartés |
| `docs/omega/reverse-eng/CONVERGENCE-TOUT.md` | Mapping capacité → port | **Le pont entre RE et architecture** |
| `docs/omega/reverse-eng/PORTING-SCOPE.md` | Ce qui est porté / non porté | Empêche de se mentir sur le périmètre |
| `docs/omega/research/rea/` | Preuves d'analyse automatique | Traçabilité des dissections |

### 3.5 La veille (TikTok, 148 vidéos)

| Fichier | Contenu |
|---|---|
| `docs/omega/research/loucash/videos_enriched.json` | 78 vidéos réelles (@0xloucash) |
| `docs/omega/research/loucash/videos_axelayzeia_complete.json` | 35 vidéos réelles |
| `docs/omega/research/loucash/videos_thomasbssh1_complete.json` | 35 vidéos réelles |
| `docs/omega/research/loucash/profile_*.json` | Profils des 2 comptes |
| `docs/omega/reverse-eng/tiktok/` | Analyse consolidée des outils |

> ⚠️ **Non utilisé** : `~/Téléchargements/veille_tiktok_dev_groupee.md` — **105 ressources dev**
> (10+ comptes : @howtowebdev, @code_simple, @sansdependances…) que nous n'avons jamaisTreatées.

### 3.6 Le workspace Rust (`omega/`)

**22 crates** <!--m:socles.rust.crates=22--> :

| Crate | Rôle | Utilisé par |
|---|---|---|
| `omega-ports` | Les **24 ports** (traits purs, zéro implémentation) | Tout le monde |
| `omega-types` | Types partagés (Erreur, Evenement, Modèle…) | Tout le monde |
| `omega-core` | Boucle d'agent, bus d'événements | Le runner |
| `omega-router` | Scoring + sélection + fallback | `/api/run` |
| `omega-providers` | Adaptateurs LLM | Le runner |
| `omega-context` | Assemblage du contexte | Le runner |
| `omega-app` | **Assemblage applicatif** — `run()` | Le serveur |
| `omega-server` | **Le binaire** — API HTTP + UI | `cargo run -p omega-server` |
| `omega-registry` | Registre de capacités (slot/ensemble, quarantaine) | La composition |
| `omega-sandbox` | Isolation + `ExecPolicy` | Les outils |
| `omega-config` | Config à 4 couches | Le serveur |
| `omega-harness` | Corpus embarqué + seuil de test | La CI |
| `omega-mcp` | Client MCP | L'agent |
| `omega-otel` | Télémétrie | La traçabilité |

**Portés depuis Python** (v0.7) :

| Crate | Port de quoi |
|---|---|
| `omega-memory` | `services/memory/` — mémoire + skills |
| `omega-search` | `services/search/` — requête + ranking |
| `omega-policy` | `budget_enforcer` + garde OPA — plafonds, permissions |
| `omega-tools` | `agent_tools/filesystem_tools.py` — 7 outils fichiers |

> Le détail des tests par crate n'est pas recopié ici : une copie dérive. Source
> unique : `STATUS.json` → `socles.rust.tests.detail_par_crate`.

**Ajouts récents** :

| Crate/module | Rôle |
|---|---|
| `omega-adapters` | 8 adaptateurs par défaut + `composition.rs` (le registre) |
| `omega-python-bridge` | Pont Rust ↔ Python (JSON-RPC) |
| `omega/bridge/handler.py` | Dispatcher JSON-RPC (**stub** — n'exécute aucun outil) |

### 3.7 L'interface

| Fichier | Rôle |
|---|---|
| `omega/ui/src/` | React/TS — cockpit |
| `omega/ui/vite.config.ts` | Build Vite |
| `omega/scripts/run-local.sh` | **Lance tout** : build UI + serveur sur `127.0.0.1:8787` |

### 3.8 Les scripts (le déterministe va ici, pas à l'agent)

| Script | Ce qu'il fait |
|---|---|
| `omega/scripts/harness_census.py` | Recense les harnais du marché via l'API GitHub |
| `harness_intake.py` | Clone un harnais + collecte les preuves REA |
| `validate_dissections.py` | **Vérifie que chaque `fichier:ligne` cité existe** |
| `gen_dissection_md.py` | Génère la fiche lisible depuis le JSON |
| `omega_complet.py` | Fusionne toutes les dissections en `OMEGA-COMPLET.md` |
| `register_dissections.py` | Enregistre dans le corpus embarqué + met à jour le seuil de test |
| `generate_memory.py` | Régénère `MEMORY.md` |
| `check_dep_graph.py` | Vérifie 0 cycle de dépendances |
| `model_policy.py` | Vérifie la politique de modèles |

### 3.9 Le projet Python (`/home/tayk0oo/Projets/odysseusAgentOS/`)

**321 modules, 729 fichiers de test, 4 440 <!--m:socles.python.tests=4440--> fonctions de test recensées.**

| Zone | Ce qu'elle contient |
|---|---|
| `src/agent_loop.py` (4 986 l.) | La boucle agent streaming + outils multi-rounds |
| `src/orchestrator/` | Saga compensée, `DurableExecutor`, checkpoints, ancestry, gates |
| `src/agent_tools/` | Les outils d'agent (fichiers, web, documents, sessions) |
| `services/` | memory, search, observability, security, stt/tts, research |
| `core/` | database, middleware, auth, atomic_io, route_loader |
| `routes/` (64) | L'API |
| `docs/master-ref/` | Analyse des 68 outils, SFD v3.1, architecture modulaire |

**C'est le système qui fonctionne.** Rust n'en est qu'un écho plus faible.

### 3.10 La documentation de travail dans `~/Téléchargements/`

| Fichier | Ce que c'est | Statut |
|---|---|---|
| `veille_systemes_ia.md` | **La vision marché 2026** — Hermes, OpenClaw, agents de code | ❌ jamais lu jusqu'à ce jour |
| `veille_tiktok_dev_groupee.md` | **105 ressources dev** | ❌ jamais traité |
| `veille_tiktok_complete.md` | Veille TikTok complète (non dev) | ⚠️ partiellement traitée |
| `odysseusAgentOS-v6..v11-rapport.md` | 6 versions du projet | ❌ jamais lus |
| `odysseusAgentOS-sprint3/sprint4-rapport.md` | Sprint 3 et 4 | ❌ jamais lus |
| `odysseusAgentOS-final-rapport.md` | Rapport final | ❌ jamais lu |
| `fiches_veille.md` | Fiches de veille | ❌ jamais lu |
| `opencode-*_classement_*.md` | Classement des modèles gratuits / go | ❌ jamais lu |
| `opencode_sources_*.md` | Sources OpenCode | ❌ jamais lu |

> C'est **la plus grosse déperte de la session** : 12 rapports + 6 docs de veille,
> soit l'histoire réelle du projet et la vision marché, non exploitées.

---

## PARTIE 4 — L'ÉTAT RÉEL (mesuré, pas déclaré)

### 4.1 Ce qui marche, prouvé par exécution

```
POST /api/run {"messages":[{"role":"user","content":"Liste les fichiers"}]}
→ run-1 · router task=T2 · pareto → ollama/qwen2.5-coder:7b
→ 8 événements · 2 steps · 0 échec · trace durable
```

| ✅ Fonctionnel | Preuve |
|---|---|
| Routage (scoring + Pareto + fallback) | `ollama/qwen2.5-coder:7b` sélectionné, score 0.9 |
| Boucle d'exécution | 2 steps, 8 événements, résumé `completed` |
| Journal durable + trace | `/api/trace/run-1` rejouable |
| API HTTP + UI | 15 routes, cockpit React servi |
| Corpus RE | 51 dissections <!--m:avancee.dissections=51-->, 910 <!--m:avancee.fonctionnalites_total=910--> fonctions, toutes les refs `fichier:ligne` résolues |
| Sécurité (Rust) | 46 tests adversariaux, 8 vulnérabilités trouvées et corrigées |
| Python odysseusAgentOS | **4 440** <!--m:socles.python.tests=4440--> fonctions de test (AST), suite non exécutée ici (ADR-0011) |

### 4.2 Ce qui est vide — le constat qu'il faut assumer

| ❌ Creux | Constat mesuré |
|---|---|
| **Le LLM n'a aucun outil** | Il répond « je ne dispose pas de l'accès direct aux fichiers ». Le run s'exécute, l'agent ne peut rien faire. |
| **Les 8 nouveaux adaptateurs ne sont pas branchés** | `/api/plugins` retourne 5 entrées : `ollama, default, mcp:rea, file, embedded`. **Aucun** des nôtres. |
| **La mémoire ne persiste pas** | Priorité n°1 de la vision : absente. |
| **Les skills ne s'auto-génèrent pas** | `omega-memory` sait *lire* un `SKILL.md`, mais rien ne l'écrit. |
| **148 vidéos TikTok → 0 code** | Cataloguées, jamais absorbées. |
| **910 fonctions RE → ~4 crates** | 910 <!--m:avancee.fonctionnalites_total=910--> fonctions recensées ; 223 k lignes Python → ~4 k traduites. |
| **Rust vs Python** | Python : 4 440 <!--m:socles.python.tests=4440--> fonctions de test. Rust : 411 <!--m:socles.rust.tests=447-->. **Nous avons réécrit en plus faible ce qui marchait.** |
| **Le handler Python est un stub** | `{"status": "executed"}` — n'exécute aucun outil. |

### 4.3 Pourquoi nous en sommes là

Le cycle GSD a optimisé **la métrique** (tests verts, gates passées, couverture de
document) plutôt que **la capacité**. Chaque phase produisait des `.md` et des stubs
testés — c'était vérifiable, donc cela passait les gates. Aucun gate n'a jamais demandé
« l'agent sait-il faire quelque chose ? ».

---

## PARTIE 5 — LE PLAN RÉEL

### 5.1 La règle unique

> **Une phase qui ne produit pas une démonstration exécutable n'est pas terminée.**

Critère de fin = une commande qu'on lance et une sortie qu'on observe. Pas un statut.

### 5.2 Les étapes, dans l'ordre qui débloque tout

| # | Étape | Démonstration de fin | Pourquoi d'abord |
|---|---|---|---|
| **1** | Brancher les outils dans la boucle LLM | `POST /api/run` → l'agent appelle `ls` → il liste les vrais fichiers | Sans ça, tout le reste est décoratif |
| **2** | Enregistrer les adaptateurs dans le registre | `/api/plugins` expose les adaptateurs branchés, pas seulement ceux d'aujourd'hui | Le code existe déjà, il n'est juste pas branché |
| **3** | Mémoire persistante | « Quel était mon projet ? » après redémarrage → il répond | Priorité n°1 de la vision |
| **4** | Skill auto-généré | 3 échecs → il écrit `SKILL.md` → 4ᵉ réussite | La preuve « agent qui s'améliore » |
| **5** | Sous-agents parallels | Trace d'arbre visible | Extension naturelle du routeur |
| **6** | Traitement des 105 ressources dev | Chaque ressource → testée contre nos ports → **taux de couverture mesuré** | Transforme un catalogue mort en instrument de mesure |
| **7** | Décision Rust/Python par zone | Document qui dit où le Rust est justifié | Arrête le saignement d'effort |

### 5.3 Le critère de mesure qui remplace tous les autres

**Couverture pondérée** — définition au §9 du `PLAN-MAITRE.md` : Σ poids des capacités
dont le scénario est vert ÷ Σ poids, sur les 51 <!--m:avancee.dissections=51--> harnais
disséqués (910 <!--m:avancee.fonctionnalites_total=910--> fonctionnalités).

Aujourd'hui : **0** <!--m:couverture_ponderee=0-->. Ce zéro ne dit pas « presque rien »,
il dit « aucun scénario exécuté » : il mesure l'absence de preuve, pas l'absence de
code. Il monte quand un scénario passe, pas quand une ligne est écrite.

---

## PARTIE 6 — CE QU'IL FAUT NE PAS REFAIRE

| Erreur | Pourquoi elle a coûté cher |
|---|---|
| Réécrire en Rust ce qui marche en Python | 4 440 <!--m:socles.python.tests=4440--> fonctions de test Python abandonnées au profit de 411 <!--m:socles.rust.tests=447--> tests Rust |
| Optimiser les gates plutôt que la capacité | 27 phases de cérémonies, 0 agent capable |
| Produire des docs au lieu de démontrations | Le corpus RE est splendide et inexploité |
| Ne pas lire les rapports de la session précédente | 12 rapports + 6 veilles ignorés |
| Extraire la veille « grand public » au lieu de la veille dev | 148 vidéos peu utiles au lieu de 105 ressources dev |

---

## PARTIE 7 — OÙ SONT LES CHOSES (index rapide)

| Je cherche… | C'est ici |
|---|---|
| La vision | `docs/omega/00-VISION-OMEGA.md` |
| Les règles | `docs/omega/rules/RULES-OMEGA.md` |
| L'état du projet | `docs/omega/MEMORY.md` (généré) |
| Ce que le marché sait faire | `docs/omega/reverse-eng/OMEGA-COMPLET.md` |
| Une fiche harnais | `docs/omega/reverse-eng/dissections/<id>.md` |
| Le pont RE → architecture | `docs/omega/reverse-eng/CONVERGENCE-TOUT.md` |
| Le périmètre porté | `docs/omega/reverse-eng/PORTING-SCOPE.md` |
| L'API du serveur | `omega/crates/omega-server/src/routes.rs` |
| La boucle agent | `omega/crates/omega-app/src/lib.rs` · `omega/crates/omega-core/src/lib.rs` |
| Les ports | `omega/crates/omega-ports/src/lib.rs` |
| Les outils portés | `omega/crates/omega-tools/src/` |
| La vision du marché | `~/Téléchargements/veille_systemes_ia.md` |
| La veille dev (105 ressources) | `~/Téléchargements/veille_tiktok_dev_groupee.md` |
| L'historique du projet Python | `~/Téléchargements/odysseusAgentOS-*.md` |
| Le système qui marche | `/home/tayk0oo/Projets/odysseusAgentOS/` |

---

## PARTIE 8 — CE DOCUMENT EST VIVANT

Il doit être mis à jour **à chaque démonstration réussie**, pas à chaque rapport.
Une ligne qui décrit une capacité non démontrée est une ligne à supprimer.

*Les chiffres vivants de ce document portent un tag `&lt;!--m:CLE=VALEUR--&gt;`, vérifié
contre `docs/omega/plan/STATUS.json` par `omega/scripts/check_doc_truth.py` (ADR-0007).
Ne recopier ici aucun chiffre mesuré sans son tag ; les chiffres historiques vivent
dans `RECAP-COMPLET.md`.*