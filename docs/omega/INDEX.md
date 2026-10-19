# INDEX — Espace Omega (carte complète)

> **Généré automatiquement** — `docs/omega/INDEX.md`  
> Source de vérité : `docs/omega/MEMORY.md` + `docs/omega/rules/RULES-OMEGA.md` + `docs/omega/00-VISION-OMEGA.md`

---

## 0. Entrées principales

| Fichier | Rôle | Lecture recommandée |
|---------|------|---------------------|
| **`docs/omega/PROJET-ET-VISION.md`** | **Document unique : projet, inventaire des fichiers, état réel, plan** | **1er —Start here** |
| `AGENTS.md` | Première lecture de tout agent | 2ème |
| `docs/omega/rules/RULES-OMEGA.md` | Règles directrices (autorité) | 2ème |
| `docs/omega/MEMORY.md` | Mémoire persistante (état, décisions, credentials) | 3ème |
| `docs/omega/00-VISION-OMEGA.md` | Vision cible (6 piliers) | 4ème |
| `docs/omega/02-WORKSTREAMS.md` | Découpage en chantiers W1→W10 | 5ème |
| `docs/omega/RECAP-COMPLET.md` | Synthèse complète session | Contexte global |

---

## 1. Décisions (ADR) — `docs/omega/adr/`

| ADR | Sujet | Statut |
|-----|-------|--------|
| 0001 | Moteur Rust (réécriture complète) | ✅ Accepté |
| 0002 | UI React/TS séparée | ✅ Accepté |
| 0003 | Architecture plugin/capacités | ✅ Accepté |

---

## 2. Règles — `docs/omega/rules/`

| Fichier | Description |
|---------|-------------|
| `RULES-OMEGA.md` | Règles directrices complètes |
| `MODEL-POLICY.md` | Politique modèles (gratuits par défaut, payants interdits) |
| `model-policy.json` | Version machine de la politique |
| `model_policy.py` | Script de vérification (`--check`, `--for <usage>`) |

---

## 3. Recherche — `docs/omega/research/`

### 3.1 TikTok @0xloucash — `docs/omega/research/loucash/`
- `videos_enriched.json` — 78 vidéos, légendes enrichies
- `transcripts.json` — Oral transcrit (Whisper)
- `ocr.json` — Écran OCR (Tesseract)
- Scripts: `transcribe.py`, `frames_ocr.py`, `enrich.py`

### 3.2 REA — `docs/omega/research/rea/`
- `REA.md` — Identification + chaîne de preuve
- `CAPABILITIES.md` — Inventaire vérifié
- `snapshot.json` / `capabilities.json` — Données structurées
- `SWEEP.md` / `HARNESS-SWEEP.md` — Bilans
- `evidence/` — **28 fichiers de preuves**

### 3.3 Modèles — `docs/omega/research/models/`
- `opencode-go_classement_2026-10-17.md` — Modèles Go (quota)
- `opencode-gratuits_classement_2026-10-17.md` — Modèles gratuits vérifiés
- `opencode_sources_2026-10-17.md` — Sources de la politique

### 3.4 Harnais — `docs/omega/research/harness/` (liens vers reverse-eng)

---

## 4. Reverse-Engineering — `docs/omega/reverse-eng/`

| Fichier | Description |
|---------|-------------|
| `HARNESS-CENSUS.md` | 66 harnais recensés (popularité GitHub API) |
| `HARNESS-INTAKE.md` | Fiche d'entrée standardisée |
| `OMEGA-COMPLET.md` | **Combiné unifié** : 108 capacités, exemplaire référence, feuille route |
| `NOYAU-ULTIME.md` | **Architecture cible noyau** : 7 règles, modèle coutures, écart mesuré, plan 4 phases |
| `dissections/` | 35 fiches détaillées + données JSON embarquées dans `omega-harness` |

---

## 5. Code — Workspace Rust `omega/`

### 5.1 Crates (16) — Ordre de dépendances
```
omega-types → omega-ports → omega-config → omega-registry → omega-plugin-api
    → omega-core → omega-protocol → omega-app
    → omega-context → omega-otel → omega-sandbox → omega-harness
    → omega-mcp → omega-router → omega-providers → omega-server
```

### 5.2 Catégories (garde-fou)
| Catégorie | Crates | Règle |
|-----------|--------|-------|
| `types` | omega-types | Aucune déps omega-* |
| `ports` | omega-ports | Déps: types seulement |
| `adapters` | omega-sandbox, omega-harness, omega-mcp, omega-otel, omega-context | Déps: types, ports, registry, plugin-api |
| `core` | omega-core | Déps: types, ports |
| `app-edge` | omega-app, omega-server, omega-router, omega-providers | Déps: tout |

### 5.3 Scripts — `omega/scripts/`
| Script | Rôle |
|--------|------|
| `census.py` | Recensement harnais (API GitHub) |
| `intake.py` | Intake + REA harnais |
| `dissect.py` | Dissection → fiche + données |
| `validate_dissections.py` | Validation structure + preuves |
| `register_dissection.py` | Enregistrement dans omega-harness |
| `plan_integration.py` | Plan d'intégration (consensus capacité × demandeurs) |
| `complet.py` | Génération OMEGA-COMPLET.md |
| `check_dep_graph.py` | **Graphe de dépendances** (0 cycle, 0 arête interdite) |
| `model_policy.py` | Vérification politique modèles |
| `generate_memory.py` | Génération MEMORY.md |
| `run-local.sh` | Lance système complet (UI + API) |

---

## 6. UI React/TS — `omega/ui/`

| Élément | Description |
|---------|-------------|
| `src/` | Code source (Vite + TS) |
| `src/pages/` | 5 onglets: Cockpit, Plugins, Router, Traces, Contexte |
| `src/components/` | Composants réutilisables |
| `src/hooks/` | Hooks personnalisés |
| `omega-protocol` | Contrats générés (Typescript) |
| `omega-server` | Backend Axum (SPA fallback) |

---

## 7. Configuration OpenCode — `.opencode/`

| Fichier | Rôle |
|---------|------|
| `.opencode/rules/omega.md` | Règles session (chargé auto) |
| `.opencode/rules/model-policy.md` | Politique modèles (chargé auto) |
| `.opencode/agents/` | 82 agents découverts nativement |
| `.opencode/skills/` | Skills GSD + projet |
| `.opencode/command/gsd/` | Commandes GSD (discuss, plan, execute, etc.) |

---

## 8. Planning GSD — `.planning/`

| Fichier | Description |
|---------|-------------|
| `PROJECT.md` | Projet Omega (bootstrap depuis docs/omega/) |
| `REQUIREMENTS.md` | Exigences R1→R10 + NF1→NF9 |
| `ROADMAP.md` | Phases 1→5 (NOYAU A→D + 3 trous) |
| `STATE.md` | État courant, phase active, open items |
| `phases/` | Artefacts par phase (CONTEXT, PLAN, SUMMARY, VERIFICATION) |

---

## 9. Ressources lourdes (hors dépôt) — `/home/tayk0oo/omega-re/`

> **Non versionnés** (14 Go) — Clones + outils + preuves complètes

| Ressource | Emplacement |
|-----------|-------------|
| 31 dépôts clonés | `omega-re/clones/` |
| Ghidra + 15 moteurs RE | `omega-re/tools/` |
| Preuves REA complètes | `omega-re/evidence/` |
| Environnements Python | `omega-re/venvs/` |

---

## 10. Commandes utiles

```bash
# État
git branch --show-current
git log --oneline -10

# Qualité Rust
cd omega && cargo test && cargo clippy

# Qualité Python (socle)
ruff check . && pytest -q

# RE TikTok
python3 docs/omega/research/loucash/enrich.py

# Génération docs auto
python3 omega/scripts/generate_memory.py

# Lancement complet (requiert Ollama + qwen2.5-coder:7b)
omega/scripts/run-local.sh

# Vérification graphe deps
python3 omega/scripts/check_dep_graph.py

# Politique modèles
python3 omega/scripts/model_policy.py --check
```

---

## 11. Provenance des artefacts

| Artefact | Provenance | Dernière MAJ |
|----------|------------|--------------|
| `MEMORY.md` | `generate_memory.py` [observed] | Auto à chaque run |
| `INDEX.md` | Maintenu manuellement [stated] | 2026-10-17 |
| `RECAP-COMPLET.md` | Session 15-17 oct [observed] | 2026-10-17 |
| `OMEGA-COMPLET.md` | `complet.py` + corpus [observed] | 2026-10-17 |
| `NOYAU-ULTIME.md` | Synthèse corpus [inferred] | 2026-10-17 |
| `MODEL-POLICY.md` | Recherche 3 fichiers [stated] | 2026-10-17 |

---

## 12. Règles de contribution

1. **Tout est un plugin** — Port + registre, rien en dur
2. **Docs d'abord, code ensuite** — Port, contrat, test d'acceptation avant code
3. **Déterminisme au système** — Parsing, calcul, formatage = scripts, jamais LLM
4. **Provenance obligatoire** — `[stated]/[observed]/[inferred]` sur chaque affirmation
5. **Zéro exécution directe** — Orchestrer → déléguer → vérifier
6. **Une décision = un ADR** — Dans `docs/omega/adr/`
7. **Modèles gratuits par défaut** — Vérifier `model_policy.py --check` avant tout changement

---

*Carte maintenue à jour manuellement. Régénérer `MEMORY.md` via script après changements majeurs.*
