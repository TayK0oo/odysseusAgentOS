/**
 * Client HTTP de l'API `omega-server`.
 * Même origine (`/api/...`) : en production l'UI est servie par le serveur ;
 * en dev, Vite proxifie vers `localhost:8787`. Toute erreur remonte en exception.
 */

import { SCHEMA_VERSION } from './protocol';
import type {
  ClassifyResponse,
  ConfigResponse,
  DissectionsResponse,
  HarnessCatalog,
  OverviewResponse,
  RegistryResponse,
  RoutesResponse,
  RunEventsResponse,
  ContextResponse,
  EvaluateResponse,
  HealthResponse,
  Message,
  ModelsResponse,
  PluginInfo,
  PolicyResponse,
  RunsResponse,
  RunResponse,
  SystemResponse,
  TraceResponse,
} from './protocol';

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(url, init);
  } catch (e) {
    throw new Error(`API injoignable (${url}) : ${e instanceof Error ? e.message : e}`);
  }
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`API ${res.status} (${url}) : ${body.slice(0, 200)}`);
  }
  return (await res.json()) as T;
}

function postJson<T>(url: string, payload: unknown): Promise<T> {
  return fetchJson<T>(url, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

export const getHealth = () => fetchJson<HealthResponse>('/api/health');
export const getPlugins = () => fetchJson<PluginInfo[]>('/api/plugins');
export const getSystem = () => fetchJson<SystemResponse>('/api/system');
export const getModels = () => fetchJson<ModelsResponse>('/api/models');
export const getRuns = () => fetchJson<RunsResponse>('/api/runs');
export const getPolicy = () => fetchJson<PolicyResponse>('/api/policy');
export const getContext = () => fetchJson<ContextResponse>('/api/context');

export const postRun = (messages: Message[], runId?: string) =>
  postJson<RunResponse>('/api/run', {
    schema_version: SCHEMA_VERSION,
    messages,
    run_id: runId ?? null,
  });

export const postClassify = (text: string) =>
  postJson<ClassifyResponse>('/api/classify', { text });

export const postEvaluate = (command: string) =>
  postJson<EvaluateResponse>('/api/policy/evaluate', { command });

export const getTrace = (runId: string) =>
  fetchJson<TraceResponse>(`/api/trace/${encodeURIComponent(runId)}`);

/* --- W7 inc5 : observabilité complète ------------------------------------- */

export const getOverview = () => fetchJson<OverviewResponse>('/api/overview');
export const getRegistry = () => fetchJson<RegistryResponse>('/api/registry');
export const getConfig = () => fetchJson<ConfigResponse>('/api/config');
export const getRoutes = () => fetchJson<RoutesResponse>('/api/routes');
export const getRunEvents = (runId: string) =>
  fetchJson<RunEventsResponse>(`/api/runs/${encodeURIComponent(runId)}/events`);

export const getHarnesses = () => fetchJson<HarnessCatalog>('/api/harnesses');

export const getDissections = () => fetchJson<DissectionsResponse>('/api/dissections');
