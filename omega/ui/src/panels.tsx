import { useEffect, useState } from 'react';
import {
  getConfig,
  getDissections,
  getHarnesses,
  getContext,
  getModels,
  getOverview,
  getPlugins,
  getPolicy,
  getRegistry,
  getRoutes,
  getRea,
  getReaTools,
  getRuns,
  getRunEvents,
  getSystem,
  getTrace,
  postClassify,
  postEvaluate,
  postRun,
} from './api';
import { Async, Banner, Bar, Card, Chip, decisionTone, useAsync } from './components';
import type {
  ClassifyResponse,
  ConfigResponse,
  ContextResponse,
  DissectionsResponse,
  DurableEvent,
  EvaluateResponse,
  Harness,
  HarnessCatalog,
  Message,
  ModelsResponse,
  OverviewResponse,
  PluginInfo,
  PolicyResponse,
  ReaResponse,
  ReaTool,
  Principle,
  PrincipleStatus,
  RegistryResponse,
  RouteInfo,
  RunEntry,
  RunResponse,
  SystemResponse,
  TraceResponse,
} from './protocol';

const FIXTURE_PLUGINS: PluginInfo[] = [
  { schema_version: 1, id: 'ollama', capability: 'llm-provider', version: '1', active: true },
  { schema_version: 1, id: 'default', capability: 'context-engine', version: '1', active: true },
  { schema_version: 1, id: 'file', capability: 'event-store', version: '1', active: true },
];

/* =========================================================================
 * Vue d'ensemble (dashboard) — tout en une requête, rafraîchi en continu
 * ========================================================================= */

export function Dashboard({
  onRun,
  onIsolateCapability,
  onOpenRun,
}: {
  onRun: (r: RunResponse) => void;
  onIsolateCapability: (cap: string) => void;
  onOpenRun: (runId: string) => void;
}) {
  const [ov, setOv] = useState<OverviewResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [auto, setAuto] = useState(true);
  const [tick, setTick] = useState(0);

  // Rafraîchissement périodique (observation live du système).
  useEffect(() => {
    if (!auto) return;
    const id = window.setInterval(() => setTick((t) => t + 1), 3000);
    return () => window.clearInterval(id);
  }, [auto]);

  useEffect(() => {
    let cancelled = false;
    getOverview()
      .then((d) => {
        if (!cancelled) {
          setOv(d);
          setError(null);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [tick]);

  if (error !== null && ov === null) return <Banner kind="error" text={error} />;
  if (ov === null) return <Banner kind="loading" text="Chargement de la vue d'ensemble…" />;

  const wired = ov.capabilities.filter((c) => c.active.length > 0);

  return (
    <section className="panel">
      <div className="panel-head">
        <h2>Vue d'ensemble</h2>
        <label className="toggle">
          <input type="checkbox" checked={auto} onChange={(e) => setAuto(e.target.checked)} />
          auto 3 s
        </label>
      </div>
      <p className="hint">
        Tout le système au même endroit : état, configuration, registre, runs récents.
        Cliquez une capacité pour l'isoler, un run pour le tracer.
      </p>

      <div className="cards">
        <Card k="État" v={<Chip tone={ov.status === 'ok' ? 'ok' : 'ko'}>{ov.status}</Chip>} />
        <Card k="Crates" v={ov.crate_count} />
        <Card k="Capacités" v={`${ov.counts.capabilities_wired}/${ov.counts.capabilities}`} />
        <Card k="Port" v={<code>{ov.config.port}</code>} />
        <Card k="UI servie" v={ov.config.ui_served ? 'oui' : 'non'} />
        <Card k="Modèle" v={<code>{ov.config.ollama_model}</code>} />
        <Card
          k="Harnais reverse-eng."
          v={
            <>
              {ov.harness.adopted}/{ov.harness.principles} adoptés
              <div className="bar">
                <i
                  style={{
                    width: `${
                      ov.harness.principles === 0
                        ? 0
                        : (ov.harness.adopted / ov.harness.principles) * 100
                    }%`,
                  }}
                />
              </div>
            </>
          }
        />
      </div>

      <h3>Capacités câblées</h3>
      <div className="grid-caps">
        {wired.length === 0 ? (
          <Banner kind="offline" text="aucune capacité câblée" />
        ) : (
          wired.map((c) => (
            <button key={c.name} className="cap" onClick={() => onIsolateCapability(c.name)}>
              <code>{c.name}</code>
              <Chip tone={c.regime === 'slot' ? 'warn' : undefined}>{c.regime}</Chip>
              <span className="muted">{c.active.join(', ')}</span>
            </button>
          ))
        )}
      </div>

      <h3>Runs récents</h3>
      {ov.recent_runs.length === 0 ? (
        <Banner kind="offline" text="aucun run — lancez-en un depuis Cockpit" />
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
            {ov.recent_runs.map((r) => (
              <tr key={r.run_id}>
                <td>
                  <code>{r.run_id}</code>
                </td>
                <td>
                  <Chip tone={r.outcome === 'completed' ? 'ok' : 'warn'}>{r.outcome}</Chip>
                </td>
                <td>
                  {r.steps_completed}/{r.steps_completed + r.steps_failed}
                </td>
                <td>{r.events}</td>
                <td>
                  <button className="link" onClick={() => onOpenRun(r.run_id)}>
                    tracer
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <h3>Record de sélection</h3>
      <div className="cards">
        <Card k="Modèle" v={<code>{ov.model_record.id}</code>} />
        <Card k="Tier" v={<Chip>{ov.model_record.tier}</Chip>} />
        <Card k="Coût / 1M" v={`${ov.model_record.cost_input_per_1m} / ${ov.model_record.cost_output_per_1m}`} />
        <Card k="Budget contexte" v={ov.config.max_tokens} />
      </div>
      <RunLauncher onRun={onRun} />
    </section>
  );
}

/** Lanceur de run minimal, réutilisé dans le dashboard. */
function RunLauncher({ onRun }: { onRun: (r: RunResponse) => void }) {
  const [text, setText] = useState('Explique Omega en une phrase.');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function go() {
    if (busy || text.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      onRun(await postRun([{ role: 'user', content: text } as Message]));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }
  return (
    <>
      <h3>Lancer un run</h3>
      <div className="row">
        <input
          type="text"
          value={text}
          onChange={(e) => setText(e.target.value)}
          aria-label="Message du run rapide"
          onKeyDown={(e) => {
            if (e.key === 'Enter') void go();
          }}
        />
        <button className="primary" onClick={() => void go()} disabled={busy}>
          {busy ? 'Run…' : 'Lancer'}
        </button>
      </div>
      {error !== null && <Banner kind="error" text={error} />}
    </>
  );
}

/* =========================================================================
 * Cockpit
 * ========================================================================= */

export function Cockpit({ onRun }: { onRun: (r: RunResponse) => void }) {
  const [input, setInput] = useState('Résume Omega en trois points.');
  const [busy, setBusy] = useState(false);
  const [run, setRun] = useState<RunResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function launch() {
    if (busy || input.trim().length === 0) return;
    setBusy(true);
    setError(null);
    try {
      const r = await postRun([{ role: 'user', content: input } as Message]);
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

/* =========================================================================
 * Système (architecture)
 * ========================================================================= */

export function SystemPanel() {
  const state = useAsync(getSystem);
  return (
    <section className="panel">
      <h2>Architecture du système</h2>
      <p className="hint">Workspace Rust (crates par couche) et catalogue des capacités.</p>
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
                        c.active.map((a) => (
                          <Chip key={a} tone="ok">
                            {a}
                          </Chip>
                        ))
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

/* =========================================================================
 * Registre (isolation par plugin)
 * ========================================================================= */

export function RegistryPanel({ isolate }: { isolate: string | null }) {
  const state = useAsync(getRegistry);
  const [filter, setFilter] = useState(isolate ?? '');

  useEffect(() => {
    if (isolate !== null) setFilter(isolate);
  }, [isolate]);

  return (
    <section className="panel">
      <h2>Registre — isolation</h2>
      <p className="hint">
        Chaque entrée : identifiant, version, priorité, voie, quarantaine. Filtrez une
        capacité pour l'isoler.
      </p>
      <div className="row">
        <input
          type="text"
          value={filter}
          placeholder="filtrer par capacité (ex. llm-provider)"
          onChange={(e) => setFilter(e.target.value)}
          aria-label="Filtre capacité"
        />
      </div>
      <Async state={state}>
        {(r: RegistryResponse) => {
          const caps = r.capabilities.filter(
            (c) => filter.trim().length === 0 || c.name.includes(filter.trim()),
          );
          const withEntries = caps.filter((c) => c.entries.length > 0);
          return (
            <>
              <div className="cards">
                <Card k="Capacités" v={r.capabilities.length} />
                <Card k="Affichées" v={caps.length} />
                <Card k="Avec entrées" v={withEntries.length} />
              </div>
              <table>
                <thead>
                  <tr>
                    <th>Capacité</th>
                    <th>Régime</th>
                    <th>Entrée</th>
                    <th>Version</th>
                    <th>Prio</th>
                    <th>Voie</th>
                    <th>État</th>
                  </tr>
                </thead>
                <tbody>
                  {caps.flatMap((c) =>
                    c.entries.length === 0
                      ? [
                          <tr key={`${c.name}-none`} className="dim">
                            <td>
                              <code>{c.name}</code>
                            </td>
                            <td>
                              <Chip tone={c.regime === 'slot' ? 'warn' : undefined}>
                                {c.regime}
                              </Chip>
                            </td>
                            <td colSpan={5} className="muted">
                              aucune implémentation
                            </td>
                          </tr>,
                        ]
                      : c.entries.map((e) => (
                          <tr key={`${c.name}/${e.id}`}>
                            <td>
                              <code>{c.name}</code>
                            </td>
                            <td>
                              <Chip tone={c.regime === 'slot' ? 'warn' : undefined}>
                                {c.regime}
                              </Chip>
                            </td>
                            <td>
                              <code>{e.id}</code>
                            </td>
                            <td>{e.version}</td>
                            <td>{e.priority}</td>
                            <td>
                              <Chip>{e.kind}</Chip>
                            </td>
                            <td>
                              <Chip tone={e.quarantined ? 'ko' : 'ok'}>
                                {e.quarantined ? 'quarantaine' : 'actif'}
                              </Chip>
                            </td>
                          </tr>
                        )),
                  )}
                </tbody>
              </table>
            </>
          );
        }}
      </Async>
    </section>
  );
}

/* =========================================================================
 * Plugins / Modèles / Routeur / Classifieur
 * ========================================================================= */

export function Plugins() {
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

export function Models() {
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
              <Banner kind="offline" text="aucun modèle — `ollama pull qwen2.5-coder:7b`" />
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
              <Card
                k="Coût / 1M (in/out)"
                v={`${m.record.cost_input_per_1m} / ${m.record.cost_output_per_1m}`}
              />
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

export function RouterPanel({ lastRun }: { lastRun: RunResponse | null }) {
  return (
    <section className="panel">
      <h2>Routeur</h2>
      <p className="hint">
        Dernière décision : classifieur déterministe puis sélecteur Pareto (+ préférence
        self-hosted, marge 0.05).
      </p>
      {lastRun === null ? (
        <Banner kind="offline" text="aucun run cette session — lancez-en un depuis Cockpit" />
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

export function Classify() {
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

/* =========================================================================
 * Runs + traçabilité brute (isolation par run)
 * ========================================================================= */

function EventTimeline({ events }: { events: DurableEvent[] }) {
  return (
    <ol className="timeline">
      {events.map((e) => (
        <li key={e.seq} className={e.name.startsWith('run.') ? 'mark' : undefined}>
          <span className="seq">#{e.seq}</span>
          <code>{e.name}</code>
          {e.operation_id && <Chip>{e.operation_id}</Chip>}
          <span className="muted mono">{JSON.stringify(e.payload)}</span>
        </li>
      ))}
    </ol>
  );
}

export function RunsPanel({
  isolate,
  onOpenRun,
}: {
  isolate: string | null;
  onOpenRun: (runId: string) => void;
}) {
  const state = useAsync(getRuns);
  const [selected, setSelected] = useState<string | null>(isolate);

  useEffect(() => {
    if (isolate !== null) setSelected(isolate);
  }, [isolate]);

  const events = useAsync(
    () => (selected ? getRunEvents(selected) : Promise.resolve(null)),
    [selected],
  );

  return (
    <section className="panel">
      <h2>Runs & traçabilité</h2>
      <p className="hint">
        Journal durable : chaque run, ses événements bruts (chaîne complète, isolable).
      </p>
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
                  <tr key={e.run_id} className={selected === e.run_id ? 'selected' : undefined}>
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
                      <button className="link" onClick={() => setSelected(e.run_id)}>
                        isoler
                      </button>
                      <button className="link" onClick={() => onOpenRun(e.run_id)}>
                        rejouer
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )
        }
      </Async>

      {selected !== null && (
        <>
          <h3>
            Événements bruts — <code>{selected}</code>
          </h3>
          <Async state={events}>
            {(ev) => (ev === null ? null : <EventTimeline events={ev.events} />)}
          </Async>
        </>
      )}
    </section>
  );
}

/* =========================================================================
 * Traces (rejeu)
 * ========================================================================= */

export function Traces({
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

  useEffect(() => {
    if (autoKey > 0 && runId.trim().length > 0) void load(runId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoKey]);

  return (
    <section className="panel">
      <h2>Traces / Replay</h2>
      <p className="hint">Rejoue un journal sans ré-exécuter les effets.</p>
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

/* =========================================================================
 * Sandbox
 * ========================================================================= */

export function Sandbox() {
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

  return (
    <section className="panel">
      <h2>Sandbox — politique d'exécution</h2>
      <p className="hint">
        Politique de référence (deny-by-default). L'évaluation est déterministe et n'exécute rien.
      </p>
      <Async state={state}>
        {(p: PolicyResponse) => (
          <>
            <Banner kind="offline" text={p.note} />
            <div className="cards">
              <Card k="Strict" v={p.strict ? 'oui' : 'non'} />
              <Card k="Défaut" v={<Chip tone={decisionTone(p.default)}>{p.default}</Chip>} />
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
                      <Chip tone={decisionTone(r.decision)}>{r.decision}</Chip>
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
          <Card k="Décision (règle)" v={<Chip tone={decisionTone(res.decision)}>{res.decision}</Chip>} />
          <Card k="Effective" v={<Chip tone={decisionTone(res.effective)}>{res.effective}</Chip>} />
          <Card k="Composition" v={res.compound ? 'oui (métacaractères)' : 'non'} />
          <Card k="Substitution" v={res.substitution ? "oui ($(...)/backticks)" : 'non'} />
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

/* =========================================================================
 * Contexte
 * ========================================================================= */

export function ContextPanel() {
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

/* =========================================================================
 * Configuration & surface d'API
 * ========================================================================= */

export function ConfigPanel() {
  const cfg = useAsync(getConfig);
  const routes = useAsync(getRoutes);
  return (
    <section className="panel">
      <h2>Configuration & API</h2>
      <p className="hint">Configuration effective (sans secret) et surface d'API auto-descriptive.</p>
      <Async state={cfg}>
        {(c: ConfigResponse) => (
          <div className="cards">
            <Card k="Port" v={<code>{c.port}</code>} />
            <Card k="Ollama" v={<code>{c.ollama_url}</code>} />
            <Card k="Modèle" v={<code>{c.ollama_model}</code>} />
            <Card k="Journal" v={<code>{c.event_dir}</code>} />
            <Card k="Budget contexte" v={c.max_tokens} />
            <Card k="UI servie" v={c.ui_served ? 'oui' : 'non'} />
          </div>
        )}
      </Async>
      <h3>Endpoints</h3>
      <Async state={routes}>
        {(r) => (
          <table>
            <thead>
              <tr>
                <th>Méthode</th>
                <th>Chemin</th>
                <th>Description</th>
              </tr>
            </thead>
            <tbody>
              {r.routes.map((x: RouteInfo) => (
                <tr key={`${x.method}${x.path}`}>
                  <td>
                    <Chip>{x.method}</Chip>
                  </td>
                  <td>
                    <code>{x.path}</code>
                  </td>
                  <td className="muted">{x.description}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </Async>
    </section>
  );
}

/* =========================================================================
 * Harnais (reverse-engineering) — observation des grands harnais
 * ========================================================================= */

export function HarnessesPanel({ isolate }: { isolate: string | null }) {
  const state = useAsync(getHarnesses);
  const [filter, setFilter] = useState(isolate ?? '');
  const [status, setStatus] = useState<'all' | PrincipleStatus>('all');

  useEffect(() => {
    if (isolate !== null) setFilter(isolate);
  }, [isolate]);

  return (
    <section className="panel">
      <h2>Harnais — principes reverse-engineerés</h2>
      <p className="hint">
        Les grands harnais, leurs principes transférables et ce qu'Omega en fait
        réellement : <Chip tone="ok">adopted</Chip> implémenté ·
        <Chip tone="warn">planned</Chip> repris à faire · <Chip tone="ko">rejected</Chip> écarté.
      </p>
      <div className="row">
        <input
          type="text"
          value={filter}
          placeholder="filtrer (harnais, principe, texte)"
          onChange={(e) => setFilter(e.target.value)}
          aria-label="Filtre harnais"
        />
      </div>
      <div className="tabs sub">
        {(['all', 'adopted', 'planned', 'rejected'] as const).map((s) => (
          <button key={s} onClick={() => setStatus(s)} aria-pressed={status === s}>
            {s === 'all' ? 'tous' : s}
          </button>
        ))}
      </div>
      <Async state={state}>
        {(c: HarnessCatalog) => {
          const q = filter.trim().toLowerCase();
          const match = (h: Harness, p?: Principle) =>
            q.length === 0 ||
            h.id.includes(q) ||
            h.name.toLowerCase().includes(q) ||
            (p !== undefined &&
              (p.name.toLowerCase().includes(q) ||
                p.what.toLowerCase().includes(q) ||
                (p.omega ?? '').toLowerCase().includes(q)));
          const harnesses = c.harnesses
            .map((h) => ({ h, ps: h.principles.filter((p) => match(h, p) && (status === 'all' || p.status === status)) }))
            .filter(({ h, ps }) => ps.length > 0 || (q.length > 0 && match(h)));
          const total = c.harnesses.reduce((n, h) => n + h.principles.length, 0);
          const adopted = c.harnesses.reduce(
            (n, h) => n + h.principles.filter((p) => p.status === 'adopted').length,
            0,
          );
          return (
            <>
              <div className="cards">
                <Card k="Harnais observés" v={c.harnesses.length} />
                <Card k="Principes" v={total} />
                <Card k="Adoptés" v={adopted} />
                <Card k="Restants" v={total - adopted} />
              </div>
              {harnesses.map(({ h, ps }) => (
                <div key={h.id} className="harness">
                  <h3>
                    {h.name} <span className="muted">{h.editor}</span>{' '}
                    <Chip>{h.oss ? 'OSS' : 'propriétaire'}</Chip> <Chip>{h.language}</Chip>
                  </h3>
                  <p className="muted">{h.architecture}</p>
                  <table>
                    <thead>
                      <tr>
                        <th>Principe</th>
                        <th>Statut</th>
                        <th>Dans Omega</th>
                        <th>Rust</th>
                        <th>Source</th>
                      </tr>
                    </thead>
                    <tbody>
                      {ps.map((p) => (
                        <tr key={p.id}>
                          <td>
                            <strong>{p.name}</strong>
                            <div className="muted">{p.what}</div>
                          </td>
                          <td>
                            <Chip
                              tone={
                                p.status === 'adopted' ? 'ok' : p.status === 'planned' ? 'warn' : 'ko'
                              }
                            >
                              {p.status}
                            </Chip>
                          </td>
                          <td>
                            {p.omega ? <code>{p.omega}</code> : <span className="muted">—</span>}
                          </td>
                          <td>
                            <Chip>{p.rust_cost}</Chip>
                          </td>
                          <td className="muted">{p.refs.join(', ')}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ))}
              {harnesses.length === 0 && <Banner kind="offline" text="aucun principe ne correspond" />}
            </>
          );
        }}
      </Async>
    </section>
  );
}

/* =========================================================================
 * Dissections (reverse-engineering exhaustif, fonctionnalité par fonctionnalité)
 * ========================================================================= */

export function DissectionsPanel({ isolate }: { isolate: string | null }) {
  const state = useAsync(getDissections);
  const [selected, setSelected] = useState<string | null>(isolate);
  const [domain, setDomain] = useState<string>('');

  useEffect(() => {
    if (isolate !== null) setSelected(isolate);
  }, [isolate]);

  return (
    <section className="panel">
      <h2>Dissections — fonctionnalité par fonctionnalité</h2>
      <p className="hint">
        Chaque application décortiquée : ses fonctionnalités, leur implémentation d'origine
        (<code>how</code>), et la traduction en plugin Omega (
        <code>capacité → port → adaptateur</code>).
      </p>
      <Async state={state}>
        {(d: DissectionsResponse) => {
          const apps = d.apps;
          const current = apps.find((a) => a.id === selected) ?? apps[0] ?? null;
          const domains = current ? Array.from(new Set(current.functionalities.map((f) => f.domain))).sort() : [];
          const shown = current
            ? current.functionalities.filter((f) => domain === '' || f.domain === domain)
            : [];
          const byDomain = domains
            .map((dom) => ({ dom, fs: shown.filter((f) => f.domain === dom) }))
            .filter(({ fs }) => fs.length > 0);
          return (
            <>
              <div className="cards">
                <Card k="Apps disséquées" v={d.totals.apps} />
                <Card k="Fonctionnalités" v={d.totals.functionalities} />
                <Card k="Adoptées" v={d.totals.adopted} />
                <Card k="Restantes" v={d.totals.planned + d.totals.rejected} />
              </div>

              <div className="tabs sub">
                {apps.map((a) => (
                  <button key={a.id} onClick={() => setSelected(a.id)} aria-pressed={current?.id === a.id}>
                    {a.name}
                  </button>
                ))}
              </div>

              {current === null ? (
                <Banner kind="offline" text="aucune dissection disponible" />
              ) : (
                <>
                  <h3>
                    {current.name} <span className="muted">{current.editor}</span>{' '}
                    <Chip>{current.license}</Chip> <Chip>{current.language}</Chip>{' '}
                    <Chip>{current.functionalities.length} fonctionnalités</Chip>
                  </h3>
                  <p className="muted">{current.architecture}</p>
                  {current.entrypoints.length > 0 && (
                    <p className="muted">
                      Entrées : {current.entrypoints.map((e) => (
                        <code key={e} style={{ marginRight: 6 }}>
                          {e}
                        </code>
                      ))}
                    </p>
                  )}

                  <div className="tabs sub">
                    <button onClick={() => setDomain('')} aria-pressed={domain === ''}>
                      tous domaines
                    </button>
                    {domains.map((dom) => (
                      <button key={dom} onClick={() => setDomain(dom)} aria-pressed={domain === dom}>
                        {dom}
                      </button>
                    ))}
                  </div>

                  {byDomain.map(({ dom, fs }) => (
                    <div key={dom} className="harness">
                      <h3>{dom}</h3>
                      <table>
                        <thead>
                          <tr>
                            <th>Fonctionnalité</th>
                            <th>Statut</th>
                            <th>capabilité → port → adaptateur</th>
                            <th>Intégration</th>
                            <th>Rust</th>
                            <th>Source</th>
                          </tr>
                        </thead>
                        <tbody>
                          {fs.map((f) => (
                            <tr key={f.id}>
                              <td>
                                <strong>{f.name}</strong>
                                <div className="muted">{f.what}</div>
                                <div className="muted">
                                  <em>Implémentation d'origine :</em> {f.how}
                                </div>
                              </td>
                              <td>
                                <Chip
                                  tone={
                                    f.status === 'adopted'
                                      ? 'ok'
                                      : f.status === 'planned'
                                        ? 'warn'
                                        : 'ko'
                                  }
                                >
                                  {f.status}
                                </Chip>
                              </td>
                              <td className="muted">
                                <code>{f.omega_capability ?? '—'}</code> →{' '}
                                <code>{f.omega_port ?? '—'}</code> →{' '}
                                <code>{f.omega_adapter ?? '—'}</code>
                              </td>
                              <td>
                                <Chip tone={f.integration === 'plugin' ? 'ok' : undefined}>
                                  {f.integration}
                                </Chip>
                              </td>
                              <td>
                                <Chip>{f.rust_cost}</Chip>
                              </td>
                              <td className="muted">{f.refs.join(', ')}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ))}
                </>
              )}
            </>
          );
        }}
      </Async>
    </section>
  );
}

/* =========================================================================
 * REA — pont MCP (reverse-engineering) : instantané embarqué + catalogue live
 * ========================================================================= */

export function ReaPanel() {
  const state = useAsync(getRea);
  const [tools, setTools] = useState<ReaTool[] | null>(null);
  const [toolsError, setToolsError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState('');

  async function loadTools() {
    setLoading(true);
    setToolsError(null);
    try {
      const r = await getReaTools();
      setTools(r.tools);
    } catch (e) {
      setToolsError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  const shown = (tools ?? []).filter(
    (t) =>
      filter === '' ||
      t.name.toLowerCase().includes(filter.toLowerCase()) ||
      t.description.toLowerCase().includes(filter.toLowerCase()),
  );

  return (
    <section className="panel">
      <h2>REA — pont MCP de reverse-engineering</h2>
      <p className="hint">
        <code>morluto/rea</code> branché comme <strong>surface d'outils</strong> (
        <code>Capability::ToolSurface</code>, plugin <code>mcp:rea</code>) : le noyau ne connaît que
        le port MCP, REA fournit les outils. Instantané <em>embarqué</em> (déterministe) ; le
        catalogue live lance <code>npx rea-agents --mcp</code>.
      </p>
      <Async state={state}>
        {(d: ReaResponse) => (
          <>
            <div className="cards">
              <Card k="REA" v={d.snapshot.rea_version} />
              <Card k="Outils MCP" v={d.snapshot.catalog.mcp_tools} />
              <Card k="Commandes CLI" v={d.snapshot.catalog.cli_commands} />
              <Card k="Prompts MCP" v={d.snapshot.catalog.mcp_prompts} />
            </div>

            <h3>Pont</h3>
            <table>
              <tbody>
                <tr>
                  <th>Plugin</th>
                  <td>
                    <code>{d.bridge.plugin_id}</code>
                  </td>
                </tr>
                <tr>
                  <th>Lancement</th>
                  <td>
                    <code>
                      {d.bridge.spec.command} {d.bridge.spec.args.join(' ')}
                    </code>
                  </td>
                </tr>
                <tr>
                  <th>Préfixe d'outils</th>
                  <td>
                    <code>{d.bridge.prefix}</code>
                  </td>
                </tr>
                <tr>
                  <th>Révision MCP</th>
                  <td>
                    <code>{d.bridge.protocol_version}</code>
                  </td>
                </tr>
                <tr>
                  <th>Hôte</th>
                  <td className="muted">
                    {d.snapshot.host.platform} · node {d.snapshot.host.node} · moteurs :{' '}
                    {d.snapshot.host.native_engines.join(', ') || '—'}
                  </td>
                </tr>
              </tbody>
            </table>

            <h3>Providers</h3>
            <table>
              <thead>
                <tr>
                  <th>Provider</th>
                  <th>Nom</th>
                  <th>Opérations dispo.</th>
                  <th>Indispo.</th>
                </tr>
              </thead>
              <tbody>
                {d.snapshot.providers.map((p) => (
                  <tr key={p.id}>
                    <td>
                      <code>{p.id}</code>
                    </td>
                    <td>{p.name}</td>
                    <td>
                      <Chip tone={p.available_ops > 0 ? 'ok' : undefined}>{p.available_ops}</Chip>
                    </td>
                    <td>
                      <Chip tone={p.unavailable_ops > 0 ? 'warn' : undefined}>
                        {p.unavailable_ops}
                      </Chip>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            <p className="muted">{d.snapshot.note}</p>
            <p className="muted">
              Digests : outils <code>{String(d.snapshot.digests.tools_sha256 ?? '').slice(0, 16)}…</code>{' '}
              · prompts <code>{String(d.snapshot.digests.prompts_sha256 ?? '').slice(0, 16)}…</code>{' '}
              (empreinte du catalogue — sert au versioning du pont).
            </p>

            <h3>Catalogue live ({tools ? tools.length : '—'} outils)</h3>
            <p>
              <button onClick={loadTools} disabled={loading}>
                {loading ? 'Interrogation de REA…' : 'Lister les outils (live)'}
              </button>{' '}
              <input
                placeholder="filtrer…"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                style={{ marginLeft: 8 }}
              />
            </p>
            {toolsError && <Banner kind="error" text={toolsError} />}
            {tools && (
              <table>
                <thead>
                  <tr>
                    <th>Outil</th>
                    <th>Description</th>
                  </tr>
                </thead>
                <tbody>
                  {shown.map((t) => (
                    <tr key={t.name}>
                      <td>
                        <code>{t.prefixed}</code>
                      </td>
                      <td className="muted">{t.description}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </>
        )}
      </Async>
    </section>
  );
}
