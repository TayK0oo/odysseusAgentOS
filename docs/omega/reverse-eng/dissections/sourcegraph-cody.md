# Dissection — Sourcegraph Cody (Sourcegraph, Apache-2.0, TypeScript + Rust)

> Source : clone `--depth 1` de https://github.com/sourcegraph/cody-public-snapshot (archive) dans `/home/tayk0oo/omega-re/sourcegraph-cody`.
> Donnees : `omega/crates/omega-harness/data/apps/sourcegraph-cody.json` (8 fonctionnalites).
> Preuve REA machine : voir `HARNESS-INTAKE.md`.

## 1. Fiche

| Champ | Valeur |
|---|---|
| Editeur | Sourcegraph |
| Langage | TypeScript + Rust |
| Repo | https://github.com/sourcegraph/cody-public-snapshot (archive) |
| Architecture | Assistant IDE multi-plateformes (snapshot public archive) : DeepCody agent chat agentic (ProcessManager de steps), CodyToolFactory, guardrails Sourcegraph (verif du code genere), cody-ignore, protocole OpenCtx de fournisseurs de contexte, recordings de sessions rejeu, protocole agent documente + bindings Rust, auth/secret storage par IDE. |

## 2. Inventaire fonctionnel

### `cg-deep-cody` — DeepCody : agent chat agentic orchestrateur de steps

- **domaine** : agents · **capacite** : agent-loop · **port** : — · **cout** : M · **statut** : planned
- **what** : Un agent agentic (DeepCody) qui orchestre des steps de traitement (ProcessManager) : mentions categorisees, contexte, fork/transcription, telemetrie otel par span.
- **how** : vscode/src/chat/agentic/DeepCody.ts:1-30 (PromptMixin, ProcessType, wrapInActiveSpan) + ProcessManager.ts:5 (class). Omega : AgentLoop + LocalSubagentRunner deja couverts.
- **adaptateur Omega** : omega-app::AgentLoop (steps + mentions)
- **preuves** : `vscode/src/chat/agentic/DeepCody.ts:1-30` · `vscode/src/chat/agentic/ProcessManager.ts:5`

### `cg-tools-factory` — CodyToolFactory/Provider : outils agentic branches au chat

- **domaine** : extension · **capacite** : tool-surface · **port** : ToolSurface · **cout** : S · **statut** : planned
- **what** : Une factory/packages de outils agentic (CodyTool) declaree par config, fournie au chat.
- **how** : vscode/src/chat/agentic/CodyToolFactory.ts + CodyToolProvider.ts + CodyTool.ts + config.ts. Omega : ToolSurface couvert.
- **adaptateur Omega** : omega-context::ToolSurfaceIndex
- **preuves** : `vscode/src/chat/agentic/CodyToolFactory.ts`

### `cg-guardrails` — Guardrails Sourcegraph : client de verif du code genere

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Un client guardrails (SourcegraphGuardrailsClient) qui verifie le code genere contre la base Sourcegraph (similarite/licence) avant acceptation.
- **how** : lib/shared/src/guardrails/client.ts:21 (class SourcegraphGuardrailsClient) + config.ts. Omega : ExecPolicy ; le pattern 'verification du code genere' est unique du corpus : a noter.
- **adaptateur Omega** : omega-core::ExecPolicy (guardrails du code genere)
- **preuves** : `lib/shared/src/guardrails/client.ts:17-21`

### `cg-cody-ignore` — cody-ignore : filtre le contexte esquive des fichiers ignores

- **domaine** : securite · **capacite** : exec-policy · **port** : ExecPolicy · **cout** : S · **statut** : planned
- **what** : Un filtre cody-ignore couvert a la fois partagé (shared/src/cody-ignore) et l'IDE : les fichiers ignores n'entrent pas au contexte/notification si feature ignore.
- **how** : lib/shared/src/cody-ignore/ + vscode/src/cody-ignore/context-filter.ts:3-11 (CodyIgnoreFeature, notification). Omega : ExecPolicy/ExecBlocklist.
- **adaptateur Omega** : omega-core::ExecPolicy (cody-ignore)
- **preuves** : `vscode/src/cody-ignore/context-filter.ts:3-11`

### `cg-openctx` — OpenCtx : protocole de fournisseurs de contexte externes

- **domaine** : contexte · **capacite** : context-engine · **port** : CatalogSource · **cout** : M · **statut** : planned
- **what** : OpenCtx : un protocole de fournisseurs de contexte externes (docs, issues, web) branche au chat via mentions (openctx.ts).
- **how** : vscode/src/context/openctx.ts + lib/shared/src/context/openctx/. Omega : CatalogSource (fournisseurs externes) ; retenu : le protocole de mentions.
- **adaptateur Omega** : omega-context (protocole de contexte externe)
- **preuves** : `vscode/src/context/openctx.ts`

### `cg-chat-recordings` — Recordings : capture de sessions chat pour rejeu/regression

- **domaine** : observabilite · **capacite** : telemetry-sink · **port** : TelemetrySink · **cout** : S · **statut** : planned
- **what** : Un module recordings : event streams de sessions chat enregistrees (dossier recordings/ + agent/recordings) pour rejeu/QA.
- **how** : recordings/ + agent/recordings/. Omega : EventStore (rejeu) deja le pattern trace Omega.
- **adaptateur Omega** : omega-otel + EventStore (rejeu des sessions)
- **preuves** : `recordings` · `agent/recordings`

### `cg-agent-protocol` — Agent protocol : bindings BinRS + protocol.md document

- **domaine** : configuration · **capacite** : config-source · **port** : — · **cout** : M · **statut** : planned
- **what** : L'agent itself exposes un protocole AI-native : agent/protocol.md + bindings BinRS (Rust bindings).
- **how** : agent/protocol.md + agent/bindings/. Omega : omega-server (protocole de browsing) ; retenu : le protocole documente.
- **adaptateur Omega** : omega-server (protocol.md + BindBinRS)
- **preuves** : `agent/protocol.md` · `agent/bindings`

### `cg-auth-handlers` — AgentAuthHandler/SecretStorage : auth + secrets par IDE

- **domaine** : securite · **capacite** : exec-policy · **port** : PermissionPolicy · **cout** : S · **statut** : planned
- **what** : Un auth handler + secret storage par IDE avec tests.
- **how** : agent/src/AgentAuthHandler.ts + AgentSecretStorage.ts. Omega : PolicyProvider couvert.
- **adaptateur Omega** : omega-core::PermissionPolicy (auth IDE)
- **preuves** : `agent/src/AgentAuthHandler.ts` · `agent/src/AgentSecretStorage.ts`

