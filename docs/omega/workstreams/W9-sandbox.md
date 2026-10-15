# W9 — Sécurité & sandbox (`omega-sandbox`)

> **Workstream :** W9 · **Statut :** 🟢 incrément 1 · **Date :** 2026-10-15
> **Sources :** `OMEGA-BLUEPRINT.md` §3 (capacités), `RE-GOOSE-CODEX.md` (sandbox typé).
> **Provenance :** `[observed]` (implémenté) · `[inferred]` · `[todo]`.

---

## 1. Objectif

Aucune commande ne s'exécute sans décision de politique **typée et tracée** :
`Allow` → exécute (borné), `Prompt` → approbation requise, `Forbidden` → refusé sans fork.

## 2. Périmètre incrément 1

| Élément | Rôle |
|---|---|
| `omega-types::ExecDecision` | `Allow` / `Prompt(motif)` / `Forbidden(motif)` |
| `omega-ports::ExecPolicy` | Port de la politique (slot exclusif, capacité `exec-policy`) |
| `RuleBasedExecPolicy` | Règles ordonnées préfixe → décision, deny-by-default |
| `LocalSandboxBackend` | Backend `sh -c` borné (timeout, kill par PID) |
| `SandboxContributor` | Câble politique + backend (`exec-policy` + `sandbox-backend`) |
| `OmegaError::{ForbiddenCommand, ApprovalRequired, CommandTimeout}` | Erreurs typées |

## 3. Contrats

- Le refus (`Forbidden`/`Prompt`) **précède tout fork** : prouvé par test (fichier témoin intact).
- Timeout : le processus est tué (kill par PID, Linux) ; `CommandTimeout` typée.
- Première règle qui matche décide ; sans match → `Forbidden("deny-by-default")`.

## 4. Tests d'acceptation

| # | Critère | Statut |
|---|---|---|
| (a) | Commande autorisée exécutée, stdout/exit capturés | ✅ |
| (b) | Commande interdite **jamais exécutée** (témoin intact) | ✅ |
| (c) | `Prompt` → `ApprovalRequired` sans exécution | ✅ |
| (d) | Timeout → processus tué + `CommandTimeout` | ✅ |
| (e) | Contributeur : politique + backend résolvables | ✅ |

## 5. Auto-revue (incrément 2 livré)

- **Dette résolue (inc2) — matching par préfixe aveugle aux subshells** :
  `RuleBasedExecPolicy::strict()` (ou `.with_strict(true)`) interdit les
  substitutions `$(...)`, backticks et `<(...)` **même quotées** (en doubles
  quotes elles s'exécutent) → `Forbidden("substitution interdite en mode
  strict")`. En mode non-strict (défaut) : comportement inchangé (`Prompt` via
  la garde anti-composition du backend). `[observed]` (tests
  `strict_mode_forbids_substitutions_quoted_or_not`,
  `substitution_detector_fires_even_quoted`).
- **Dette résolue (inc2) — casse** : le **nom de commande** (premier mot) est
  normalisé en minuscules avant matching (`EcHo` matche `echo`) ; les
  **arguments gardent leur casse**. `[observed]` (test
  `command_name_case_is_normalized_but_not_args`).
- **File d'approbations (inc2, lève le `Prompt`)** : `ApprovalQueue`
  (`request(command, reason) -> ApprovalId`, `approve(id)`, `deny(id)`,
  `pending()`, consommation à usage unique via `consume_for(commande exacte)`).
  `LocalSandboxBackend::with_approvals(queue)` : devant un `Prompt`, une
  approbation `Approved` pour la commande exacte est consommée puis la commande
  s'exécute ; sinon `ApprovalRequired` sans fork. `Forbidden` ne consulte jamais
  la file. `[observed]` (tests `approval_queue_runs_once_then_asks_again`
  — exécute une fois puis re-demande — et `denied_approval_never_executes`).
- Kill par PID : Linux-only assumé Phase 1 (documenté dans le code).

## 6. Reste (incrément 3)

- Backends Docker/SSH (adaptateurs du port `SandboxBackend`).
- Isolation renforcée Linux (Landlock/seccomp, bubblewrap, cgroup pour le kill
  de groupe — mitige N1 `setsid`).
- `prune_denied()` : les refus sont purgés explicitement (pas d'accumulation
  silencieuse) — déjà implémenté + testé.
