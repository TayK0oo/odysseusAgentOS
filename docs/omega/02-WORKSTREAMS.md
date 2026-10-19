# 02 — WORKSTREAMS Omega (décomposition & phasage)

> **Objet :** décomposer la vision (`00-VISION-OMEGA.md`) en chantiers exécutables,
> ordonnés par dépendances, avec estimation et critère de sortie.
> **Statut :** vivant · **Date :** 2026-10-15 · **Branche :** `feat/agentos-omega`.
> **Provenance :** `[observed]` (fait) · `[inferred]` (estimation) · `[todo]`.
>
> **État des chantiers :** la colonne *Statut* et les chiffres par chantier sont ceux
> du **2026-10-15** (`[observed]` à cette date) ; rien ne les revérifie. Plusieurs
> chantiers marqués « à venir » ont depuis des crates. Ne pas les lire comme l'état
> courant : l'état mesuré est `docs/omega/plan/STATUS.md`.

---

## Vue d'ensemble

| WS | Titre | Dépend de | Effort `[inferred]` | Statut |
|---|---|---|---|---|
| **W0** | Fondation (branche, règles, mémoire, ADR, CI modularité) | — | S | 🟢 socle posé |
| **W1** | Recherche & veille (corpus, harnais, benchmarks) | — | M | ✅ fait |
| **W2** | Socle plugin : capacités, ports, registre, manifestes | W0 | L | 🟢 en cours |
| **W3** | Moteur Rust : boucle d'agent, event-bus, durable exec | W2 | L | ⚪ à venir |
| **W4** | Routing intelligent : registre, scoring, politique, fallback | W2 | L | ⚪ à venir |
| **W5** | Reverse-engineering : fiches + blueprint | W1 | M | ✅ fait |
| **W6** | Économie de contexte : Context Engine, tool search, compaction, mémoire | W3 | L | ⚪ à venir |
| **W7** | Cockpit UI React/TS | W3, W8 | L | ⚪ à venir |
| **W8** | Traçabilité : event store, provenance, OTel, replay | W3 | M | ⚪ à venir |
| **W9** | Sécurité & sandbox : backends, execpolicy, approbations | W3 | L | ⚪ à venir |
| **W10** | Économie & self-hosting : modèles locaux, gateways, cache | W4 | M | ⚪ à venir |

*(S ≈ ≤1 j · M ≈ 2-5 j · L ≈ 1-3 sem., à l'échelle d'un opérateur assisté.)*

## Chemin critique

```
W2 ──► W3 ──► W6 ──► W7
        │       ▲
        ├──► W8 ┘
        ├──► W9
        └──► W4 ──► W10
```

**W2 est le verrou** : rien ne s'assemble proprement sans les capacités/ports/registre.

---

## Détail par chantier

### W0 — Fondation `[observed]`
Branche isolée (worktree), `RULES-OMEGA.md`, `MEMORY.md`, ADR 0001-0003, `.opencode/rules/omega.md`,
gabarit + validateur de credentials. **Critère de sortie :** une session neuve peut reprendre via `AGENTS.md` → `MEMORY.md`.

### W1 — Recherche & veille `[observed]`
- Corpus TikTok `@0xloucash` : 78 vidéos, `classified.json`, vault Obsidian (94 notes).
- `HARNESS-LANDSCAPE.md` (25 harnais), `MODEL-ROUTING-BENCHMARKS.md` (sources, composite, taxonomie).
**Critère de sortie :** chaque pilier a une base documentaire sourcée.

### W2 — Socle plugin `[observed] 🟢`
Crates `omega-types`, `omega-ports`, `omega-config`, `omega-registry`, `omega-plugin-api`
(+ `omega-core`, `omega-protocol` en incrément 2). Deux régimes (slot/ensemble), quarantaine,
config à couches. **Critère de sortie :** (a)-(f) verts — dont le **garde-fou zéro-cycle**.

### W3 — Moteur Rust
Boucle `Operation/Inference/Effect` + `EventBus` durable/éphémère + annulation coopérative +
exécution durable (reprise après crash). **Critère :** un run reprend après interruption sans perte.

### W4 — Routing intelligent
Registre de modèles (capacités, coûts, vitesses, axes), classifieur de tâche déterministe,
scoring composite, sélecteur Pareto, fallback + circuit breaker, feedback bandit, cache de décision.
**Critère :** une tâche T1 route vers un modèle rapide, une T5 vers un modèle élite, sans LLM de décision.

### W5 — Reverse-engineering `[observed] ✅`
3 fiches (2 066 l.), `FEATURE-MATRIX.md` (96 features), `OMEGA-BLUEPRINT.md` (18 crates), `features.json`.

### W6 — Économie de contexte
Port `ContextEngine` (ingest→assemble→compact→afterTurn, `ownsCompaction`), tool search BM25
+ rarest-token gate, progressive disclosure des skills, prompt caching (breakpoints + affinité),
budget de contexte (estimate→plafond→arrêt), mémoire bornée + FTS.
**Critère :** X % des tours n'embarquent pas les schémas d'outils inutiles ; budget respecté.

### W7 — Cockpit UI React/TS
App `omega/ui/` (Vite+TS) : Cockpit, Plugin Explorer, Router Dashboard, Trace Viewer/Replay,
Context Inspector. Contrats générés depuis `omega-protocol`.
**Critère :** on remplace un plugin depuis l'UI et on voit la décision de routing + son coût.

### W8 — Traçabilité
`TelemetrySink` (OTLP/stdout/fichier), event store append-only, provenance `[stated]/[observed]/[inferred]`,
replay de session. **Critère :** toute décision est rejouable depuis les traces.

### W9 — Sécurité & sandbox
`SandboxBackend` (local/docker/ssh/…), `ExecPolicy` (DSL), `PermissionPolicy` (allow/ask/deny),
approbations granulaires. **Critère :** une commande hors politique est bloquée et tracée.

### W10 — Économie & self-hosting
Providers locaux (Ollama/llama.cpp), gateway, cache, alternatives gratuites. Budget par run.
**Critère :** un flux complet tourne à coût nul (modèle local) sans changer le noyau.

---

## Règles de progression

1. Un chantier ne démarre que si son **dépendant** est au vert.
2. Chaque brique naît d'un **port + contrat + test d'acceptation** (RULES-OMEGA §5.4).
3. Vérification **indépendante** (Reviewer) pour tout livrable de code critique.
4. Le déterministe reste un **script/outil**, jamais un tour de LLM.
