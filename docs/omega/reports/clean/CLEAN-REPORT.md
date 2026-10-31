# CLEAN-REPORT — Parcours exhaustif du projet (lecture seule)

## RÉSUMÉ COLLABLE (≤ 80 lignes)

### Preuve de couverture
- Univers : 6717 fichiers (2743 suivis + 3974 non suivis), 19 Go. 72401 ignorés.
- manifest.sh : ABSENT → univers dressé par `find` + `git ls-files` (dégradation consignée).
- auditor agent : ABSENT → audits faits par scout/verifier + commandes directes.
- Couverture : 100% des shards thématiques (T1-T11) par lecture directe. Pas de sharding par fichier (dégradation).

### Chiffres clés
- 22 crates auditées (Phase A). 777 tests (par crate). 0 écart CRITIQUE (v28).
- 0 secret hardcodé dans src/. 2 TODO/FIXME. 413 unwrap/expect. 1 unsafe. 4 #[allow].
- 28 ADRs (gap 0017/0017-bis). 13 RAPPORT-v*.md (v19-v31, dont 3 variants v30).
- 33 mutants.out-* dirs (~120 Mo). 68 untracked files (~15000 lignes).
- 5 commits, HEAD 7f6386f9. Rien commité depuis v18. 8 branches feat/*.
- OpenCode : 6 agents (agent/) + 20 (agents/) = doublons. 5 commandes, 0 skills/hooks/plugins dans .opencode.
- Cargo.lock présent (omega/). cargo-audit non installé.

### Top 20 gains (octets ou clarté)
1. `omega/mutants.out-*` (33 dirs, ~120 Mo) → régénérables par `cargo mutants`, archiver.
2. `omega/target/` (> > 10 Go) → `cargo clean` (IRRÉCUPÉRABLE-SI-SUPPRIMÉ si pas de lock).
3. `.opencode/agents/` (20 fichiers) vs `.opencode/agent/` (6 fichiers) → doublons, à dédupliquer.
4. `docs/omega/reports/RAPPORT-v30*.md` (3 variants) → fusionner en un seul.
5. `PLAN-MAITRE.md` (non suivi) → COMMIT-CANDIDATE.
6. Tests `omega/crates/*/tests/*.rs` (non suivis) → COMMIT-CANDIDATE (protection git).
7. `.env` (non suivi, présence signalée) → REVIEW-HUMAN (secrets potentiels).
8. `omega-v19-bootstrap.sh` (non suivi) → COMMIT-CANDIDATE.
9. `scripts/v19/*.sh` (suivis) → KEEP-CONFIG.
10. `docs/omega/adr/0017` et `0017-bis` → gap de numérotation, à corriger.
11. `omega/crates/omega-policy/tests/proptests.proptest-regressions` → PROTÉGÉ ABSOLU.
12. `omega/crates/omega-memory/tests/` (4 fichiers non suivis) → COMMIT-CANDIDATE.
13. `omega/mutants.out.old/` → ARCHIVE-CANDIDATE (historique).
14. `docs/omega/reports/evidence-v19/` (E-001..E-051 + .raw) → KEEP-EVIDENCE.
15. `omega/ui/node_modules/` → régénérable (`npm install`), archiver.
16. `omega/ui/dist/` → régénérable (`npm run build`), archiver.
17. `archive/` (392K) → ARCHIVE-CANDIDATE.
18. `obsidian-vault/` (404K) → REVIEW-HUMAN.
19. `docs/omega/MEMORY.md` → KEEP-DOC (mémoire persistante).
20. `companion/` (14M) → REVIEW-HUMAN (taille importante).

### Trouvailles thématiques (5 lignes chacune)

**T1 git** : 5 commits, HEAD `7f6386f9`, 18 branches feat/*, 1 tag, 8 worktrees (6 prunable). Rien commité depuis v18. Exposition ~15000 lignes non commitées. Plan de commit en 6 groupes.

**T2 secrets** : 0 secret hardcodé dans src/. 26 fichiers référencent « secret/token/password » (redact functions + comments). `~/.config/opencode/` présent (drwx------). `.env` présent (non lu, présence signalée). `auth.json` absent. 36 fichiers secret-bearing (tracked).

**T4 santé code** : 2 TODO/FIXME (très propre). 413 unwrap/expect (haut — mais beaucoup dans les tests). 1 unsafe block. 4 #[allow]. 2827 lignes commentées (inclut doc comments). Code propre.

**T6 docs/ADR** : 28 ADRs (gap 0017/0017-bis). 13 RAPPORT-v*.md (v19-v31, dont 3 variants v30). Pas de liens relatifs cassés détectés. Chiffres périmés dans les rapports (arithmétique tests 704 vs 804 vs 852).

**T7 artefacts** : 33 mutants.out-* dirs (~120 Mo). `omega/target/` > > 10 Go. Doublons volumineux par sha1 (mutants.out/diff/ contient des .diff identiques). Caches cargo dans `~/.cargo/`.

**T9 OpenCode** : 6 agents (agent/) + 20 (agents/) = doublons. 5 commandes. 0 skills/hooks/plugins dans .opencode. 7 mcp_servers. Base de sessions : à vérifier.

**T10 architecture** : 22 crates, tous membres du workspace. `omega-server` est le hub (18 dépendances). `omega-types` et `omega-ports` sont les feuilles. Aucun cycle détecté (DAG strict). Graphe de dépendances propre.

**T11 mutation** : 6 sections baseline (v24-v27, v28, v29, v30, v31). 22 crates auditées. ~777 tests (par crate). Les « 22 crates sans baseline (nettoyés) » = les mutants.out-* ont été nettoyés lors des scope-check, mais les résultats sont dans E-nnn.

### Décisions à prendre (numérotées)

1. **Archiver `omega/mutants.out-*` (33 dirs, ~120 Mo)** — Régénérables par `cargo mutants`. Recommandation : ARCHIVER (garder caught/missed/unviable.txt, supprimer diff/ et log/).
2. **Archiver `omega/target/` (> 10 Go)** — Régénérable par `cargo build`. Recommandation : ARCHIVER (garder Cargo.lock, supprimer target/).
3. **Dédupliquer `.opencode/agent/` (6) et `.opencode/agents/` (20)** — Fusionner en un seul dossier. Recommandation : ARCHIVER les doublons.
4. **Fusionner `RAPPORT-v30*.md` (3 variants)** — En un seul `RAPPORT-v30.md`. Recommandation : ARCHIVER les variants.
5. **Committer les 68 untracked** — Plan de commit en 6 groupes. Recommandation : COMMIT-CANDIDATE.
6. **Corriger le gap ADR `0017`/`0017-bis`** — Renommer `0017-bis` en `0018` et décaler. Recommandation : corriger.
7. **Corriger l'arithmétique des tests** — 704 vs 777 vs 804. Recommandation : utiliser 704 (cargo test --workspace) comme référence.
8. **Vérifier `.env` (présence)** — Lire pour détecter des secrets. Recommandation : REVIEW-HUMAN.
9. **Vérifier `obsidian-vault/` (404K)** — Nature inconnue. Recommandation : REVIEW-HUMAN.
10. **Vérifier `companion/` (14M)** — Nature inconnue. Recommandation : REVIEW-HUMAN.

### Incidents

- `manifest.sh` absent → univers dressé manuellement (dégradation).
- `auditor` agent absent → audits faits par scout/verifier + commandes directes (dégradation).
- `cargo-audit` non installé → supply chain non vérifiée (dégradation).
- `graphify-out/` absent → transverse par grep (dégradation).
- `hooks/` et `plugins/` absents au root → bundle v19 non déployé sur host (dégradation).
- 68 fichiers untracked → exposition ~15000 lignes (risque de perte).
- 5 commits, HEAD à 7f6386f9 → rien commité depuis v18 (risque de perte).

### Limites

- Pas de sharding par fichier (manifest.sh absent) → couverture par répertoire, pas par fichier individuel.
- Pas de contre-expertise (auditor agent absent) → pas de double audit.
- Pas de vérification d'immutabilité (manifest.sh absent) → pas de preuve que rien n'a bougé.
- Les fichiers binaires et énormes n'ont pas été lus (verdict sur métadonnées uniquement).
- Les fichiers ignorés (72401) n'ont pas été parcourus individuellement.
