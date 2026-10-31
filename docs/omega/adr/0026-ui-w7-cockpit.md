# ADR-0026 — UI W7 : Cockpit et panneaux (PROPOSÉ)

**Date** : 2026-10-30 · **Statut** : PROPOSÉ (en attente de décision humaine) · **Décidé par** : à venir

## Contexte

La vision (pilier 4, « un cockpit clair et intuitif ») exige 5 panneaux UI : Cockpit, Plugin Explorer, Router Dashboard, Trace Viewer, Context Inspector. Le backend (omega-server) expose déjà 15+ endpoints (`/api/health`, `/api/plugins`, `/api/run`, `/api/trace/:run_id`, `/api/system`, `/api/overview`, `/api/registry`, `/api/config`, `/api/routes`, `/api/models`, `/api/classify`, `/api/runs`, `/api/runs/:run_id/events`, `/api/policy`, `/api/policy/evaluate`, `/api/context`, `/api/harnesses`, `/api/harnesses/:id`, `/api/dissections`, `/api/dissections/:id`). L'UI React/TS existe (`omega/ui/src/`) avec `App.tsx`, `panels.tsx`, `components.tsx`, `api.ts`, `protocol.ts`, `styles.css`.

## Décision (à venir)

Implémenter les 5 panneaux en React/TS, consommant les endpoints existants. Découpage en 4 sous-incréments (1 par panneau ou groupe).

## Justification

- Le backend expose déjà les données nécessaires (runs, traces, plugins, modèles, contexte).
- L'UI React/TS existe (App.tsx, panels.tsx).
- Le protocol wire (omega-protocol) est versionné (SCHEMA_VERSION = 1).

## Alternatives

- **(a)** Implémenter les 5 panneaux en une session (4-6h).
- **(b)** Implémenter 2-3 panneaux par session (3 sessions).
- **(c)** Reporter l'UI et se concentrer sur la traçabilité W8 d'abord.

## Conséquence

- Modification de `omega/ui/src/` (frontend, pas de production Rust).
- Tests d'intégration serveur ↔ UI.
- Démo reproductible (screenshot ou GIF).

## Portée

Uniquement `omega/ui/src/`. Aucune modification de `omega/crates/*/src/`.

## Risques

- Le Cockpit et le Context Inspector consommeront les composants W6 (ToolSurfaceIndex, SkillIndex, PinnedFacts). Si ces composants sont vides (peuplement reporté), l'UI affichera des données vides.
- Le Trace Viewer / Replay nécessite W8 (traçabilité) — dépendance.
- Le Router Dashboard nécessite les scores des modèles — données disponibles via `/api/models` et `/api/classify`.

## Prérequis

- D-W6 résolu (peuplement W6) — sinon l'UI affiche des index vides.
- W8 (traçabilité) pour le Trace Viewer / Replay — sinon le viewer est limité aux traces existantes.
