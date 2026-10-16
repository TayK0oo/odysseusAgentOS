import { useEffect, useState } from 'react';
import { getHealth, getPlugins, getTrace, postRun } from './api';
import type {
  Message,
  PluginInfo,
  RouterDecision,
  RunResponse,
  TraceResponse,
} from './protocol';

/**
 * Omega Cockpit — 5 panneaux branchés sur `omega-server`.
 *
 * Chaque panneau a trois états (chargement / données / erreur) ; si l'API est
 * injoignable, il bascule sur des fixtures hors-ligne avec un bandeau explicite
 * — jamais un écran vide.
 */

type Panel = 'cockpit' | 'plugins' | 'router' | 'traces' | 'context';

const PANELS: { id: Panel; label: string }[] = [
  { id: 'cockpit', label: 'Cockpit' },
  { id: 'plugins', label: 'Plugins' },
  { id: 'router', label: 'Router' },
  { id: 'traces', label: 'Traces' },
  { id: 'context', label: 'Contexte' },
];

/* --- Fixtures hors-ligne (repli si l'API est injoignable) ------------------ */

const FIXTURE_PLUGINS: PluginInfo[] = [
  { schema_version: 1, id: 'ollama', capability: 'llm-provider', version: '1', active: true },
  { schema_version: 1, id: 'default', capability: 'context-engine', version: '1', active: true },
  { schema_version: 1, id: 'file', capability: 'event-store', version: '1', active: true },
];

const FIXTURE_DECISION: RouterDecision = {
  model_id: 'ollama/qwen2.5-coder:7b',
  score: 0.91,
  reason: 'task=T2 eligible=1 pareto=1 selected=ollama/qwen2.5-coder:7b',
};

/* --- Petits composants ----------------------------------------------------- */

function Banner({ kind, text }: { kind: 'loading' | 'error' | 'offline'; text: string }) {
  if (kind === 'loading') return <p className="banner loading">{text || 'Chargement…'}</p>;
  if (kind === 'error') return <p className="banner error">Erreur : {text}</p>;
  return <p className="banner offline">Hors-ligne : {text} — repli sur fixtures.</p>;
}

function Card({ k, v }: { k: string; v: React.ReactNode }) {
  return (
    <div className="card">
      <div className="k">{k}</div>
      <div className="v">{v}</div>
    </div>
  );
}

function StatusBadge({ status }: { status: string }) {
  return <span className={`badge ${status === 'ok' ? 'ok' : 'ko'}`}>{status}</span>;
}

function DecisionCards({ d }: { d: RouterDecision }) {
  return (
    <div className="cards">
      <Card k="Modèle" v={<code>{d.model_id}</code>} />
      <Card
        k="Score"
        v={
          <>
            {d.score.toFixed(3)}
            <div className="bar">
              <i style={{ width: `${Math.max(0, Math.min(1, d.score)) * 100}%` }} />
            </div>
          </>
        }
      />
    </div>
  );
}

function Answer({ text }: { text?: string }) {
  if (!text) return null;
  return (
    <div className="answer">
      <h3>Réponse du modèle</h3>
      {text}
    </div>
  );
}

/* --- Panneaux -------------------------------------------------------------- */

function Cockpit({ onRun }: { onRun: (r: RunResponse) => void }) {
  const [input, setInput] = useState('Résume Omega en trois points.');
  const [busy, setBusy] = useState(false);
  const [run, setRun] = useState<RunResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function launch() {
    if (busy || input.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      const messages: Message[] = [{ role: 'user', content: input }];
      const r = await postRun(messages);
      setRun(r);
      onRun(r);
    } catch (e) {
      setRun(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel">
      <h2>Cockpit</h2>
      <p className="hint">
        Un run complet : classification déterministe → sélection Pareto → assemblage →
        inférence locale (coût nul) → journal durable.
      </p>
      <div className="row">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          aria-label="Message à envoyer"
          onKeyDown={(e) => {
            if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) void launch();
          }}
        />
        <button className="primary" onClick={() => void launch()} disabled={busy}>
          {busy ? 'Run…' : 'Lancer le run'}
        </button>
      </div>
      {busy && <Banner kind="loading" text="Inférence locale en cours…" />}
      {error !== null && <Banner kind="error" text={error} />}
      {run !== null && (
        <>
          <div className="cards">
            <Card k="Run" v={<code>{run.run_id}</code>} />
            <Card k="Statut" v={<StatusBadge status={run.status} />} />
            <Card
              k="Étapes"
              v={`${run.summary.steps_completed}/${run.summary.steps_completed + run.summary.steps_failed}`}
            />
            <Card k="Événements journal" v={run.summary.total_events} />
          </div>
          <DecisionCards d={run.decision} />
          <p className="hint">
            Raison du routage : <code>{run.decision.reason}</code>
          </p>
          <Answer text={run.answer} />
        </>
      )}
    </section>
  );
}

function Plugins() {
  const [plugins, setPlugins] = useState<PluginInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getPlugins()
      .then((p) => {
        if (!cancelled) setPlugins(p);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const shown = plugins ?? FIXTURE_PLUGINS;
  return (
    <section className="panel">
      <h2>Plugin Explorer</h2>
      <p className="hint">Capacités câblées dans le registre (quarantaine déjà appliquée).</p>
      {plugins === null && error === null && <Banner kind="loading" text="" />}
      {error !== null && <Banner kind="offline" text={error} />}
      <table>
        <thead>
          <tr>
            <th>Plugin</th>
            <th>Capacité</th>
            <th>Actif</th>
          </tr>
        </thead>
        <tbody>
          {shown.map((p) => (
            <tr key={`${p.capability}/${p.id}`}>
              <td>
                <code>{p.id}</code>
              </td>
              <td>
                <code>{p.capability}</code>
              </td>
              <td>
                <span className={`badge ${p.active ? 'ok' : 'ko'}`}>
                  {p.active ? 'oui' : 'non'}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

function RouterPanel({ lastRun }: { lastRun: RunResponse | null }) {
  const d = lastRun?.decision ?? FIXTURE_DECISION;
  return (
    <section className="panel">
      <h2>Router Dashboard</h2>
      <p className="hint">
        Décision de routage : classifieur de tâche déterministe (zéro LLM) puis sélecteur
        Pareto + préférence self-hosted (marge 0.05).
      </p>
      {lastRun === null && <Banner kind="offline" text="aucun run cette session — exemple" />}
      <DecisionCards d={d} />
      <div className="cards">
        <Card k="Run" v={lastRun ? <code>{lastRun.run_id}</code> : '—'} />
        <Card k="Tâche" v={<code>{d.reason.split(' ')[0]}</code>} />
      </div>
      <p className="hint">
        Raison complète : <code>{d.reason}</code>
      </p>
    </section>
  );
}

function Traces({ lastRun }: { lastRun: RunResponse | null }) {
  const [runId, setRunId] = useState(lastRun?.run_id ?? '');
  const [trace, setTrace] = useState<TraceResponse | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (lastRun) setRunId(lastRun.run_id);
  }, [lastRun]);

  async function load() {
    if (runId.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      setTrace(await getTrace(runId.trim()));
    } catch (e) {
      setTrace(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel">
      <h2>Trace Viewer / Replay</h2>
      <p className="hint">
        Rejoue un journal sans ré-exécuter les effets (preuve de traçabilité).
      </p>
      <div className="row">
        <input
          type="text"
          value={runId}
          placeholder="identifiant de run (ex. run-1)"
          onChange={(e) => setRunId(e.target.value)}
          aria-label="Run à rejouer"
          onKeyDown={(e) => {
            if (e.key === 'Enter') void load();
          }}
        />
        <button className="primary" onClick={() => void load()} disabled={busy}>
          {busy ? '…' : 'Rejouer'}
        </button>
      </div>
      {busy && <Banner kind="loading" text="" />}
      {error !== null && <Banner kind="error" text={error} />}
      {trace !== null && (
        <>
          <div className="cards">
            <Card k="Issue" v={<code>{trace.replayed.outcome}</code>} />
            <Card
              k="Étapes"
              v={`${trace.summary.steps_completed}/${trace.summary.steps_completed + trace.summary.steps_failed}`}
            />
            <Card k="Événements" v={trace.summary.total_events} />
            <Card k="Ignorés" v={trace.summary.ignored_events} />
          </div>
          {trace.replayed.outcome_conflict && (
            <Banner kind="offline" text="double marqueur terminal — premier retenu" />
          )}
          <ol className="steps">
            {trace.replayed.steps.map((s) => (
              <li key={`${s.seq}-${s.operation_id}`}>
                <span className="seq">#{s.seq}</span>
                <code>{s.operation_id}</code>
                <span className={`badge ${s.status === 'completed' ? 'ok' : 'ko'}`}>{s.status}</span>
              </li>
            ))}
          </ol>
          <Answer text={trace.answer} />
        </>
      )}
    </section>
  );
}

function Context() {
  return (
    <section className="panel">
      <h2>Context Inspector</h2>
      <p className="hint">
        Économie de contexte : estimation déterministe (chars/4), budget avec coupes
        journalisées, tool search BM25 + gate rarest-token, progressive disclosure des skills.
      </p>
      <div className="cards">
        <Card k="Estimation tokens" v={<code>ceil(chars / 4)</code>} />
        <Card k="Budget défaut" v={<code>8000</code>} />
        <Card k="Contexte exigé" v={<code>4000</code>} />
      </div>
      <p className="hint">
        Inspection live du contexte par run : endpoint dédié prévu (incrément suivant).
      </p>
    </section>
  );
}

export default function App() {
  const [panel, setPanel] = useState<Panel>('cockpit');
  const [lastRun, setLastRun] = useState<RunResponse | null>(null);
  const [online, setOnline] = useState<boolean | null>(null);

  useEffect(() => {
    getHealth()
      .then(() => setOnline(true))
      .catch(() => setOnline(false));
  }, []);

  const statusText =
    online === null ? 'Connexion…' : online ? 'API en ligne' : 'API injoignable';

  return (
    <div className="app">
      <header className="top">
        <h1>
          Omega <span>Cockpit</span>
        </h1>
        <div className="status" role="status">
          <span className={`dot ${online === null ? '' : online ? 'online' : 'offline'}`} />
          {statusText}
        </div>
      </header>
      <nav className="tabs">
        {PANELS.map((p) => (
          <button key={p.id} onClick={() => setPanel(p.id)} aria-pressed={panel === p.id}>
            {p.label}
          </button>
        ))}
      </nav>
      {panel === 'cockpit' && <Cockpit onRun={setLastRun} />}
      {panel === 'plugins' && <Plugins />}
      {panel === 'router' && <RouterPanel lastRun={lastRun} />}
      {panel === 'traces' && <Traces lastRun={lastRun} />}
      {panel === 'context' && <Context />}
    </div>
  );
}
