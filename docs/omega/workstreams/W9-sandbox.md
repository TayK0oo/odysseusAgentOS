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

## 5. Reste (incrément 2)

- File d'approbations humaines + `ApprovalReviewContributor` (lever `Prompt`).
- Backends Docker/SSH (adaptateurs du port `SandboxBackend`).
- Isolation renforcée Linux (Landlock/seccomp, bubblewrap).
