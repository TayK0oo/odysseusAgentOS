# PAUSE 2026-10-21 — état gelé du Lot A (P1)

> Reprise : lire ce fichier, `journal.md`, puis `STATUS.json`. Le HEAD au gel est `831a4145` (README : A6 mergé).

## Ce qui est mergé (1/6)

- **A6 hygiène** — `831a4145` dans `feat/agentos-omega` : ruff épinglé 0.6.9 (venv cache, jamais le PATH ; legacy 3720→1040, échelle verify) ; STATUS regénéré ; doc-truth VERT.

## Ce qui est gelé (branches survivent au wipe de /tmp)

| Unité | Branche | SHA du gel | Contenu (récit des agents, à re-vérifier en reprise) |
|---|---|---|---|
| A1 portes | `feat/p1-A1-portes` | `677ffdf3` + `cc821144` | ADR-0013 (mort du faux vert, G1 exécutable), gates.toml réoutillé, S1 déjà outil_reel_canary, S2 = limites. ** complexion : scripted_provider.py touché, README scénarios écrit, RG?"
| A2 boucle | `feat/p1-A2-boucle` | `cd8214aa` + `065dc19c` + tiède | Port ToolSurface, détection ReAct+native, budget monotone ; contrat rouge (065dc19c) puis implémentation (cd8214aa). Test `contract_tool_loop.rs` retouché au gel. **Non compilé vert lors du gel. |
| A3 politique | `feat/p1-A3-politique` | `78de85c1` | `budget.rs` + `tiers.rs` non commités avant gel — SANS commit rouge. Paliers T0–T3 inachevés. |
| A4 adaptateurs | `feat/p1-A4-adaptateurs` | `44f20a39` + `05026efe` | test rouge `/api/plugins` (8 adaptateurs attendus), composition.rs touché au gel. |
| A5 sandbox | `feat/p1-A5-sandbox` | `9a05b21a` + `18f1dcaf` | bwrap fail-closed : 8 tests rouge (évasion FS/réseau, binaire absent, null, unicode, concurrence) ; `timeout.rs` nouveau au gel. |

Tous les worktrees : `/tmp/opencode/wt-A1..A6` (recréer si wipe : `git worktree add /tmp/opencode/wt-Ax feat/p1-Ax`).

## Reprise (protocole)

1. Vérité : `git log`, `STATUS.json`, `gate.py --phase 1` ; **l'attendu rouge sincère** — si encore VERT placeholder, A1 n'est pas intégré.
2. Fusionner un par un **A1 → A3 → A4 → A2 → A5** (détail : A1 d'abord = porte verte honnête à vivre le plus longtemps ; A2 la boucle, le plus gros chantier, intégré après sa dépendance A3 politique), `verify.py` après chaque fusion.
3. Lot B après : P1-5 (robustesse : repli ReAct, JSON malformé, boucle, troncature, timeouts) dépend d'A2 ; puis fuzz parseur tool-calls + mutation politique/jail (A5/A6 du plan).
4. Deux rouges consécutifs sur un scénario qui était vert → stop fonctionnalités.
5. Fin : STATUS regénéré au HEAD, journal 5 lignes, push.

## Faits CI à ne pas re-découvrir

- Les runs `omega-gates.yml` JOUENT sur chaque push et sont **rouges by design** (S0/S1 ECHEC). Pas d'action OAuth nécessaire.
- Toujours `gh -R TayK0oo/odysseusAgentOS` (fork : sinon résolution upstream → 404).
- G0.e = « CI exécute gate.py --phase 0 » : le job existe et joue — le rouge actuel est le S0/S1 attendu (P0-4 : S0 PASS local mais rouge en CI = écart à comprendre en reprise).
