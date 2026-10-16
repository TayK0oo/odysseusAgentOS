import { useEffect, useState } from 'react';

/** Composants d'affichage partagés par tous les panneaux. */

export function Banner({ kind, text }: { kind: 'loading' | 'error' | 'offline'; text: string }) {
  if (kind === 'loading') return <p className="banner loading">{text || 'Chargement…'}</p>;
  if (kind === 'error') return <p className="banner error">Erreur : {text}</p>;
  return <p className="banner offline">{text}</p>;
}

export function Card({ k, v }: { k: string; v: React.ReactNode }) {
  return (
    <div className="card">
      <div className="k">{k}</div>
      <div className="v">{v}</div>
    </div>
  );
}

export function Chip({ children, tone }: { children: React.ReactNode; tone?: 'ok' | 'warn' | 'ko' }) {
  return <span className={`chip ${tone ?? ''}`}>{children}</span>;
}

export function Bar({ value, max = 1 }: { value: number; max?: number }) {
  const pct = Math.max(0, Math.min(1, max === 0 ? 0 : value / max)) * 100;
  return (
    <div className="bar">
      <i style={{ width: `${pct}%` }} />
    </div>
  );
}

/** Hook minimal : charge une fois (ou à chaque changement de `deps`). */
export function useAsync<T>(fn: () => Promise<T>, deps: unknown[] = []) {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    fn()
      .then((d) => {
        if (!cancelled) setData(d);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
  return { data, error, loading };
}

export function Async<T>({
  state,
  children,
}: {
  state: { data: T | null; error: string | null; loading: boolean };
  children: (d: T) => React.ReactNode;
}) {
  if (state.loading) return <Banner kind="loading" text="" />;
  if (state.error !== null) return <Banner kind="error" text={state.error} />;
  if (state.data === null) return null;
  return <>{children(state.data)}</>;
}

/** Horodatage court pour l'affichage (ms epoch -> HH:MM:SS). */
export function clock(ms?: number): string {
  if (ms === undefined) return '—';
  const d = new Date(ms);
  return d.toLocaleTimeString();
}

/** Ton d'une décision de politique. */
export function decisionTone(d: string): 'ok' | 'warn' | 'ko' {
  return d === 'allow' ? 'ok' : d === 'prompt' ? 'warn' : 'ko';
}
