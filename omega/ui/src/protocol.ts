/**
 * Types du protocole omega-server — miroir de `omega-protocol` (DTO serde)
 * + miroir des DTO de `omega-server` (`dto.rs`) et de `omega-otel::replay`.
 * Source unique de vérité côté Rust ; ce fichier est maintenu à la main en
 * attendant la génération (`ts-rs`, OMEGA-BLUEPRINT §2.1 — dette W7 documentée).
 */

export const SCHEMA_VERSION = 1;

export type Role = 'system' | 'user' | 'assistant' | 'tool';

export interface Message {
  role: Role;
  content: string;
}

export interface RunRequest {
  schema_version: number;
  run_id?: string;
  messages: Message[];
}

export interface RunEvent {
  schema_version: number;
  run_id: string;
  name: string;
  payload: unknown;
}

export interface PluginInfo {
  schema_version: number;
  id: string;
  capability: string;
  version: string;
  active: boolean;
}

export interface RouterDecisionDto {
  schema_version: number;
  model_id: string;
  score: number;
  reason: string;
}

export interface ManifestDto {
  schema_version: number;
  id: string;
  capability: string;
  version: string;
  priority: number;
}

/* --- Miroirs de `omega-server` (W7 inc2) ---------------------------------- */

export interface HealthResponse {
  status: string;
  schema_version: number;
}

/** Décision de routage (`omega_types::RouterDecision`, sérialisé tel quel). */
export interface RouterDecision {
  model_id: string;
  score: number;
  reason: string;
}

/** Réponse de `POST /api/run` : décision + résumé rejoué (preuve). */
export interface RunResponse {
  run_id: string;
  /** `RunStatus` sérialisé (`omega-core`, kebab-case). */
  status: 'ok' | 'failed' | 'cancelled';
  decision: RouterDecision;
  summary: RunSummary;
}

/* --- Miroirs de `omega-otel::replay` (W8, rejoué sans ré-exécuter) --------- */

export type ReplayedStepStatus = 'completed' | 'failed';

export interface ReplayedStep {
  operation_id: string;
  status: ReplayedStepStatus;
  seq: number;
}

export type ReplayedOutcome = 'completed' | 'failed' | 'unknown';

export interface ReplayedRun {
  run_id: string;
  steps: ReplayedStep[];
  outcome: ReplayedOutcome;
  ignored_events: number;
  outcome_conflict: boolean;
}

/** Résumé déterministe d'un journal (`TraceReplayer::summarize`). */
export interface RunSummary {
  run_id: string;
  total_events: number;
  steps_completed: number;
  steps_failed: number;
  outcome: ReplayedOutcome;
  ignored_events: number;
  outcome_conflict: boolean;
}

/** Réponse de `GET /api/trace/:run_id` : séquence rejouée + résumé. */
export interface TraceResponse {
  run_id: string;
  replayed: ReplayedRun;
  summary: RunSummary;
}

/** Corps d'erreur JSON de l'API (statuts 4xx/5xx). */
export interface ApiErrorBody {
  schema_version: number;
  error: string;
}
