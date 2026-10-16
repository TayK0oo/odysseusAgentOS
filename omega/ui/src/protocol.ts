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

/** Réponse de `POST /api/run` : décision + résumé rejoué (preuve) + réponse du modèle. */
export interface RunResponse {
  run_id: string;
  /** `RunStatus` sérialisé (`omega-core`, kebab-case). */
  status: 'ok' | 'failed' | 'cancelled';
  decision: RouterDecision;
  summary: RunSummary;
  /** Dernière réponse du modèle, relue dans le journal (`infer.completed`). */
  answer?: string;
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

/** Réponse de `GET /api/trace/:run_id` : séquence rejouée + résumé + réponse. */
export interface TraceResponse {
  run_id: string;
  replayed: ReplayedRun;
  summary: RunSummary;
  answer?: string;
}

/** Corps d'erreur JSON de l'API (statuts 4xx/5xx). */
export interface ApiErrorBody {
  schema_version: number;
  error: string;
}

/* --- Miroirs de `omega-server::system` (W7 inc4) -------------------------- */

export interface CrateInfo {
  name: string;
  layer: number;
  role: string;
  deps: string[];
}

export interface CapabilityInfo {
  name: string;
  regime: 'slot' | 'ensemble';
  active: string[];
}

export interface OllamaInfo {
  base_url: string;
  model: string;
}

export interface SystemCounts {
  crates: number;
  capabilities: number;
  capabilities_wired: number;
}

export interface SystemResponse {
  schema_version: number;
  crates: CrateInfo[];
  capabilities: CapabilityInfo[];
  counts: SystemCounts;
  ollama: OllamaInfo;
}

export interface RawModel {
  id: string;
  provider: string;
  context_length?: number | null;
}

export interface QualityAxes {
  reasoning: number;
  code: number;
  agentic: number;
  knowledge: number;
  chat: number;
  vision: number;
}

export interface Capabilities {
  function_calling: boolean;
  multimodal: boolean;
  structured_output: boolean;
  reasoning: boolean;
}

export interface ModelRecord {
  id: string;
  provider: string;
  context_length: number;
  cost_input_per_1m: number;
  cost_output_per_1m: number;
  tokens_per_second: number;
  ttft_ms: number;
  axes: QualityAxes;
  capabilities: Capabilities;
  tier: 'FastCheap' | 'Standard' | 'Elite' | 'SelfHosted';
}

export interface ModelsResponse {
  schema_version: number;
  installed: RawModel[];
  record: ModelRecord;
}

export interface ClassifyResponse {
  schema_version: number;
  task: string;
  task_code: string;
  weights: Record<string, number>;
}

export interface RunEntry {
  run_id: string;
  events: number;
  steps_completed: number;
  steps_failed: number;
  outcome: 'completed' | 'failed' | 'unknown';
}

export interface RunsResponse {
  schema_version: number;
  runs: RunEntry[];
}

export interface PolicyRule {
  prefix: string;
  decision: 'allow' | 'prompt' | 'forbidden';
  reason?: string | null;
}

export interface PolicyResponse {
  schema_version: number;
  strict: boolean;
  rules: PolicyRule[];
  default: string;
  note: string;
}

export interface EvaluateResponse {
  schema_version: number;
  command: string;
  decision: string;
  reason?: string | null;
  compound: boolean;
  substitution: boolean;
  effective: string;
}

export interface ContextSample {
  chars: number;
  tokens: number;
}

export interface ContextResponse {
  schema_version: number;
  estimator: string;
  max_tokens: number;
  required_context: number;
  sample: ContextSample[];
}

/* --- Miroirs de `omega-server::system` (W7 inc5) -------------------------- */

export interface RegistryEntryDto {
  id: string;
  version: string;
  priority: number;
  quarantined: boolean;
  kind: 'typed' | 'trait-object';
}

export interface RegistryCapabilityDto {
  name: string;
  regime: 'slot' | 'ensemble';
  entries: RegistryEntryDto[];
}

export interface RegistryResponse {
  schema_version: number;
  capabilities: RegistryCapabilityDto[];
}

export interface ConfigResponse {
  schema_version: number;
  port: number;
  ollama_url: string;
  ollama_model: string;
  event_dir: string;
  max_tokens: number;
  ui_dir?: string | null;
  ui_served: boolean;
}

export interface RouteInfo {
  method: string;
  path: string;
  description: string;
}

export interface RoutesResponse {
  schema_version: number;
  routes: RouteInfo[];
}

export interface DurableEvent {
  run_id: string;
  seq: number;
  name: string;
  operation_id?: string | null;
  payload: unknown;
}

export interface RunEventsResponse {
  schema_version: number;
  run_id: string;
  events: DurableEvent[];
}

export interface OverviewResponse {
  schema_version: number;
  status: string;
  config: ConfigResponse;
  counts: SystemCounts;
  capabilities: CapabilityInfo[];
  crate_count: number;
  recent_runs: RunEntry[];
  model_record: ModelRecord;
  harness: HarnessCounts;
  dissection: DissectionTotals;
}

/* --- Miroirs de `omega-harness` (reverse-engineering) --------------------- */

export type PrincipleStatus = 'adopted' | 'planned' | 'rejected';
export type RustCost = 'S' | 'M' | 'L';

export interface Principle {
  id: string;
  name: string;
  what: string;
  why: string;
  status: PrincipleStatus;
  omega?: string | null;
  rust_cost: RustCost;
  refs: string[];
}

export interface Harness {
  id: string;
  name: string;
  editor: string;
  oss: boolean;
  language: string;
  architecture: string;
  principles: Principle[];
}

export interface HarnessCatalog {
  schema_version: number;
  note: string;
  harnesses: Harness[];
}

export interface HarnessCounts {
  harnesses: number;
  principles: number;
  adopted: number;
  planned: number;
  rejected: number;
}

/* --- Miroirs de `omega-harness::AppDissection` ---------------------------- */

export type IntegrationMode = 'plugin' | 'service' | 'native' | 'rejected';

export interface Functionality {
  id: string;
  domain: string;
  name: string;
  what: string;
  how: string;
  omega_capability?: string | null;
  omega_port?: string | null;
  omega_adapter?: string | null;
  integration: IntegrationMode;
  status: PrincipleStatus;
  rust_cost: RustCost;
  refs: string[];
}

export interface AppDissection {
  schema_version: number;
  id: string;
  name: string;
  editor: string;
  oss: boolean;
  language: string;
  license: string;
  repo?: string | null;
  architecture: string;
  entrypoints: string[];
  functionalities: Functionality[];
}

export interface DissectionTotals {
  apps: number;
  functionalities: number;
  adopted: number;
  planned: number;
  rejected: number;
}

export interface DissectionsResponse {
  schema_version: number;
  apps: AppDissection[];
  totals: DissectionTotals;
}
