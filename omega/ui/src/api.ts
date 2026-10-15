/**
 * Client HTTP de l'API `omega-server` (W7 inc2).
 * Même origine (`/api/...`, proxy Vite vers `localhost:8787` en dev) ;
 * toute erreur réseau/HTTP remonte en exception → l'appelant bascule sur
 * les fixtures hors-ligne (`App.tsx`).
 */

import { SCHEMA_VERSION } from './protocol';
import type {
  HealthResponse,
  Message,
  PluginInfo,
  RunResponse,
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
  const data = (await res.json()) as T;
  return data;
}

export function getHealth(): Promise<HealthResponse> {
  return fetchJson<HealthResponse>('/api/health');
}

export function getPlugins(): Promise<PluginInfo[]> {
  return fetchJson<PluginInfo[]>('/api/plugins');
}

export function postRun(messages: Message[], runId?: string): Promise<RunResponse> {
  return fetchJson<RunResponse>('/api/run', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      schema_version: SCHEMA_VERSION,
      messages,
      run_id: runId ?? null,
    }),
  });
}

export function getTrace(runId: string): Promise<TraceResponse> {
  return fetchJson<TraceResponse>(`/api/trace/${encodeURIComponent(runId)}`);
}
