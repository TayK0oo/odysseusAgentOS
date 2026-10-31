# PLAN-NETTOYAGE-PROPOSE — Groupes A, B, C

## Prérequis (avant exécution)

1. **Sauvegarde vérifiée** : `scripts/backup.sh` ou copie manuelle du dépôt.
2. **Commit de protection des COMMIT-CANDIDATE** : `git add -A && git commit -m "backup: pre-clean"` (ou équivalent) pour protéger les fichiers non suivis.
3. **Retour arrière** : `git stash` ou `git reset --hard HEAD` pour revenir à l'état actuel.

## Groupe A — Sûr (régénérable ou doublon) — ~120 Mo + target

### A1 — Archiver mutants.out-* (33 dirs, ~120 Mo)

**Commande proposée** :
```bash
# Archiver les .txt et .json, supprimer diff/ et log/
for d in omega/mutants.out-*; do
  mkdir -p archive/mutants/$(basename $d)
  cp $d/*.txt $d/*.json archive/mutants/$(basename $d)/ 2>/dev/null
  rm -rf $d/diff $d/log
done
```

**Prérequis** : vérifier que `MUTATION-SURVIVORS-BASELINE.md` contient les résultats (caught/missed/unviable/timeout) pour chaque crate.

**Retour arrière** : `git checkout -- omega/mutants.out*` (si tracked) ou restauration depuis l'archive.

### A2 — Archiver omega/target/ (> 10 Go)

**Commande proposée** :
```bash
# Sauvegarder Cargo.lock (déjà tracké)
cp omega/Cargo.lock /tmp/Cargo.lock.backup
# Supprimer target/
rm -rf omega/target/
```

**Prérequis** : `Cargo.lock` est tracké et intact. `cargo build` peut le régénérer.

**Retour arrière** : `cargo build --workspace` (régénère target/).

### A3 — Archiver omega/ui/node_modules/ et omega/ui/dist/

**Commande proposée** :
```bash
rm -rf omega/ui/node_modules omega/ui/dist
```

**Prérequis** : `package.json` et `package-lock.json` sont trackés.

**Retour arrière** : `cd omega/ui && npm install && npm run build`.

### A4 — Archiver archive/ (392K)

**Commande proposée** :
```bash
# Déjà dans archive/, rien à faire. Vérifier le contenu.
ls archive/
```

**Prérequis** : vérifier que `archive/` ne contient rien de critique (code de production, secrets).

**Retour arrière** : `git checkout -- archive/` (si tracked) ou restauration depuis backup.

## Groupe B — Archivage (historique utile, inutile en vivant)

### B1 — Dédupliquer .opencode/agent/ et .opencode/agents/

**Commande proposée** :
```bash
# Comparer les deux dossiers
diff -r .opencode/agent/ .opencode/agents/ 2>/dev/null | head -20
# Archiver le doublon
mkdir -p archive/opencode-agents-duplicate
cp -r .opencode/agents/* archive/opencode-agents-duplicate/ 2>/dev/null
# Garder le dossier canonique (.opencode/agent/ ou .opencode/agents/ selon lequel est utilisé)
```

**Prérequis** : vérifier quel dossier est utilisé par OpenCode (probablement `.opencode/agent/`).

**Retour arrière** : `cp -r archive/opencode-agents-duplicate/* .opencode/agents/`.

### B2 — Fusionner RAPPORT-v30*.md (3 variants)

**Commande proposée** :
```bash
# Garder RAPPORT-v30.md comme canonique, archiver les variants
mkdir -p archive/reports-v30
cp docs/omega/reports/RAPPORT-v30-bis.md docs/omega/reports/RAPPORT-v30-ter.md archive/reports-v30/
# Fusionner le contenu des variants dans RAPPORT-v30.md (manuel)
```

**Prérequis** : vérifier que RAPPORT-v30.md contient les informations essentielles.

**Retour arrière** : `cp archive/reports-v30/* docs/omega/reports/`.

### B3 — Archiver mutants.out.old/ (si existe)

**Commande proposée** :
```bash
rm -rf omega/mutants.out.old
```

**Prérequis** : vérifier que `mutants.out.old/` ne contient rien d'unique (comparer sha1 avec `mutants.out/`).

**Retour arrière** : `git checkout -- omega/mutants.out.old/` (si tracked).

## Groupe C — À décider (humain)

### C1 — Committer les 68 untracked (COMMIT-CANDIDATE)

**Commande proposée** (par groupe, dans l'ordre) :
```bash
# Groupe 1 : bundle v19
git add omega-v19-bootstrap.sh .opencode/ scripts/v19/ hooks/ plugins/
git commit -m "feat: bundle v19 — agentOS omega orchestration"

# Groupe 2 : ADRs
git add docs/omega/adr/0023-*.md docs/omega/adr/0024-*.md docs/omega/adr/0025-*.md docs/omega/adr/0026-*.md docs/omega/adr/0027-*.md
git commit -m "docs: ADR-0023 à 0027"

# Groupe 3 : tests
git add omega/crates/*/tests/*.rs
git commit -m "test: tests comportementaux v21-v30 (mutation + intégration)"

# Groupe 4 : code (state.rs)
git add omega/crates/omega-server/src/state.rs
git commit -m "feat: branchement W6 dans pipeline (ADR-0024)"

# Groupe 5 : rapports
git add docs/omega/reports/RAPPORT-v*.md docs/omega/reports/MUTATION-SURVIVORS-BASELINE.md
git commit -m "docs: rapports v19-v31 + baseline mutation"

# Groupe 6 : plan maître
git add PLAN-MAITRE.md docs/omega/
git commit -m "docs: plan maître + vision"
```

**Prérequis** : vérifier que chaque groupe compile (`cargo check --workspace`).

**Retour arrière** : `git reset --soft HEAD~6` (annule les 6 commits, garde les changements).

### C2 — Corriger le gap ADR 0017/0017-bis

**Commande proposée** :
```bash
# Renommer 0017-bis en 0018 et décaler
mv docs/omega/adr/0017-bis-*.md docs/omega/adr/0018-*.md
# Mettre à jour les références dans les autres ADRs
grep -rl "0017-bis" docs/omega/adr/ | xargs sed -i 's/0017-bis/0018/g'
```

**Prérequis** : vérifier qu'aucun autre fichier ne référence `0017-bis`.

**Retour arrière** : `git checkout -- docs/omega/adr/`.

### C3 — Vérifier .env (présence signalée)

**Commande proposée** :
```bash
# Lire .env (ATTENTION: peut contenir des secrets)
cat .env
```

**Prérequis** : accepter le risque de lire des secrets (déjà signalé dans le rapport).

**Retour arrière** : aucune modification.

### C4 — Vérifier obsidian-vault/ (404K) et companion/ (14M)

**Commande proposée** :
```bash
ls -la obsidian-vault/ companion/
du -sh obsidian-vault/ companion/
```

**Prérequis** : vérifier la nature des dossiers (code, docs, données).

**Retour arrière** : aucune modification.

### C5 — Corriger l'arithmétique des tests

**Commande proposée** :
```bash
# Mettre à jour les rapports avec le bon compte (704)
sed -i 's/777/704/g' docs/omega/reports/RAPPORT-v27.md docs/omega/reports/RAPPORT-v28-coherence.md
```

**Prérequis** : vérifier que 704 est le bon compte (cargo test --workspace).

**Retour arrière** : `git checkout -- docs/omega/reports/`.

## Résumé

| Groupe | Action | Gain estimé | Risque |
|---|---|---|---|
| A1 | Archiver mutants.out-* | ~120 Mo | Faible (régénérable) |
| A2 | Archiver omega/target/ | ~10 Go | Faible (régénérable) |
| A3 | Archiver node_modules + dist | ~200 Mo | Faible (npm install) |
| A4 | Archiver archive/ | 392 Ko | Faible (déjà dans archive/) |
| B1 | Dédupliquer .opencode/agents | ~4 Ko | Moyen (vérifier lequel est canonique) |
| B2 | Fusionner RAPPORT-v30*.md | ~10 Ko | Moyen (fusion manuelle) |
| B3 | Archiver mutants.out.old/ | ~20 Mo | Faible |
| C1 | Committer les 68 untracked | Protection git | Moyen (6 commits, hooks) |
| C2 | Corriger gap ADR 0017 | Clarté | Faible |
| C3 | Vérifier .env | Sécurité | Moyen (secrets) |
| C4 | Vérifier obsidian-vault/ + companion/ | Clarté | Faible |
| C5 | Corriger arithmétique | Cohérence | Faible |

**Décision finale** : humain. Le plan est proposé, pas exécuté. Chaque groupe peut être exécuté indépendamment. Le retour arrière est documenté pour chaque groupe.
EOF
echo "PLAN-NETTOYAGE-PROPOSE.md written"