/**
 * Types du protocole omega-server — miroir de `omega-protocol` (DTO serde).
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
