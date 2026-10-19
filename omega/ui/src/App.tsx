import { useEffect, useState } from 'react';
import { getHealth } from './api';
import type { RunResponse } from './protocol';
import {
  Classify,
  Cockpit,
  ConfigPanel,
  ContextPanel,
  Dashboard,
  DissectionsPanel,
  HarnessesPanel,
  Models,
  Plugins,
  ReaPanel,
  RegistryPanel,
  RouterPanel,
  RunsPanel,
  Sandbox,
  SystemPanel,
  Traces,
} from './panels';

/**
 * Omega Cockpit — shell de l'observatoire.
 *
 * 13 vues, une origine (`omega-server`) : tout le système est observable et
 * traçable au même endroit, et chaque brique est isolable (capacité, run).
 */

type Panel =
  | 'dashboard'
  | 'cockpit'
  | 'system'
  | 'harnesses'
  | 'dissections'
  | 'rea'
  | 'registry'
  | 'plugins'
  | 'models'
  | 'router'
  | 'classify'
  | 'runs'
  | 'traces'
  | 'sandbox'
  | 'context'
  | 'config';

const PANELS: { id: Panel; label: string }[] = [
  { id: 'dashboard', label: "Vue d'ensemble" },
  { id: 'cockpit', label: 'Cockpit' },
  { id: 'system', label: 'Système' },
  { id: 'harnesses', label: 'Harnais' },
  { id: 'dissections', label: 'Dissections' },
  { id: 'rea', label: 'REA' },
  { id: 'registry', label: 'Registre' },
  { id: 'plugins', label: 'Plugins' },
  { id: 'models', label: 'Modèles' },
  { id: 'router', label: 'Routeur' },
  { id: 'classify', label: 'Classifieur' },
  { id: 'runs', label: 'Runs' },
  { id: 'traces', label: 'Traces' },
  { id: 'sandbox', label: 'Sandbox' },
  { id: 'context', label: 'Contexte' },
  { id: 'config', label: 'Config / API' },
];

/** Cible d'isolation : la valeur + une clé de remontage (isolation stricte). */
type Isolation<T> = { value: T; key: number } | null;

export default function App() {
  const [panel, setPanel] = useState<Panel>('dashboard');
  const [lastRun, setLastRun] = useState<RunResponse | null>(null);
  const [online, setOnline] = useState<boolean | null>(null);
  const [traceRunId, setTraceRunId] = useState('');
  const [traceAuto, setTraceAuto] = useState(0);
  const [capIsolation, setCapIsolation] = useState<Isolation<string>>(null);
  const [runIsolation, setRunIsolation] = useState<Isolation<string>>(null);

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

  // Isoler une capacité : on ouvre le registre, filtré sur elle.
  function isolateCapability(name: string) {
    setCapIsolation({ value: name, key: Date.now() });
    setPanel('registry');
  }

  // Isoler un run : on ouvre le journal d'événements bruts sur lui.
  function isolateRun(runId: string) {
    setRunIsolation({ value: runId, key: Date.now() });
    setPanel('runs');
  }

  // Rejouer un run : on ouvre les traces et on déclenche le rejeu.
  function replayRun(runId: string) {
    setTraceRunId(runId);
    setTraceAuto((k) => k + 1);
    setPanel('traces');
  }

  const statusText = online === null ? 'Connexion…' : online ? 'API en ligne' : 'API injoignable';

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

      {panel === 'dashboard' && (
        <Dashboard onRun={onRun} onIsolateCapability={isolateCapability} onOpenRun={isolateRun} />
      )}
      {panel === 'cockpit' && <Cockpit onRun={onRun} />}
      {panel === 'system' && <SystemPanel />}
      {panel === 'dissections' && (
        <DissectionsPanel key={`d-${capIsolation?.key ?? 0}`} isolate={capIsolation?.value ?? null} />
      )}
      {panel === 'harnesses' && (
        <HarnessesPanel key={`h-${capIsolation?.key ?? 0}`} isolate={capIsolation?.value ?? null} />
      )}
      {panel === 'rea' && <ReaPanel />}
      {panel === 'registry' && (
        <RegistryPanel
          key={capIsolation?.key ?? 0}
          isolate={capIsolation?.value ?? null}
        />
      )}
      {panel === 'plugins' && <Plugins />}
      {panel === 'models' && <Models />}
      {panel === 'router' && <RouterPanel lastRun={lastRun} />}
      {panel === 'classify' && <Classify />}
      {panel === 'runs' && (
        <RunsPanel
          key={runIsolation?.key ?? 0}
          isolate={runIsolation?.value ?? null}
          onOpenRun={replayRun}
        />
      )}
      {panel === 'traces' && (
        <Traces runId={traceRunId} setRunId={setTraceRunId} autoKey={traceAuto} />
      )}
      {panel === 'sandbox' && <Sandbox />}
      {panel === 'context' && <ContextPanel />}
      {panel === 'config' && <ConfigPanel />}
    </div>
  );
}
