# W8 — Traçabilité (`omega-otel`)

> **Workstream :** W8 · **Statut :** 🟢 incrément 1 · **Date :** 2026-10-15
> **Sources :** `OMEGA-BLUEPRINT.md` §7, `00-VISION-OMEGA.md` §8, `W3-app-durable.md`.
> **Provenance :** `[observed]` (implémenté) · `[inferred]` · `[todo]`.

---

## 1. Objectif

Rendre **toute décision traçable et rejouable** : provenance taguée, télémétrie nommée,
sinks, rejeu de journaux sans ré-exécuter les effets.

## 2. Périmètre incrément 1

| Élément | Rôle |
|---|---|
| `Provenance` / `ProvenanceKind` | Tags `[stated]` / `[observed]` / `[inferred]` + source |
| `MetricName` | Noms hiérarchiques validés `omega.<domaine>.<objet>[.<détail>]` |
| `StdoutSink` / `FileSink` | Implémentations du port `TelemetrySink` (live / JSONL) |
| `TraceReplayer` | Rejoue un journal → séquence + issue, **sans ré-exécuter** |
| `RunSummary` | Résumé déterministe (événements, étapes OK/KO, issue) |
| `OtelContributor` | Câble les sinks (capacité `telemetry-sink`, régime *ensemble*) |

## 3. Contrats

- Un nom de métrique invalide → erreur typée (`Config`) à la construction.
- `FileSink` : une ligne JSON par événement, émissions concurrentes sérialisées.
- `TraceReplayer::replay` : pure fonction du journal (tri par `seq`) ; `summarize`
  déterministe (même journal → même résumé).

## 4. Tests d'acceptation

| # | Critère | Statut |
|---|---|---|
| (a) | Provenance sérialisée en tags stables + source portée | ✅ |
| (b) | Noms canoniques acceptés, noms invalides rejetés | ✅ |
| (c) | `FileSink` : round-trip + persistance | ✅ |
| (d) | Replay : séquence reconstruite, issue détectée (OK et Failed) | ✅ |
| (e) | Résumé déterministe | ✅ |
| (f) | Contributeur : sinks résolvables comme ports | ✅ |

## 5. Reste (incrément 2)

- Export OTLP (adaptateur distant du port `TelemetrySink`).
- Cibles log-only / trace-safe (séparation PII).
- `Trace Viewer` UI branché sur `RunSummary` + replay (W7).
