# Agent OS — Références Maîtres

> **Dossier unique** regroupant les 3 documents fondateurs du projet.
> Tout développement, toute décision, toute migration part de ces 3 fichiers.

---

## 01 — SFD v3.0 (Spécification Fonctionnelle Détaillée)

**Fichier :** `01-SFD-v3.0.md` (88 KB, 1065 lignes)

Ce que le système **DOIT** faire :
- 22 principes invariants (Constitution)
- 20 modules fonctionnels (§5.1 à §5.20)
- 19 exigences non-fonctionnelles (§6)
- 19 cas d'usage (UC-01 à UC-19)
- Architecture logique 7 couches (§7)
- Feuille de route 20 étapes (§8)

**Statut :** 20/20 modules implémentés → migrés vers OpenCode natif

---

## 02 — Analyse Outils Tiers

**Fichier :** `02-OUTILS-TIERS.md` (81 KB)

Ce que le système **PEUT** utiliser :
- 60 outils analysés (Mai-Juin 2026)
- 23 intégrés, 7 patterns absorbés, 6 en veille, 11 en banque, 14 ignorés
- Grille d'évaluation 7 axes (score /35)
- Architecture 5 couches (Connaissance, Exécution, Design, Observabilité, Automation)

**Statut :** Outils clés intégrés (CBM, Graphify, Scrapling, Serena, Kroki, Design Extract...)

---

## 03 — Objectifs Système & Migration Design

**Fichier :** `03-OBJECTIFS-SYSTEME.md` (3 KB)

Ce que le système **EST** et **VA DEVENIR** :
- Architecture cible : Odysseus = UI, OpenCode = cœur, Plugins = features SFD
- 7 plugins npm (@agentos/sfd-*)
- 16 agents natifs (.opencode/agents/)
- 2 skills (.opencode/skills/)
- Bridge subprocess/API

**Statut :** Migration terminée, 8 014 lignes supprimées, système opérationnel

---

## Ordre de lecture

1. **03-OBJECTIFS** — Comprendre la vision et l'architecture cible
2. **01-SFD** — Comprendre ce que le système doit faire
3. **02-OUTILS** — Comprendre avec quoi le faire
