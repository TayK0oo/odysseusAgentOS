# ADR-0027 — Traçabilité W8 : Event-bus, provenance, replay (PROPOSÉ)

**Date** : 2026-10-30 · **Statut** : PROPOSÉ (en attente de décision humaine) · **Décidé par** : à venir

## Contexte

La vision (pilier 5, « tout est mesuré, prouvé, daté ») exige une traçabilité complète : chaque décision, appel modèle, outil, coût, erreur est un événement. La provenance `[stated]/[observed]/[inferred]` doit être partout. Le replay doit rejouer une session à partir des traces sans ré-exécuter les effets.

## État actuel

- `omega-core` : `EventBus` (InMemoryBus), `Event` (durable/ephemeral), `DurableEvent` (run_id, seq, name, operation_id, payload).
- `omega-app` : `FileEventStore` (append-only JSONL), `InMemoryEventStore`, `TraceReplayer::summarize`.
- `omega-server` : `GET /api/trace/:run_id`, `GET /api/runs`, `GET /api/runs/:run_id/events`.
- `omega-otel` : `FileSink`, `StdoutSink`, `TelemetrySink`, `Provenance`, `ProvenanceKind`, `tag()`.
- `omega-harness` : `HarnessCatalogContributor` (données embarquées).
- `omega-types` : `Event`, `DurableEvent`, `TelemetryEvent`.

## Décision (à venir)

Implémenter le replay déterministe et l'export OTel, avec provenance partout.

## Justification

- L'infrastructure existe (EventBus, FileEventStore, TraceReplayer, OTel sinks).
- Le replay est déjà partiellement implémenté (`TraceReplayer::summarize`).
- La provenance (`[stated]/[observed]/[inferred]`) est dans `omega-otel` (Provenance, ProvenanceKind, tag).

## Alternatives

- **(a)** Implémenter replay + OTel en une session (4-6h).
- **(b)** Implémenter replay d'abord, OTel ensuite (2 sessions).
- **(c)** Reporter W8 et se concentrer sur W7 (UI) d'abord.

## Conséquence

- Modification de `omega-app/src/store.rs` (replay), `omega-otel/src/sinks.rs` (export), `omega-server/src/routes.rs` (endpoints trace).
- Tests d'intégration : replay déterministe sur session réelle.
- OTel export fonctionnel (gRPC ou HTTP).

## Portée

`omega-app`, `omega-otel`, `omega-server`. Pas de modification de `omega-context` ni `omega-tools`.

## Risques

- Le replay nécessite que tous les événements soient durables (`durable: true`). Les événements éphémères (UI live) ne sont pas rejouables.
- L'export OTel nécessite un collector OTel (ou un mock pour les tests).
- La provenance doit être ajoutée à tous les événements existants (rétrocompatibilité).

## Prérequis

- W7 (UI) pour le Trace Viewer — le viewer consomme les traces.
- W6 (contexte) pour les métriques de compaction dans les traces.
- ADR-0023 (peuplement W6) si les métriques W6 sont dans les traces.
