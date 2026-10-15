import { useEffect, useState } from 'react';
import { getPlugins, getTrace, postRun } from './api';
import type {
  Message,
  PluginInfo,
  RouterDecisionDto,
  RunEvent,
  RunResponse,
  TraceResponse,
} from './protocol';

/**
 * Omega Cockpit — 5 panneaux branchés sur `omega-server` (W7 inc2).
 * Chaque appel API a trois états (chargement / données / erreur) ; si l'API
 * est injoignable, le panneau bascule sur ses fixtures hors-ligne avec un
 * bandeau explicite — jamais un écran vide.
 */

type Panel = 'cockpit' | 'plugins' | 'router' | 'traces' | 'context';

/* --- Fixtures hors-ligne (repli si l'API est injoignable) ------------------ */

const FIXTURE_PLUGINS: PluginInfo[] = [
  { schema_version: 1, id: 'ollama', capability: 'llm-provider', version: '1', active: true },
  { schema_version: 1, id: 'default', capability: 'context-engine', version: '1', active: true },
  { schema_version: 1, id: 'file', capability: 'event-store', version: '1', active: true },
];

const FIXTURE_DECISION: RouterDecisionDto = {
  schema_version: 1,
  model_id: 'ollama/qwen2.5-coder:7b',
  score: 0.91,
  reason: 'task=T2 eligible=1 pareto=1',
};

const FIXTURE_EVENTS: RunEvent[] = [
  { schema_version: 1, run_id: 'live-1', name: 'run.started', payload: null },
  { schema_version: 1, run_id: 'live-1', name: 'step.completed', payload: { operation: 'assemble' } },
  { schema_version: 1, run_id: 'live-1', name: 'step.completed', payload: { operation: 'infer' } },
  { schema_version: 1, run_id: 'live-1', name: 'run.completed', payload: null },
];

function Banner({ kind, text }: { kind: 'loading' | 'offline'; text: string }) {
  return (
    <p role={kind === 'loading' ? 'status' : 'alert'}>
      {kind === 'loading' ? 'Chargement…' : `Hors-ligne : ${text} (repli sur fixtures).`}
    </p>
  );
}

/* --- Panneaux --------------------------------------------------------------- */

function Cockpit({ onRun }: { onRun: (r: RunResponse) => void }) {
  const [input, setInput] = useState('Résume Omega en trois points.');
  const [state, setState] = useState<
    | { kind: 'idle' }
    | { kind: 'loading' }
    | { kind: 'done'; run: RunResponse }
    | { kind: 'error'; message: string }
  >({ kind: 'idle' });

  async function launch() {
    const messages: Message[] = [{ role: 'user', content: input }];
    setState({ kind: 'loading' });
    try {
      const run = await postRun(messages);
      onRun(run);
      setState({ kind: 'done', run });
    } catch (e) {
      setState({ kind: 'error', message: e instanceof Error ? e.message : String(e) });
    }
  }

  return (
    <section>
      <h2>Cockpit</h2>
      <p>
        <label>
          Message :{' '}
          <input
            value={input}
            onChange={(e) => setInput(e.target.value)}
            size={48}
            aria-label="Message à envoyer"
          />
        </label>{' '}
        <button onClick={() => void launch()} disabled={state.kind === 'loading'}>
          Lancer le run
        </button>
      </p>
      {state.kind === 'loading' && <Banner kind="loading" text="" />}
      {state.kind === 'error' && <Banner kind="offline" text={state.message} />}
      {state.kind === 'done' && (
        <div>
          <p>
            Run <code>{state.run.run_id}</code> : <code>{state.run.status}</code>,{' '}
            {state.run.summary.steps_completed}/
            {state.run.summary.steps_completed + state.run.summary.steps_failed} étapes,
            modèle <code>{state.run.decision.model_id}</code>.
          </p>
          <p>
            Raison : <code>{state.run.decision.reason}</code>
          </p>
        </div>
      )}
      {state.kind !== 'done' && (
        <div>
          <p>Exemple hors-ligne (run <code>live-1</code>) :</p>
          <ol>
            {FIXTURE_EVENTS.map((e, i) => (
              <li key={i}>
                <code>{e.name}</code>
              </li>
            ))}
          </ol>
        </div>
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
    <section>
      <h2>Plugin Explorer</h2>
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
              <td>{p.active ? 'oui' : 'non'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

function Router({ lastRun }: { lastRun: RunResponse | null }) {
  return (
    <section>
      <h2>Router Dashboard</h2>
      {lastRun === null ? (
        <div>
          <p>Aucun run cette session — exemple hors-ligne :</p>
          <p>
            Modèle : <code>{FIXTURE_DECISION.model_id}</code> (score{' '}
            {FIXTURE_DECISION.score.toFixed(2)})
          </p>
          <p>
            Raison : <code>{FIXTURE_DECISION.reason}</code>
          </p>
        </div>
      ) : (
        <div>
          <p>
            Run <code>{lastRun.run_id}</code> → modèle :{' '}
            <code>{lastRun.decision.model_id}</code> (score {lastRun.decision.score.toFixed(2)})
          </p>
          <p>
            Raison : <code>{lastRun.decision.reason}</code>
          </p>
        </div>
      )}
    </section>
  );
}

function Traces({ lastRun }: { lastRun: RunResponse | null }) {
  const [runId, setRunId] = useState(lastRun?.run_id ?? 'live-1');
  const [trace, setTrace] = useState<TraceResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      setTrace(await getTrace(runId));
    } catch (e) {
      setTrace(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section>
      <h2>Trace Viewer / Replay</h2>
      <p>Séquence rejouée sans ré-exécuter (cf. `TraceReplayer`).</p>
      <p>
        <label>
          Run :{' '}
          <input value={runId} onChange={(e) => setRunId(e.target.value)} aria-label="Run à rejouer" />
        </label>{' '}
        <button onClick={() => void load()} disabled={loading}>
          Rejouer
        </button>
      </p>
      {loading && <Banner kind="loading" text="" />}
      {error !== null && <Banner kind="offline" text={error} />}
      {trace !== null && (
        <div>
          <p>
            {trace.summary.steps_completed}/{trace.summary.steps_completed + trace.summary.steps_failed}{' '}
            étapes, {trace.summary.total_events} événements, issue{' '}
            <code>{trace.replayed.outcome}</code>
            {trace.replayed.outcome_conflict && ' (conflit : premier marqueur retenu)'}
            {trace.replayed.ignored_events > 0 && ` (${trace.replayed.ignored_events} ignorés)`}
          </p>
          <ol>
            {trace.replayed.steps.map((s) => (
              <li key={`${s.seq}-${s.operation_id}`}>
                <code>{s.operation_id}</code> — {s.status} (seq {s.seq})
              </li>
            ))}
          </ol>
        </div>
      )}
    </section>
  );
}

function Context() {
  return (
    <section>
      <h2>Context Inspector</h2>
      <p>Budget : estimation déterministe, coupes journalisées (`AssembleReport`).</p>
      <p>Inspection live du budget par run : suivi prévu (endpoint dédié, incrément suivant).</p>
    </section>
  );
}

export default function App() {
  const [panel, setPanel] = useState<Panel>('cockpit');
  const [lastRun, setLastRun] = useState<RunResponse | null>(null);
  const panels: Panel[] = ['cockpit', 'plugins', 'router', 'traces', 'context'];
  return (
    <main>
      <h1>Omega Cockpit</h1>
      <nav>
        {panels.map((p) => (
          <button key={p} onClick={() => setPanel(p)} aria-pressed={panel === p}>
            {p}
          </button>
        ))}
      </nav>
      {panel === 'cockpit' && <Cockpit onRun={setLastRun} />}
      {panel === 'plugins' && <Plugins />}
      {panel === 'router' && <Router lastRun={lastRun} />}
      {panel === 'traces' && <Traces lastRun={lastRun} />}
      {panel === 'context' && <Context />}
    </main>
  );
}
