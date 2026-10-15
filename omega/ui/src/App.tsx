import { useState } from 'react';
import type { PluginInfo, RouterDecisionDto, RunEvent } from './protocol';

/**
 * Omega Cockpit — squelette des 5 panneaux (W7 inc1).
 * Chaque panneau répond à une question : que fait le système ? pourquoi ?
 * combien ça coûte ? où est le contexte ?
 * Données branchées sur `omega-server` en inc2 (pour l'instant : fixtures).
 */

type Panel = 'cockpit' | 'plugins' | 'router' | 'traces' | 'context';

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

function Cockpit() {
  return (
    <section>
      <h2>Cockpit</h2>
      <p>Run <code>live-1</code> : terminé, 2/2 étapes, coût nul (local).</p>
      <ol>
        {FIXTURE_EVENTS.map((e, i) => (
          <li key={i}>
            <code>{e.name}</code>
          </li>
        ))}
      </ol>
    </section>
  );
}

function Plugins() {
  return (
    <section>
      <h2>Plugin Explorer</h2>
      <table>
        <thead>
          <tr>
            <th>Plugin</th>
            <th>Capacité</th>
            <th>Actif</th>
          </tr>
        </thead>
        <tbody>
          {FIXTURE_PLUGINS.map((p) => (
            <tr key={p.id}>
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

function Router() {
  return (
    <section>
      <h2>Router Dashboard</h2>
      <p>
        Modèle : <code>{FIXTURE_DECISION.model_id}</code> (score{' '}
        {FIXTURE_DECISION.score.toFixed(2)})
      </p>
      <p>
        Raison : <code>{FIXTURE_DECISION.reason}</code>
      </p>
    </section>
  );
}

function Traces() {
  return (
    <section>
      <h2>Trace Viewer / Replay</h2>
      <p>Séquence rejouée sans ré-exécuter (cf. `TraceReplayer`).</p>
    </section>
  );
}

function Context() {
  return (
    <section>
      <h2>Context Inspector</h2>
      <p>Budget : estimation déterministe, coupes journalisées (`AssembleReport`).</p>
    </section>
  );
}

export default function App() {
  const [panel, setPanel] = useState<Panel>('cockpit');
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
      {panel === 'cockpit' && <Cockpit />}
      {panel === 'plugins' && <Plugins />}
      {panel === 'router' && <Router />}
      {panel === 'traces' && <Traces />}
      {panel === 'context' && <Context />}
    </main>
  );
}
