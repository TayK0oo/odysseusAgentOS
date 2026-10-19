# Programme de reverse-engineering — 4 étapes reproductibles

> Objectif : extraire **chaque fonctionnalité** des harnais d'agents **les plus
> utilisés**, la vérifier par **preuve machine**, et en faire **un combiné
> unifié** pour notre système. Aucune étape n'est manuelle : tout est
> régénérable par script.

## Vue d'ensemble

```
A. Recensement          harness_census.py     popularite mesuree (API GitHub)
        |
B. Intake + REA         harness_intake.py     clone + Evidence MCP par harnais
        |
C. Dissection           (agents, schema)      chaque fonctionnalite -> capacite/port/adaptateur
        |
D. Combine unifie       omega_complet.py      une seule feuille de route pour Omega
```

## A. Recensement — `omega/scripts/harness_census.py`

- **Popularite = preuve** : etoiles, forks, derniere activite, licence, langage
  (API GitHub). Aucun jugement manuel : le classement est mecanique.
- Sorties : `docs/omega/reverse-eng/HARNESS-CENSUS.md` et
  `omega/crates/omega-harness/data/harnesses_census.json`.
- Le script identifie les **ecarts** : harnais populaires non encore disseques.

## B. Intake + REA — `omega/scripts/harness_intake.py`

- Clone en profondeur 1 dans `$OMEGA_RE_CLONES` (defaut `/home/tayk0oo/omega-re`).
- Detection de nature : `js` (package.json), `rust` (Cargo.toml), `go` (go.mod),
  `python` (pyproject/requirements).
- **REA** : analyse JavaScript profonde pour le JS/TS (`analyze-javascript-application`),
  vue d'ensemble sinon (`analyze`). Chaque resultat est une **Evidence** MCP avec
  `provider`, `confidence` (`observed`/`derived`) et **limites declarees**.
- Sorties : `HARNESS-INTAKE.md` + `docs/omega/research/rea/evidence/intake-*.summary.json`.
- Limites constatees (non contournees) : `resource_constraint` sur les depots
  tres volumineux (Void, Qwen Code, OpenClaw), pas de Ghidra/Hopper/IDA, pas de
  navigateur (CDP requis pour `analyze-web-bundle`).
- **Limite majeure, constatee sur le terrain** : REA n'a **pas d'analyseur de
  source Python/Rust/Go**. Sur ces depots, `analyze` retombe sur
  `rea-javascript-application` (il ne voit que le JS incident) : le graphe est
  minuscule (`graph_nodes: 3`) et `confidence: derived`. **La preuve machine ne
  corrobore donc que le clonage, les etoiles, le langage et la licence** — tout le
  fond technique vient de la lecture du clone (`fichier:ligne`). Les dissections
  doivent le declarer explicitement (c'est fait pour gpt-engineer, CrewAI,
  MetaGPT, LangGraph, GPT Pilot, Tabby). REA reste **pleinement probant sur le
  JS/TS** (Gemini CLI, Continue, Kilo Code, AgentGPT, OpenCode, Cline) et sur les
  binaires ELF (`pwntools-elf`, `confidence: observed`).

## C. Dissection — un agent par harnais

- Schema : `DISSECTION-SCHEMA.md` (champs obligatoires, table d'honnetete pour
  `adopted`, `rust_cost` en `S|M|L`).
- Regle de concurrence : **un ecrivain par arbre** — chaque agent n'ecrit que
  `data/apps/<id>.json` et `dissections/<id>.md`.
- Chaque fonctionnalite cite `fichier:ligne` (ou une URL de doc) : la preuve est
  dans le JSON (`refs`), pas dans le recit.
- `status` n'est `adopted` **que** si l'equivalent existe deja dans notre code.

## D. Combine unifie — `omega/scripts/omega_complet.py`

Produit `docs/omega/reverse-eng/OMEGA-COMPLET.md` :

1. **synthese** (harnais, fonctionnalites, capacites, popularite mesuree) ;
2. **consensus** : capacites reclamees par >= 4 harnais, **ponderees par les etoiles** ;
3. **combiné par famille** : capacite -> harnais demandeurs -> **exemplaire de
   reference** (`file:line`) -> port -> adaptateur -> cout ;
4. **feuille de route** : Phase 1 (socle, cout S), Phase 2 (structure, M),
   Phase 3 (lourd, L) ;
5. **non-objectifs** : fonctionnalites ecartees, avec raison ;
6. **provenance** : dissections + preuves REA.

Complement deterministe : `omega/scripts/integration_plan.py` (classement par
nombre de harnais demandeurs) — les deux se regenerent a chaque vague de
dissections.

## Boucle de verification

```bash
python3 omega/scripts/harness_census.py          # A
python3 omega/scripts/harness_intake.py --top N  # B
# C : agents de dissection (1 par harnais)
python3 omega/scripts/integration_plan.py        # D (consensus)
python3 omega/scripts/omega_complet.py           # D (combine)
cd omega && cargo test --workspace               # le corpus embarque reste valide
python3 omega/scripts/check_dep_graph.py
```

## Etat de la derniere vague

- 57 harnais recenses, **32 ecarts** identifies ;
- **14 harnais** intake (clone + REA) : 12 analyses, 2 `resource_constraint` ;
- dissections en cours (14 nouveaux) ;
- combine regenere a chaque vague ; popularite mesuree jointe aux dissections.
