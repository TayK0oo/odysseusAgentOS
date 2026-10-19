# Revue de code — W8 inc1 (`omega-otel`) + W9 inc1 (`omega-sandbox`)

> **Réviseur :** Reviewer SFD (indépendant, adversarial) · **Date :** 2026-10-15
> **Périmètre :** `omega/crates/omega-otel/src/*.rs`, `omega/crates/omega-sandbox/src/*.rs`
> (+ contrats `omega-ports`, `omega-types`, `omega/scripts/check_dep_graph.py`)
> **Exécutions :** `cargo test --workspace` (vert, 169 tests unitaires, 0 échec),
> `cargo clippy --workspace --all-targets -- -D warnings` (vert),
> `cargo fmt --all --check` (vert), `python3 scripts/check_dep_graph.py` (valide, 13 crates, 0 cycle, 0 arête interdite).
> **Reproductions :** mini-crate isolée sous `/tmp/opencode/advtest` (hors dépôt, dépendances `path` vers le workspace ; dépôt non modifié — `git status` propre).

---

## §1 Verdict

**NO-GO — score 4/10.**

W8 est fonctionnel mais **infidèle sur les cas limites** (perte silencieuse : étape sans `operation_id` ignorée, double marqueur terminal « dernier gagne », ligne JSONL corrompue sautée sans signal). W9 est **contournable et le timeout est mensonger** : le chaînage shell (`;`, `&&`, `|`, `$(...)`, backticks) traverse une règle `Allow` préfixe et **s'exécute réellement** (fichier témoin créé, prouvé §3 A9/A9b), et le kill par PID ne tue que le wrapper `sh` — **le petit-fils survit** (prouvé §3 A10). L'auto-revue de `W9-sandbox.md` §5 admet les subshells et la casse mais **oublie `;`/`&&`/`|`**, pires car ils n'exigent aucune règle `Allow` sur la seconde commande. Ces deux points sont des failles de sécurité, pas des dettes cosmétiques.

---

## §2 Constats (`fichier:ligne` + preuve + correctif)

### BLOQUANT

**B1 — W9 : la policy préfixe est contournée par les opérateurs shell ; la commande réellement exécutée n'est jamais celle qui a été autorisée.**
`omega-sandbox/src/policy.rs:55` (`trimmed.starts_with(&rule.prefix)` sur la chaîne complète) + `omega-sandbox/src/backend.rs:49-55` (`sh -c <commande>` après décision `Allow`). Preuves (§3) : `decide("echo hi; touch /tmp/adv_semi")` → `Allow`, et `backend.run("echo hi; touch /tmp/adv_semi_exec")` → `ok=true`, **fichier créé**. Idem `&&` (A4), `|` (A5), `$(...)` (A1→A9b, fichier créé), backticks (A2). Le contrat W9 §3 (« le refus précède tout fork ») n'est vrai que pour le préfixe, pas pour le graphe d'exécution shell.
**Correctif (au choix, sans réécriture) :** (a) en `decide`, refuser (`Forbidden`) toute commande contenant un métacaractère shell hors allowlist stricte (`; & | $ ` ` \` \n`, parenthèses) sauf si un mode `shell` est explicitement autorisé ; ou (b) exécuter sans shell (argv direct, pas `sh -c`) pour les règles `Allow` simples. Compléter `W9-sandbox.md` §5 avec `;`/`&&`/`|`.

**B2 — W9 : le timeout tue le wrapper `sh`, pas la cible ; le processus survit.**
`omega-sandbox/src/backend.rs:57-73` (`child.id()` = PID du `sh`, `kill -9 <pid>`). Preuve (§3 A10) : après `CommandTimeout` sur `sleep 20`, `pgrep -f "sleep 20"` retrouve un survivant (`200112`, nettoyé ensuite). Le contrat W9 §3 (« le processus est tué ») est faux pour tout `sh -c` qui fork (cas général). Le test existant `backend.rs:126-132` ne vérifie que le **type** d'erreur → victoire prématurée.
**Correctif :** placer l'enfant dans son groupe de processus (`std::os::unix::process::CommandExt::process_group(0)` + `kill -- -PID`) ou tuer l'arbre ; tester la mort réelle (cf. §4). Documenter « Linux-only » déjà fait (`backend.rs:61-62`), étendre au PGID.

**B3 — W8 : le replay perd des événements en silence ; le résumé devient mensonger.**
`omega-otel/src/replay.rs:66-72` et `74-81` (`if let Some(op)` : étape sans `operation_id` **ignorée sans compteur ni trace**) ; `replay.rs:83-84` (`run.completed` puis `run.failed` → **dernier gagne**, aucun conflit signalé). Preuves (§3) : B1 `steps=0` pour un `step.completed` sans `op_id`, résumé `total=3, completed=0` (le total compte l'événement, les steps l'ignorent — incohérent) ; B2 double terminal → `Failed` sans erreur.
**Correctif :** au minimum exposer les ignorés (ex. champ `skipped: usize` sur `ReplayedRun`/`RunSummary`, ou `operation_id` synthétique `seq:<n>`) ; double marqueur terminal → `ReplayedOutcome::Unknown` + flag `conflicting_outcomes: true` (ou erreur typée). Sans cela, `summarize` n'est pas une preuve rejouable.

### MAJEUR (devrait bloquer la promotion en inc2, pas le merge inc1 si tracé comme dette)

**M1 — W8 : `FileSink` ignore toutes les erreurs ; `read_all` saute les lignes corrompues sans signal.**
`omega-otel/src/sinks.rs:81-88` (`if let Ok(...)`, `let _ = write/flush` : IO en échec = événement perdu, zéro trace) ; `sinks.rs:58-64` (`filter_map(... .ok())`). Preuve (§3 D) : 3 lignes dont 1 corrompue → 2 lues, aucun signal. De plus le `Mutex` (`sinks.rs:73`) ne sérialise que **intra-processus** : deux processus émetteurs peuvent entrelacer leurs `write_all` (pas de verrou fichier), contrairement au contrat W8 §3 (« émissions concurrentes sérialisées » — vrai seulement intra-processus).
**Correctif :** compteur d'erreurs (`emit_failed: AtomicU64`) ou `try_emit -> Result` ; `read_all -> (Vec<_>, skipped)` ou `Result` ; documenter « mono-processus » ou verrou fichier (ex. `fs2`) en inc2.

**M2 — W8 : `MetricName` incohérent avec son contrat (trop strict d'un côté, trop lâche de l'autre).**
`omega-otel/src/metric.rs:20-37`. Preuves (§3 C) : `omega.http-client.req` **rejeté** (tiret interdit — pourtant idiomatique OTEL/kebab-case) ; mais `omega.a.b.c.d.e.f` (profondeur illimitée alors que le contrat dit `[.<détail>]` = max 4 segments), `omega.0.0`, `omega._a._b` **acceptés**. Le test `rejects_bad_names` ne couvre aucun de ces cas.
**Correctif :** trancher et aligner doc + code (autoriser `-` ? borner à 3–4 segments ? longueur max ? segments non vides de sens ?) ; ajouter les cas au test.

### MINEUR

- **m1 — `expect` en chemin prod.** `sinks.rs:73` (`lock().expect("sink empoisonne")`) : seul `panic!` potentiel hors tests du périmètre. **Correctif :** `lock().unwrap_or_else(|e| e.into_inner())`.
- **m2 — `Provenance.source` non validée.** `provenance.rs:32-35` (« jamais vide en pratique ») mais `tag(kind, "")` accepté. **Correctif :** `debug_assert!` ou validation.
- **m3 — `OtelContributor::name()` = préfixe d'IDs.** `contributor.rs:34-36` : l'identité du contributeur est confondue avec le préfixe de manifests (`format!("{}-{id}")`, `contributor.rs:41`). Fonctionnel, mais fragile si deux jeux de sinks partagent un préfixe. **Correctif :** champ `name` distinct.
- **m4 — Fragilité casse admise mais non testée.** `policy.rs:56` sensible à la casse ; `EcHo hi` → `Forbidden` deny-by-default (sûr, prouvé A6) mais aucun test ne fige ce comportement fail-closed. **Correctif :** tester `EcHo` → `Forbidden` (garde-fou anti-régression en attendant la normalisation inc2).

---

## §3 Cas adversariaux reproduits (sorties réelles de `/tmp/opencode/advtest`)

Policy de test : `rm→Forbidden`, `echo→Allow`, défaut deny-by-default (= config des tests du dépôt, `policy.rs:89-95` / `backend.rs:89-97`).

| # | Entrée | Résultat observé | Attendu |
|---|---|---|---|
| A1 | `decide("echo $(touch /tmp/adv_subshell)")` | `Allow` | `Forbidden` (subshell non autorisé) |
| A2 | `decide("echo \`touch /tmp/adv_backtick\`")` | `Allow` | `Forbidden` |
| A3 | `decide("echo hi; touch /tmp/adv_semi")` | `Allow` | `Forbidden` (2e commande non autorisée) |
| A4 | `decide("echo hi && touch /tmp/adv_and")` | `Allow` | `Forbidden` |
| A5 | `decide("echo hi \| tee /tmp/adv_pipe")` | `Allow` | `Forbidden` |
| A6 | `decide("EcHo hi")` | `Forbidden("deny-by-default")` | `Forbidden` ✓ (fail-closed, sûr mais fragile — cf. m4) |
| A7 | `decide("\techo hi")` | `Allow` | `Allow` ✓ (trim documenté `policy.rs:5`) |
| A8 | `decide("rm /tmp/x > /tmp/y")` | `Forbidden("non")` | `Forbidden` ✓ (préfixe simple couvert) |
| A9 | `run("echo hi; touch /tmp/adv_semi_exec")` | `ok=true`, **fichier créé** | refus avant fork |
| A9b | `run("echo $(touch /tmp/adv_subshell_exec)")` | `ok=true`, **fichier créé** | refus avant fork |
| A10 | `run("sleep 20")` timeout 200 ms | `CommandTimeout` en ~204 ms, puis `pgrep` → **`200112` survivant** (tué manuellement après constat) | aucun survivant |
| B1 | replay `step.completed` sans `operation_id` | `steps=0`, résumé `total=3, completed=0` | événement compté/signalé |
| B2 | replay `run.completed` + `run.failed` | `outcome=Failed`, sans erreur | conflit signalé |
| B3 | replay journal vide | `steps=0, outcome=Unknown` | ✓ (comportement sain, à figer en test) |
| C | `MetricName::parse` | `omega.a.b.c.d.e.f`→Ok, `omega.0.0`→Ok, `omega.http-client.req`→Err | à trancher (cf. M2) |
| D | `FileSink` + 1 ligne corrompue / 3 | 2 lus, 0 signal | erreur comptée (cf. M1) |

---

## §4 Tests manquants (à exiger avant GO)

**W9 (`backend.rs`, `policy.rs`) :**
1. `;` / `&&` / `||` / `|` / `$(...)` / backticks après un préfixe `Allow` → refus (ou dette explicitement acceptée avec garde-fou).
2. `Prompt` → `ApprovalRequired` **sans exécution** avec fichier témoin (le test témoin n'existe que pour `Forbidden`, `backend.rs:107-117`).
3. Timeout : **mort réelle vérifiée** (plus aucun PID/groupe après `CommandTimeout`), y compris petit-fils `sh -c "sleep N"`.
4. `EcHo` → `Forbidden` (fige le fail-closed) ; `\techo` → `Allow` (fige le trim).

**W8 (`replay.rs`, `sinks.rs`, `metric.rs`) :**
5. Étape sans `operation_id` : comportement explicite testé (comptée ou synthétisée, jamais silencieuse).
6. Double marqueur terminal : conflit détecté (`Unknown` + flag ou erreur).
7. Journal vide → `Unknown` / `total=0` (fige B3).
8. `seq` dupliqués / non monotones : ordre et déterminisme figés.
9. `FileSink` : émissions concurrentes multi-threads (N threads × M events = N×M lignes) ; ligne corrompue comptée ; échec IO (chemin non créable) signalé.
10. `MetricName` : profondeur > 4, segments triviaux (`0`, `_`), `-`, longueur extrême — selon la règle tranchée (M2).
11. Contributeur otel avec `FileSink` enregistré (seul `StdoutSink` est testé, `contributor.rs:59-72`).

---

## §5 Points solides (vérifiés, pas déclarés)

- **Refus avant fork structurel** pour les cas préfixe simples : `match policy.evaluate` (`backend.rs:33-47`) précède `spawn` (`backend.rs:49-55`) ; `Forbidden` et `Prompt` retournent sans exécution — `Prompt` n'est **jamais** exécuté en inc1 (file d'approbations = inc2, contrat tenu).
- **Erreurs typées correctes** : `ForbiddenCommand` / `ApprovalRequired` / `CommandTimeout` (`error.rs:46-56`) toutes trois exercées par les tests existants.
- **Deny-by-default réel** : `RuleBasedExecPolicy::new` (`policy.rs:35-41`) + première-règle-gagnante testée (`policy.rs:88-95`).
- **Replay déterministe sur le nominal** : tri par `seq` + `summarize` pur (`replay.rs:59-60`, `96-114`), tests d'ordre et de déterminisme présents.
- **Contributeurs conformes « ports = register_dyn »** : `OtelContributor` (`contributor.rs:40-47`) et `SandboxContributor` (`contributor.rs:41-52`), résolvables comme ports en test.
- **Architecture propre** : `omega-otel`/`omega-sandbox` ne dépendent que de `types`, `ports`, `registry`, `plugin-api` (jamais `core`/`app`) — confirmé par `Cargo.toml` + `check_dep_graph.py` ; zéro `unwrap`/`panic` en chemin prod hors `expect` du verrou (m1) — le reste (`unwrap_or`, `unwrap_or_default`, `unwrap_or_else`) est justifié, `unwrap` cantonnés aux tests.
- **Hygiène CI verte** : 169 tests OK, clippy `-D warnings` OK, `fmt --check` OK.
- **Auto-revue W9 §5 honnête mais incomplète** : les dettes subshell/casse sont admises (rare et appréciable) — il y manque `;`/`&&`/`|` (B1) et le PGID (B2).

---

# §Résolution — re-vérification indépendante (2026-10-15, commit `f68435a` + descendants `1de0172`, `40b3432`, `8a3a9be`)

> **Méthode :** mêmes assauts qu'en §3 + cas demandés, rejoués depuis mini-crate isolée `/tmp/opencode/advtest2` (dépôt non modifié, `git status` propre hors ce fichier). Policy de test identique (`echo→Allow`, `rm→Forbidden`, défaut deny).

## Portes (gates)

| Porte | Résultat | Commentaire |
|---|---|---|
| `cargo test --workspace` | **VERT** — 180 tests, 0 échec (+11 vs revue initiale, dont `compound_command_escalates_to_prompt`, `timeout_kills_grandchildren_not_just_sh`, `steps_without_operation_id_are_counted_not_lost`, `conflicting_terminal_markers_first_wins`, `file_sink_counts_io_errors`, `accepts_hyphenated_names`) | — |
| `cargo clippy --workspace --all-targets -- -D warnings` | **ROUGE hors périmètre** : `omega-app/examples/live_ollama.rs:114` (`format!` dans `println!`, commit W2 `40b3432`, fichier non touché par `f68435a`). **Vert sur le périmètre** (`-p omega-sandbox -p omega-otel -p omega-ports -p omega-types`) | À corriger côté W2, pas bloquant W8/W9 |
| `cargo fmt --all --check` | **ROUGE hors périmètre** (`live_ollama.rs`, `ollama.rs`, commit W2). Aucun diff sur `omega-sandbox`/`omega-otel` | Idem |
| `python3 scripts/check_dep_graph.py` | **1 violation hors périmètre** : `omega-app (app) -> omega-router (router)` (dépendance ajoutée par W2 `40b3432`) | À trancher côté archi W2, pas une régression du correctif |

## Chaque constat → statut + preuve

- **B1 — CORRIGÉ.** 20 cas rejoués end-to-end (décision + témoin) : `;`, `&&`, `||`, `|`, `$(...)`, backticks, `>`, `&`, `\n` réel, `<(...)`, `$(...)`/backtick **dans** `"..."` → tous `ApprovalRequired`, **témoin jamais créé**. `EcHo …; touch` → `ForbiddenCommand`, non exécuté. Cas sains préservés : `\techo hi` → `Allow` exécuté ; `echo "a;b"`, `echo 'a;b'` → `Allow` (stdout `a;b`) ; `echo $HOME` → `Allow` ; `echo a\;b` → `Allow` littéral ; `echo '$(…)'` → `Allow` littéral sans exécution ; `'...'` non fermé → `Allow` mais erreur shell (exit 2), témoin absent ; variable piégée `ADV_EVIL="; touch …"` → `Allow`, **contenu non re-parsé** (fichier absent — sémantique shell correcte). `evaluate` n'est consommé que par `backend.rs:36` : la garde couvre tout le chemin d'exécution.
- **B2 — CORRIGÉ (cas général).** `sh bg.sh` (`sleep 61 & wait`, timeout 300 ms) → `CommandTimeout`, `pgrep` vide : **aucun survivant**. Le test `timeout_kills_grandchildren_not_just_sh` est **significatif** : démo témoin kill-par-PID-seul → 2 survivants (parent + petit-fils), donc il échoue sur l'ancien code et passe sur le nouveau. Réserve : voir N1.
- **B3 — CORRIGÉ.** Étape sans `operation_id` → `steps=1, ignored_events=1, total=4` (comptée, plus de silence). Double marqueur contradictoire → **premier gagne + `outcome_conflict=true`** dans les deux ordres (C-puis-F → `Completed` ; F-puis-C → `Failed`). Double marqueur identique → pas de faux conflit (`Completed`, `false`).
- **M1 — PARTIEL.** Côté émission : **CORRIGÉ** — `FileSink::emit_errors()` = 1 sur chemin `/proc` non créable (prouvé). Côté lecture : **NON CORRIGÉ** — `read_all` (`sinks.rs:68-74`, inchangé) saute toujours la ligne corrompue sans signal (prouvé : 3 lignes dont 1 corrompue → 2 lues, `emit_errors=0`). Verrou toujours intra-processus seul (doc `sinks.rs:38-39` inchangée).
- **M2 — CORRIGÉ.** `omega.http-client.req` → `Ok` ; borne 3–8 segments appliquée (`omega.a.b` Ok, 9+ segments Err) ; doc alignée (`[a-z0-9_-]`, 3-8) + tests des deux côtés. Résidu assumé (charset seul) : `omega.0.0`, `omega._a._b` toujours `Ok` — conforme à la doc, pas un bug.
- **m1 — NON CORRIGÉ** (inchangé `sinks.rs:83`, `expect` sur le verrou) — toujours mineur.
- **m2, m3** — non re-testés, inchangés (mineurs).
- **m4 — NON CORRIGÉ (test manquant).** Comportement sûr confirmé (`EcHo` → `Forbidden`, prouvé) mais **aucun test ne fige le fail-closed casse** (pas de `EcHo` dans `policy.rs` tests).

## Nouveaux constats

- **N1 (MAJEUR-limite, non bloquant) — le kill de groupe ne tue pas le petit-fils qui change volontairement de session.** Repro : `sh daemon.sh` (`setsid sleep 64 & wait`, timeout 300 ms) → `CommandTimeout` mais **`sleep 64` (PID 216676, vérifié `pgrep -af`) survit** : `setsid` quitte le PGID, `kill -9 -pgid` ne l'atteint plus. Portée : exige un préfixe `Allow` couvrant `sh` + charge délibérément daemonisante (pas le `&` courant, lui corrigé). **Correctif inc2 :** cgroup/pidfd ou interdiction `setsid`/`daemon` ; **a minima documenter la limite** (actuellement le commentaire `backend.rs:60-62` promet « le groupe entier », sans cette exception).
- **N2 (mineur, assumable) — sur-blocage bénin.** `echo $((1+1))` (arithmétique inoffensive) escalade en `Prompt` car `$(` déclenche la garde. Fail-closed donc sûr ; à noter si plainte UX en inc2 (distinguer `$(` de `$((`).
- **N3 (hors périmètre, à router) — portes workspace rouges depuis W2** (clippy `live_ollama.rs`, fmt `live_ollama.rs`+`ollama.rs`, arête `omega-app→omega-router`). Non causés par `f68435a`, mais la mention mémoire « clippy/fmt/graphe verts » (176 tests) est devenue fausse au niveau workspace — requalifier en « verts sur le périmètre W8/W9 ».

## Verdict mis à jour

**GO conditionnel — score 8/10** (contre NO-GO 4/10).

B1/B2/B3 tiennent contre tous les assauts rejoués + 5 contournements inédits (`\n`, quote non fermée, `$((…))`, `<(…)`, env piégée) ; M2 tranché et testé ; M1 moitié émission faite. Conditions avant promotion inc2 (dettes tracées, pas de re-revue) : (1) `read_all` signale les lignes corrompues (ou API `(Vec, skipped)`) ; (2) documenter la limite N1 (`setsid`) + test `EcHo→Forbidden` (m4) ; (3) remettre au vert les 3 portes hors périmètre côté W2. Dépôt non modifié par cette re-vérification hormis le présent fichier.
