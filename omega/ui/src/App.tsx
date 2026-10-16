import { useEffect, useState } from 'react';
import {
  getContext,
  getHealth,
  getModels,
  getPlugins,
  getPolicy,
  getRuns,
  getSystem,
  getTrace,
  postClassify,
  postEvaluate,
  postRun,
} from './api';
import type {
  ClassifyResponse,
  ContextResponse,
  EvaluateResponse,
  Message,
  ModelsResponse,
  PluginInfo,
  PolicyResponse,
  RunEntry,
  RunResponse,
  SystemResponse,
  TraceResponse,
} from './protocol';

/**
 * Omega Cockpit — inspection complète du harnais.
 *
 * 10 panneaux branchés sur `omega-server` (origine unique) : chaque brique du
 * système est observable (architecture, capacités, modèles, routage, journal,
 * politique d'exécution, contexte). Les appels API ont trois états
 * (chargement / données / erreur) ; le repli est toujours explicite.
 */

type Panel =
  | 'cockpit'
  | 'system'
  | 'plugins'
  | 'models'
  | 'router'
  | 'classify'
  | 'traces'
  | 'events'
  | 'sandbox'
  | 'context';

const PANELS: { id: Panel; label: string }[] = [
  { id: 'cockpit', label: 'Cockpit' },
  { id: 'system', label: 'Système' },
  { id: 'plugins', label: 'Plugins' },
  { id: 'models', label: 'Modèles' },
  { id: 'router', label: 'Routeur' },
  { id: 'classify', label: 'Classifieur' },
  { id: 'traces', label: 'Traces' },
  { id: 'events', label: 'Événements' },
  { id: 'sandbox', label: 'Sandbox' },
  { id: 'context', label: 'Contexte' },
];

const FIXTURE_PLUGINS: PluginInfo[] = [
  { schema_version: 1, id: 'ollama', capability: 'llm-provider', version: '1', active: true },
  { schema_version: 1, id: 'default', capability: 'context-engine', version: '1', active: true },
  { schema_version: 1, id: 'file', capability: 'event-store', version: '1', active: true },
];

/* --- Composants partagés --------------------------------------------------- */

function Banner({ kind, text }: { kind: 'loading' | 'error' | 'offline'; text: string }) {
  if (kind === 'loading') return <p className="banner loading">{text || 'Chargement…'}</p>;
  if (kind === 'error') return <p className="banner error">Erreur : {text}</p>;
  return <p className="banner offline">{text}</p>;
}

function Card({ k, v }: { k: string; v: React.ReactNode }) {
  return (
    <div className="card">
      <div className="k">{k}</div>
      <div className="v">{v}</div>
    </div>
  );
}

function Chip({ children, tone }: { children: React.ReactNode; tone?: 'ok' | 'warn' | 'ko' }) {
  return <span className={`chip ${tone ?? ''}`}>{children}</span>;
}

function Bar({ value, max = 1 }: { value: number; max?: number }) {
  const pct = Math.max(0, Math.min(1, max === 0 ? 0 : value / max)) * 100;
  return (
    <div className="bar">
      <i style={{ width: `${pct}%` }} />
    </div>
  );
}

/** Hook minimal : charge une fois, expose chargement/erreur/données. */
function useAsync<T>(fn: () => Promise<T>) {
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
  }, []);
  return { data, error, loading };
}

function Async<T>({
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

/* --- Cockpit --------------------------------------------------------------- */

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
        Run complet : classification déterministe → sélection Pareto → assemblage →
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
            <Card k="Statut" v={<Chip tone={run.status === 'ok' ? 'ok' : 'ko'}>{run.status}</Chip>} />
            <Card
              k="Étapes"
              v={`${run.summary.steps_completed}/${run.summary.steps_completed + run.summary.steps_failed}`}
            />
            <Card k="Événements" v={run.summary.total_events} />
          </div>
          <div className="cards">
            <Card k="Modèle" v={<code>{run.decision.model_id}</code>} />
            <Card
              k="Score"
              v={
                <>
                  {run.decision.score.toFixed(3)}
                  <Bar value={run.decision.score} />
                </>
              }
            />
          </div>
          <p className="hint">
            Raison : <code>{run.decision.reason}</code>
          </p>
          {run.answer && (
            <div className="answer">
              <h3>Réponse du modèle</h3>
              {run.answer}
            </div>
          )}
        </>
      )}
    </section>
  );
}

/* --- Système --------------------------------------------------------------- */

function SystemPanel() {
  const state = useAsync(getSystem);
  return (
    <section className="panel">
      <h2>Architecture du système</h2>
      <p className="hint">
        Workspace Rust (crates par couche) et catalogue des capacités — le registre
        montre ce qui est câblé à cet instant.
      </p>
      <Async state={state}>
        {(s: SystemResponse) => (
          <>
            <div className="cards">
              <Card k="Crates" v={s.counts.crates} />
              <Card k="Capacités" v={s.counts.capabilities} />
              <Card k="Capacités câblées" v={s.counts.capabilities_wired} />
              <Card k="Modèle" v={<code>{s.ollama.model}</code>} />
            </div>

            <h3>Crates par couche</h3>
            <ul className="layers">
              {s.crates.map((c) => (
                <li key={c.name}>
                  <span className="chip">L{c.layer}</span>
                  <code>{c.name}</code>
                  <span className="muted">{c.role}</span>
                </li>
              ))}
            </ul>

            <h3>Capacités du registre</h3>
            <table>
              <thead>
                <tr>
                  <th>Capacité</th>
                  <th>Régime</th>
                  <th>Implémentations actives</th>
                </tr>
              </thead>
              <tbody>
                {s.capabilities.map((c) => (
                  <tr key={c.name} className={c.active.length === 0 ? 'dim' : undefined}>
                    <td>
                      <code>{c.name}</code>
                    </td>
                    <td>
                      <Chip tone={c.regime === 'slot' ? 'warn' : undefined}>{c.regime}</Chip>
                    </td>
                    <td>
                      {c.active.length === 0 ? (
                        <span className="muted">—</span>
                      ) : (
                        c.active.map((a) => <Chip key={a} tone="ok">{a}</Chip>)
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </>
        )}
      </Async>
    </section>
  );
}

/* --- Plugins --------------------------------------------------------------- */

function Plugins() {
  const state = useAsync(getPlugins);
  const shown = state.data ?? FIXTURE_PLUGINS;
  return (
    <section className="panel">
      <h2>Plugin Explorer</h2>
      <p className="hint">Capacités câblées (quarantaine déjà appliquée).</p>
      {state.loading && <Banner kind="loading" text="" />}
      {state.error !== null && <Banner kind="offline" text={`${state.error} — repli fixtures`} />}
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
                <Chip tone={p.active ? 'ok' : 'ko'}>{p.active ? 'oui' : 'non'}</Chip>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

/* --- Modèles --------------------------------------------------------------- */

function Models() {
  const state = useAsync(getModels);
  return (
    <section className="panel">
      <h2>Modèles</h2>
      <p className="hint">Catalogue local Ollama (`/api/tags`) et record de sélection.</p>
      <Async state={state}>
        {(m: ModelsResponse) => (
          <>
            <h3>Installés localement</h3>
            {m.installed.length === 0 ? (
              <Banner kind="offline" text="aucun modèle installé — `ollama pull qwen2.5-coder:7b`" />
            ) : (
              <table>
                <thead>
                  <tr>
                    <th>Modèle</th>
                    <th>Provider</th>
                    <th>Contexte</th>
                  </tr>
                </thead>
                <tbody>
                  {m.installed.map((r) => (
                    <tr key={r.id}>
                      <td>
                        <code>{r.id}</code>
                      </td>
                      <td>{r.provider}</td>
                      <td>{r.context_length ?? '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
            <h3>Record de sélection (coût nul)</h3>
            <div className="cards">
              <Card k="Modèle" v={<code>{m.record.id}</code>} />
              <Card k="Tier" v={<Chip>{m.record.tier}</Chip>} />
              <Card k="Contexte" v={m.record.context_length} />
              <Card k="Coût / 1M (in/out)" v={`${m.record.cost_input_per_1m} / ${m.record.cost_output_per_1m}`} />
              <Card k="Vitesse" v={`${m.record.tokens_per_second} tok/s`} />
              <Card k="TTFT" v={`${m.record.ttft_ms} ms`} />
            </div>
            <h3>Axes de qualité</h3>
            <ul className="axes">
              {Object.entries(m.record.axes).map(([k, v]) => (
                <li key={k}>
                  <span className="muted">{k}</span>
                  <Bar value={v as number} max={100} />
                  <span>{v as number}</span>
                </li>
              ))}
            </ul>
          </>
        )}
      </Async>
    </section>
  );
}

/* --- Routeur --------------------------------------------------------------- */

function RouterPanel({ lastRun }: { lastRun: RunResponse | null }) {
  return (
    <section className="panel">
      <h2>Routeur</h2>
      <p className="hint">
        Dernière décision : classifieur déterministe puis sélecteur Pareto
        (+ préférence self-hosted, marge 0.05).
      </p>
      {lastRun === null ? (
        <Banner kind="offline" text="aucun run cette session — lancez-en un dans Cockpit" />
      ) : (
        <>
          <div className="cards">
            <Card k="Run" v={<code>{lastRun.run_id}</code>} />
            <Card k="Modèle" v={<code>{lastRun.decision.model_id}</code>} />
            <Card
              k="Score"
              v={
                <>
                  {lastRun.decision.score.toFixed(3)}
                  <Bar value={lastRun.decision.score} />
                </>
              }
            />
          </div>
          <p className="hint">
            Raison : <code>{lastRun.decision.reason}</code>
          </p>
        </>
      )}
    </section>
  );
}

/* --- Classifieur ----------------------------------------------------------- */

function Classify() {
  const [text, setText] = useState('Résous cette équation du second degré.');
  const [busy, setBusy] = useState(false);
  const [res, setRes] = useState<ClassifyResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function go() {
    if (busy || text.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      setRes(await postClassify(text));
    } catch (e) {
      setRes(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel">
      <h2>Classifieur de tâche</h2>
      <p className="hint">
        Heuristique déterministe (zéro LLM) : le type de tâche pilote les poids du scoring.
      </p>
      <div className="row">
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          aria-label="Texte à classifier"
          onKeyDown={(e) => {
            if (e.key === 'Enter') void go();
          }}
        />
        <button className="primary" onClick={() => void go()} disabled={busy}>
          Classifier
        </button>
      </div>
      {error !== null && <Banner kind="error" text={error} />}
      {res !== null && (
        <>
          <div className="cards">
            <Card k="Type" v={<code>{res.task_code}</code>} />
            <Card k="Tâche" v={`${res.task}`} />
          </div>
          <h3>Poids du profil</h3>
          <ul className="axes">
            {Object.entries(res.weights).map(([k, v]) => (
              <li key={k}>
                <span className="muted">{k}</span>
                <Bar value={v as number} />
                <span>{(v as number).toFixed(2)}</span>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}

/* --- Traces ---------------------------------------------------------------- */

function Traces({
  runId,
  setRunId,
  autoKey,
}: {
  runId: string;
  setRunId: (v: string) => void;
  autoKey: number;
}) {
  const [trace, setTrace] = useState<TraceResponse | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function load(id?: string) {
    const target = (id ?? runId).trim();
    if (target.length === 0) return;
    setBusy(true);
    setError(null);
    try {
      setTrace(await getTrace(target));
    } catch (e) {
      setTrace(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  // Ouverture depuis le panneau Événements : rejeu automatique.
  useEffect(() => {
    if (autoKey > 0 && runId.trim().length > 0) void load(runId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoKey]);

  return (
    <section className="panel">
      <h2>Traces / Replay</h2>
      <p className="hint">Rejoue un journal sans ré-exécuter les effets (preuve de traçabilité).</p>
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
                <Chip tone={s.status === 'completed' ? 'ok' : 'ko'}>{s.status}</Chip>
              </li>
            ))}
          </ol>
          {trace.answer && (
            <div className="answer">
              <h3>Réponse du modèle (rejouée)</h3>
              {trace.answer}
            </div>
          )}
        </>
      )}
    </section>
  );
}

/* --- Événements ------------------------------------------------------------ */

function Events({ onOpen }: { onOpen: (runId: string) => void }) {
  const state = useAsync(getRuns);
  return (
    <section className="panel">
      <h2>Journal des runs</h2>
      <p className="hint">Runs écrits dans le journal durable (du plus récent au plus ancien).</p>
      <Async state={state}>
        {(r) =>
          r.runs.length === 0 ? (
            <Banner kind="offline" text="aucun run dans le journal" />
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Run</th>
                  <th>Issue</th>
                  <th>Étapes</th>
                  <th>Événements</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {r.runs.map((e: RunEntry) => (
                  <tr key={e.run_id}>
                    <td>
                      <code>{e.run_id}</code>
                    </td>
                    <td>
                      <Chip tone={e.outcome === 'completed' ? 'ok' : 'warn'}>{e.outcome}</Chip>
                    </td>
                    <td>
                      {e.steps_completed}/{e.steps_completed + e.steps_failed}
                    </td>
                    <td>{e.events}</td>
                    <td>
                      <button className="link" onClick={() => onOpen(e.run_id)}>
                        voir
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        }
      </Async>
    </section>
  );
}

/* --- Sandbox --------------------------------------------------------------- */

function Sandbox() {
  const state = useAsync(getPolicy);
  const [cmd, setCmd] = useState('echo hi; rm -rf /');
  const [busy, setBusy] = useState(false);
  const [res, setRes] = useState<EvaluateResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function go() {
    if (busy || cmd.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      setRes(await postEvaluate(cmd));
    } catch (e) {
      setRes(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  function tone(d: string): 'ok' | 'warn' | 'ko' {
    return d === 'allow' ? 'ok' : d === 'prompt' ? 'warn' : 'ko';
  }

  return (
    <section className="panel">
      <h2>Sandbox — politique d'exécution</h2>
      <p className="hint">
        Politique de référence (deny-by-default). L'évaluation est déterministe et
        **n'exécute rien**.
      </p>
      <Async state={state}>
        {(p: PolicyResponse) => (
          <>
            <Banner kind="offline" text={p.note} />
            <div className="cards">
              <Card k="Strict" v={p.strict ? 'oui' : 'non'} />
              <Card k="Défaut" v={<Chip tone={tone(p.default)}>{p.default}</Chip>} />
              <Card k="Règles" v={p.rules.length} />
            </div>
            <table>
              <thead>
                <tr>
                  <th>Préfixe</th>
                  <th>Décision</th>
                  <th>Motif</th>
                </tr>
              </thead>
              <tbody>
                {p.rules.map((r) => (
                  <tr key={r.prefix}>
                    <td>
                      <code>{r.prefix}</code>
                    </td>
                    <td>
                      <Chip tone={tone(r.decision)}>{r.decision}</Chip>
                    </td>
                    <td className="muted">{r.reason ?? '—'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </>
        )}
      </Async>

      <h3>Tester une commande</h3>
      <div className="row">
        <input
          type="text"
          value={cmd}
          onChange={(e) => setCmd(e.target.value)}
          aria-label="Commande à évaluer"
          onKeyDown={(e) => {
            if (e.key === 'Enter') void go();
          }}
        />
        <button className="primary" onClick={() => void go()} disabled={busy}>
          Évaluer
        </button>
      </div>
      {error !== null && <Banner kind="error" text={error} />}
      {res !== null && (
        <div className="cards">
          <Card k="Décision (règle)" v={<Chip tone={tone(res.decision)}>{res.decision}</Chip>} />
          <Card k="Effective" v={<Chip tone={tone(res.effective)}>{res.effective}</Chip>} />
          <Card k="Composition" v={res.compound ? 'oui (métacaractères)' : 'non'} />
          <Card k="Substitution" v={res.substitution ? 'oui ($(...)/backticks)' : 'non'} />
        </div>
      )}
      {res?.reason && (
        <p className="hint">
          Motif : <code>{res.reason}</code>
        </p>
      )}
    </section>
  );
}

/* --- Contexte -------------------------------------------------------------- */

function Context() {
  const state = useAsync(getContext);
  return (
    <section className="panel">
      <h2>Context Inspector</h2>
      <p className="hint">
        Économie de contexte : estimation déterministe, budget, tool search BM25 + gate
        rarest-token, progressive disclosure des skills.
      </p>
      <Async state={state}>
        {(c: ContextResponse) => (
          <>
            <div className="cards">
              <Card k="Estimateur" v={<code>{c.estimator}</code>} />
              <Card k="Budget" v={<code>{c.max_tokens}</code>} />
              <Card k="Contexte exigé" v={<code>{c.required_context}</code>} />
            </div>
            <h3>Échantillons d'estimation</h3>
            <table>
              <thead>
                <tr>
                  <th>Caractères</th>
                  <th>Tokens estimés</th>
                </tr>
              </thead>
              <tbody>
                {c.sample.map((s) => (
                  <tr key={s.chars}>
                    <td>{s.chars}</td>
                    <td>{s.tokens}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </>
        )}
      </Async>
    </section>
  );
}

/* --- Application ----------------------------------------------------------- */

export default function App() {
  const [panel, setPanel] = useState<Panel>('cockpit');
  const [lastRun, setLastRun] = useState<RunResponse | null>(null);
  const [online, setOnline] = useState<boolean | null>(null);
  const [traceRunId, setTraceRunId] = useState('');
  const [traceAuto, setTraceAuto] = useState(0);

  useEffect(() => {
    getHealth()
      .then(() => setOnline(true))
      .catch(() => setOnline(false));
  }, []);

  // Un run alimente le Routeur et pré-remplit la cible de rejeu.
  function onRun(r: RunResponse) {
    setLastRun(r);
    setTraceRunId(r.run_id);
  }

  // Depuis le panneau Événements : cibler un run et rejouer immédiatement.
  function openTrace(runId: string) {
    setTraceRunId(runId);
    setTraceAuto((k) => k + 1);
    setPanel('traces');
  }

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
      {panel === 'cockpit' && <Cockpit onRun={onRun} />}
      {panel === 'system' && <SystemPanel />}
      {panel === 'plugins' && <Plugins />}
      {panel === 'models' && <Models />}
      {panel === 'router' && <RouterPanel lastRun={lastRun} />}
      {panel === 'classify' && <Classify />}
      {panel === 'traces' && (
        <Traces runId={traceRunId} setRunId={setTraceRunId} autoKey={traceAuto} />
      )}
      {panel === 'events' && <Events onOpen={openTrace} />}
      {panel === 'sandbox' && <Sandbox />}
      {panel === 'context' && <Context />}
    </div>
  );
}
