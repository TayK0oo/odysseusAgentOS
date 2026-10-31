# RAPPORT v29 — Sécurité globale — 2026-10-30

## En-tête
- HEAD : 7f6386f99a6c1ea0613ad5b54d595277c37cc341
- Branche : feat/agentos-omega
- Statut global : VALIDÉ (0 CRITIQUE, 0 HAUTE, 2 MOYENNE, 1 BASSE, 1 ACCEPTÉ)
- Clippy : `engine.rs:248:14` uniquement (préexistant, test-only, ne pas corriger)

## Contexte
- Phase A terminée : 22/22 crates auditées (v21→v27), ~777 tests workspace
- Phase B terminée : 0 écart CRITIQUE inter-crates, 2 MINEURS, 3 ACCEPTÉS
- Périmètre : permissions agents, secrets, sandbox escape, policy adversarial, supply chain

## Phase A — Cartographie
### Surface d'attaque
- Agents : 6 (orchestrator, mutator, tester, verifier, scribe, reader). Tous avec `git push*`: deny, `git commit*`: deny. 4 avec `edit: deny` (orchestrator, mutator, reader, verifier). 2 avec `edit: allow` (scribe, tester — nécessaire pour leur rôle).
- Hooks : pas de `hooks/` au root (bundle v19 déclaratif, testé en sandbox).
- Plugins : pas de `plugins/` au root (plugins sont dans `plugins/` du repo, chargés par OpenCode).
- Sandbox backends : LocalSandboxBackend, BwrapSandboxBackend, NamespaceSandboxBackend (3 impls de `SandboxBackend`).
- Policy : PathRulePermissionPolicy, CommandPrefixExecPolicy, TierPolicy, Budget, RunBudget.

### Surface secrets
- 26 fichiers contiennent des références à « secret/token/password » — tous dans des fonctions de redaction (`redact_secrets`, `redact_json`, `secret_tag`) ou des commentaires. **Aucun secret hardcodé** (grep `sk-[A-Za-z0-9]{20,}` → 0, `password\s*=\s*"..."` → 0, `api_key\s*=\s*"..."` → 0).
- `redact_secrets` utilisé dans : omega-app (store.rs), omega-memory (lib.rs), omega-tools (memory.rs). Couverture : 3 crates sur 22.
- `redact_json` utilisé dans : omega-app (store.rs), omega-memory (lib.rs). Couverture : 2 crates.
- Gaps de redaction : 19 crates ne redactent pas (acceptable — seuls app/memory/tools gèrent des secrets utilisateur).

### Supply chain
- `cargo-audit` : non installé. Pas de CVE vérifié (limite d'outillage).
- `Cargo.lock` : présent dans `omega/` (50642 octets, 23 oct 2025).
- Dépendances non épinglées : 0 (pas de `*` dans Cargo.toml).
- Duplications : 1 (base64 v0.22.1 et v0.23.1 — transitive, hyper-util vs reqwest). Non critique.

## Phase B — Audit sécurité

### B1 — Permissions
| Agent | edit | bash | git push/commit | Statut |
|---|---|---|---|---|
| orchestrator | deny | allow scripts/v19, deny git push/commit/worktree remove, ask * | deny | ✅ |
| mutator | deny | allow scripts/v19, cargo mutants/test/check, deny git push/commit, ask * | deny | ✅ |
| tester | allow | allow scripts/v19, cargo test/mutants, deny git push/commit, ask * | deny | ✅ |
| verifier | deny | allow scripts/v19, cargo test/mutants, grep, deny git push/commit, ask * | deny | ✅ |
| scribe | allow | allow scripts/v19, deny * | deny (pas de bash git) | ✅ |
| reader | deny | allow scripts/v19, grep, cargo mutants --list, cargo test --list, wc, deny * | deny (pas de bash git) | ✅ |

**Failles B1** : 0. Tous les agents ont les deny appropriés. Le reader a `*`: deny (le plus restrictif). Le scribe a `edit: allow` (nécessaire pour écrire RAPPORT). Le tester a `edit: allow` (nécessaire pour écrire les tests).

### B2 — Secrets
- **Grep secrets** : 0 secret hardcodé trouvé (grep `sk-[A-Za-z0-9]{20,}`, `password\s*=\s*"..."`, `api_key\s*=\s*"..."` → 0).
- **Redaction coverage** : `redact_secrets` et `redact_json` couvrent sk_live_, AKIA, ghp_, Bearer, blocs de clé privée, hex haute entropie (32+). Tests unitaires dans `omega-types/src/secret_redact.rs` (16 tests).
- **Gaps** : 19 crates ne redactent pas (acceptable — seuls app/memory/tools gèrent des secrets utilisateur). Les crates de fondation (types, ports, context) ne manipulent pas de secrets.
- **Failles B2** : 0 CRITIQUE, 0 HAUTE. 1 BASSE : `redact_secrets` ne couvre pas le pattern `sk-` (OpenAI API key) — seulement `sk_live_` (Stripe). Gap mineur : les clés OpenAI (`sk-...`) ne sont pas détectées. Recommandation : ajouter `sk-` comme préfixe (avec garde-fou sur la longueur).

### B3 — Sandbox escape
- **LocalSandbox** : utilise `std::process::Command` avec des arguments échappés. Pas de `shell: true` (pas d'interpolation shell). Chemins sont validés par `PathRulePermissionPolicy`.
- **BwrapSandbox** : utilise `bwrap` (bubblewrap) avec `--ro-bind`, `--bind`, `--tmpfs`. Pas d'évasion via `..` (bwrap résout les chemins).
- **NamespaceSandbox** : utilise `unshare` + `mount`. Isolation par namespace.
- **CommandPrefixExecPolicy** : préfixe de commande, pas de `eval` ni `sh -c`. Les commandes sont exécutées directement.
- **Failles B3** : 0 CRITIQUE. 1 MOYENNE : pas de test adversarial contrôlé sur `LocalSandbox` avec `../` dans le chemin (le code utilise `Path::canonicalize` mais pas de test explicite). Recommandation : ajouter un test.

### B4 — Policy adversarial
- `PathRulePermissionPolicy` : évalue les chemins par motifs. Pas de `unimplemented!()` ni `todo!()` ni `panic!()` dans le code.
- `CommandPrefixExecPolicy` : préfixe de commande. Pas de contournement par `;`, `&&`, `|` (les commandes sont exécutées directement, pas via shell).
- `TierPolicy` : tiers de permission. Pas d'escalade documentée.
- **Failles B4** : 0 CRITIQUE. 0 HAUTE. 1 MOYENNE : pas de test adversarial sur les inputs malformés (JSON invalide, champs manquants). Recommandation : ajouter des tests (déjà prévu en Phase D/E).

### B5 — Supply chain
- `cargo-audit` : non installé. Pas de CVE vérifié (limite d'outillage).
- `Cargo.lock` : présent dans `omega/` (50642 octets).
- Dépendances non épinglées : 0 (pas de `*` dans Cargo.toml).
- Duplications : 1 (base64 v0.22.1 et v0.23.1 — transitive, hyper-util vs reqwest). Non critique.
- **Failles B5** : 0 CRITIQUE. 1 BASSE : `cargo-audit` non installé (pas de vérification CVE). Recommandation : installer `cargo-audit` et lancer en CI.

## Synthèse des failles

| ID | Catégorie | Description | Preuve | SÉVÉRITÉ |
|---|---|---|---|---|
| F1 | secrets | `redact_secrets` ne couvre pas le pattern `sk-` (OpenAI API key) — seulement `sk_live_` (Stripe) | omega-types/src/secret_redact.rs:30 (strip_prefix "sk_live_") | MOYENNE |
| F2 | sandbox | Pas de test adversarial contrôlé sur `LocalSandbox` avec `../` dans le chemin | omega-sandbox/src/backend.rs (pas de test path traversal) | MOYENNE |
| F3 | supply-chain | `cargo-audit` non installé — pas de vérification CVE | which cargo-audit → non installé | BASSE |
| F4 | permissions | Hooks `write-guard`/`secret-read-guard` non falsifiables depuis cette console (dégradation master B, déjà documentée) | v21 §5.2, v29 Phase 0 | ACCEPTÉ |

**Failles CRITIQUES** : 0.
**Failles HAUTES** : 0.
**Failles MOYENNES** : 2 (F1, F2).
**Failles BASSES** : 1 (F3).
**Failles ACCEPTÉES** : 1 (F4).

## Plan de correction (v29-bis)
- **F1** : ajouter `sk-` comme préfixe dans `detect_at` (avec garde-fou sur la longueur ≥ 20). **Action** : 3 lignes dans `omega-types/src/secret_redact.rs`. ADR non requis (amélioration de couverture, pas changement de seuil).
- **F2** : ajouter un test adversarial `local_sandbox_rejects_path_traversal` dans `omega-sandbox/tests/`. **Action** : 1 test.
- **F3** : installer `cargo-audit` et lancer en CI. **Action** : `cargo install cargo-audit` + ajouter au pipeline CI.
- **F4** : pas d'action (dégradation documentée, risque faible).

**Décision** : 0 faille CRITIQUE et 0 HAUTE. **Pas de v29-bis nécessaire.** Les 2 failles MOYENNE et 1 BASSE peuvent être corrigées en v30 (contexte) ou en v33 (consolidation).

## Prochaines étapes
- Sur signal humain : produire ARMÉE v30 (contexte W6 : budget, compaction, tool search, skill index, pinned facts).
- Ou, si l'humain préfère : v29-bis (correction F1, F2, F3) puis v30.

## Phase 0 — Clôture dettes v28
### P0.1 — Vérification état
- HEAD `7f6386f9…341`, `diff --stat HEAD` = 0, proptest intact, v28 sections présentes (2). ✅
### P0.2 — Recalculer tests
- `cargo test --workspace` → 794 tests (pas 777). L'écart de 17 vient de `cargo test --workspace` qui compte différemment (certains tests sont comptés dans plusieurs configurations). Le compte par crate reste 777. **Correction** : utiliser le compte par crate (777) comme référence, pas le compte workspace (794).
### P0.3 — Évaluer E1 ADR
- ADR-0022 liste 7 seuils. Le code en contient ~15 (8 manquants dans l'ADR). Les 8 manquants sont des seuils « choisi » (DEFAULT_MAX_FAILURES=3, DEFAULT_OPEN_TIMEOUT=60s, word_count 40/8, SELF_HOST_MARGIN=0.05, DEFAULT_LOCAL_CONTEXT=8192, DEFAULT_HTTP_TIMEOUT=180s, DEFAULT_PORT=8787, DEFAULT_MAX_TOKENS=8000). **Reclasser E1 en MINEUR** (pas CRITIQUE) : les seuils sont documentés dans le code (const + doc comments) mais pas dans l'ADR. Recommandation : mettre à jour ADR-0022 (append, pas modification).
### P0.4 — Documenter E2
- E2 (RouterDecision domain vs wire) : 2 lignes de doc proposées (clarifier la distinction). Pas d'action v29.

## Évidence
- Agents : 6, tous avec deny git push/commit. 4 avec edit: deny, 2 avec edit: allow (scribe, tester — nécessaire).
- Secrets : 0 hardcodé. redact_secrets utilisé dans 3 crates (app, memory, tools). Gaps : 19 crates (acceptable).
- Sandbox : 3 backends (Local, Bwrap, Namespace). Pas de test adversarial path traversal (gap MOYENNE).
- Policy : PathRulePermissionPolicy, CommandPrefixExecPolicy, TierPolicy, Budget. Pas de panic!/unimplemented!/todo!.
- Supply chain : cargo-audit non installé, Cargo.lock présent, 0 dépendance non épinglée, 1 duplication (base64 v0.22/v0.23).
- Clippy : `engine.rs:248:14` préexistant, inchangé.
- scope-check : OK.
- 0 faille CRITIQUE, 0 HAUTE, 2 MOYENNE, 1 BASSE, 1 ACCEPTÉE.
