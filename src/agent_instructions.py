"""
Agent System Instructions — Encodes the complete project lifecycle
best practices guide into the agent's behavior.

When in AGENT mode (not chat), the system follows this structured
approach for every project, applying the 10-phase lifecycle from
the "Guide Complet — Bonnes Pratiques Dev & Conception".

Mapping: Guide 10 phases → SFD 7 phases → Agent actions
"""

from __future__ import annotations

from typing import Optional

# ============================================================================
# Phase mapping: Guide lifecycle → SFD phases
# ============================================================================

GUIDE_TO_SFD_PHASE_MAP = {
    # Guide Phase 1: Avant-projet (Discovery & Cadrage)
    "avant_projet": {
        "sfd_phase": "CLASSIFY",
        "actions": [
            "Identifier le problème réel (5 pourquoi)",
            "Lister les contraintes non négociables",
            "Évaluer la faisabilité technique",
            "Définir les critères de succès mesurables",
        ],
        "checklist": [
            "Problème formulé clairement (pas la solution)",
            "Charte de projet créée",
            "Registre des risques initial créé",
            "Budget et délai estimés avec marge",
        ],
    },
    # Guide Phase 2: Conception (Design & Architecture)
    "conception": {
        "sfd_phase": "KNOW",
        "actions": [
            "Rédiger les specs fonctionnelles (user stories)",
            "Définir l'architecture (C4 model si pertinent)",
            "Documenter les choix via ADR",
            "Modéliser les données (SQL/NoSQL)",
            "Identifier les exigences non-fonctionnelles",
        ],
        "checklist": [
            "User stories avec critères d'acceptation",
            "Architecture documentée (≥1 ADR)",
            "Modèle de données validé",
            "Menaces sécurité identifiées (STRIDE)",
        ],
    },
    # Guide Phase 3: Initialisation (Setup)
    "initialisation": {
        "sfd_phase": "PLAN",
        "actions": [
            "Définir la structure du repo",
            "Choisir le Git workflow (GitHub Flow recommandé)",
            "Mettre en place CI/CD (lint → build → test → deploy)",
            "Configurer les environnements (dev/staging/prod)",
            "Externaliser la configuration (.env)",
        ],
        "checklist": [
            "Repo initialisé avec conventions documentées",
            "Pipeline CI/CD fonctionnel",
            "Secrets gérés hors du code",
            "README + CONTRIBUTING rédigés",
        ],
    },
    # Guide Phase 4: Développement
    "developpement": {
        "sfd_phase": "BUILD",
        "actions": [
            "Appliquer les standards de code (SOLID, DRY, KISS, YAGNI)",
            "Écrire les tests AVANT ou AVEC le code (TDD si logique complexe)",
            "Code review systématique (PR < 400 lignes)",
            "Tracker la dette technique (tickets, pas TODO)",
            "Auditer les dépendances (vulnérabilités)",
            "Feature flags pour découpler déploiement et release",
        ],
        "checklist": [
            "Lint + format vérifiés automatiquement",
            "Tests écrits avec la fonctionnalité",
            "PR review obligatoire avant merge",
            "Dette trackée dans le backlog",
            "Documentation à jour (code + API)",
        ],
    },
    # Guide Phase 5: Gestion de projet (transversal)
    "gestion_projet": {
        "sfd_phase": "BUILD",
        "actions": [
            "Maintenir le backlog priorisé (INVEST)",
            "Suivre les métriques (vélocité, burndown, DORA)",
            "Gérer les risques en continu",
            "Communiquer avec les parties prenantes",
        ],
        "checklist": [
            "Backlog priorisé et affiné",
            "Registre des risques à jour",
            "Changement de scope = analyse d'impact",
        ],
    },
    # Guide Phase 6: Qualité & Tests
    "qualite": {
        "sfd_phase": "QUALITY",
        "actions": [
            "Exécuter la pyramide de tests (unit > intégration > e2e)",
            "Vérifier la Definition of Done",
            "Scanner la sécurité (SAST + dépendances)",
            "Tests de performance si release majeure",
        ],
        "checklist": [
            "Plan de tests exécuté",
            "Definition of Done respectée",
            "Scan sécurité passé",
            "Recette validée (UAT)",
        ],
    },
    # Guide Phase 7: Déploiement & Release
    "deploiement": {
        "sfd_phase": "BUILD",
        "actions": [
            "Versionner selon SemVer",
            "Générer le changelog (Keep a Changelog)",
            "Choisir la stratégie de déploiement (rolling/blue-green/canary)",
            "Vérifier la checklist go-live",
            "Préparer le plan de rollback",
        ],
        "checklist": [
            "Version taguée SemVer",
            "Changelog à jour",
            "Checklist go-live validée",
            "Plan de rollback testé",
            "Communication de lancement envoyée",
        ],
    },
    # Guide Phase 8: Maintenance
    "maintenance": {
        "sfd_phase": "MEMORY_OBSERVE",
        "actions": [
            "Activer le monitoring (logs + métriques + traces)",
            "Configurer l'alerting (pas d'alert fatigue)",
            "Documenter le process d'incident (post-mortem sans blâme)",
            "Définir SLA/SLO/SLI",
            "Tester les sauvegardes régulièrement",
        ],
        "checklist": [
            "Monitoring actif sur 3 piliers",
            "Alerting calibré",
            "Process incident documenté",
            "Sauvegardes testées",
        ],
    },
    # Guide Phase 9: Évolution
    "evolution": {
        "sfd_phase": "AUTOEVAL",
        "actions": [
            "Collecter le feedback utilisateur",
            "Analyser l'impact avant nouvelle feature",
            "Refactoring continu (boy scout rule)",
            "Planifier les migrations (expand/contract)",
        ],
        "checklist": [
            "Nouvelle feature = mini-cycle complet",
            "Feedback utilisateur exploité",
            "Refactoring intégré en continu",
            "Politique de dépréciation communiquée",
        ],
    },
    # Guide Phase 10: Culture (transversal)
    "culture": {
        "sfd_phase": "MEMORY_OBSERVE",
        "actions": [
            "Documenter les décisions (ADR)",
            "Maintenir la doc à jour (as code)",
            "Appliquer le post-mortem sans blâme",
            "Partager la connaissance (bus factor > 1)",
        ],
        "checklist": [
            "Documentation versionnée avec le code",
            "Décisions structurantes en ADR",
            "Post-mortems systématiques après incident",
        ],
    },
}

# ============================================================================
# System Prompt — Agent Mode (injecté dans le contexte BUILD)
# ============================================================================

AGENT_SYSTEM_PROMPT = """# Agent OS — Mode Agent (Cycle de Vie Complet)

Tu es en mode AGENT. Tu vas suivre un cycle de vie projet complet
en 7 phases, en appliquant les bonnes pratiques de développement
logiciel à chaque étape.

## Règles fondamentales

1. **Commence simple, complexifie sur preuve** (P11 SFD)
2. **Contexte construit, pas déversé** (P3 SFD) — ne donne que l'info pertinente
3. **Toute action longue est durable** (P14 SFD) — retry, reprise après panne
4. **Chaque fait stocké a une provenance** (P16 SFD) — [stated]/[observed]/[inferred]
5. **L'humain est SUR la boucle, pas DANS la boucle** (P10 SFD)

## Cheminement en 7 phases

### PHASE 1 — CLASSIFY (Avant-projet)
- Analyse le besoin : quel est le VRAI problème ? (5 pourquoi)
- Évalue le risque : FAIBLE / MODÉRÉ / ÉLEVÉ / CRITIQUE
- Identifie les contraintes non négociables
- Check : le problème est formulé, pas la solution

### PHASE 2 — KNOW (Conception)
- Recherche dans la mémoire et l'historique
- Définit l'architecture (C4 model si pertinent)
- Documente les choix via ADR (pourquoi cette techno, ce pattern)
- Modélise les données (SQL > NoSQL par défaut)
- Check : architecture documentée, menaces sécurité identifiées

### PHASE 3 — PLAN (Initialisation)
- Décompose en objectifs → sous-projets → tâches
- Chaque tâche a : description, critères de finition, budget
- Définit la structure du projet
- Check : plan validé, CI/CD prévu, secrets externalisés

### PHASE 4 — BUILD (Développement)
- Code selon SOLID, DRY, KISS, YAGNI
- Écris les tests AVEC le code (TDD si logique complexe)
- PR < 400 lignes, review obligatoire
- Feature flags pour découpler déploiement/release
- Check : lint OK, tests passent, PR reviewée, doc à jour

### PHASE 5 — QUALITY (Qualité & Tests)
- Pyramide de tests : unitaires > intégration > e2e
- Vérifie la Definition of Done
- Scan sécurité (injection, XSS, dépendances)
- Check : tous les tests passent, scan sécurité OK

### PHASE 6 — AUTOEVAL (Évaluation)
- Auto-évalue le résultat vs critères de succès
- Mesure la performance (temps, tokens, coût)
- Identifie les leçons apprises
- Check : critères atteints, leçons extraites

### PHASE 7 — MEMORY_OBSERVE (Mémoire & Maintenance)
- Écris dans la mémoire avec provenance [stated]/[observed]
- Mets à jour les compétences (compteur aidant/nuisant)
- Configure le monitoring post-déploiement
- Check : mémoire écrite, monitoring actif, sauvegardes testées

## Checklist Globale (validée à la fin de chaque projet)

- [ ] Problème clarifié, pas la solution
- [ ] Architecture documentée (ADR)
- [ ] Repo structuré, CI/CD fonctionnel
- [ ] Tests écrits avec le code, review systématique
- [ ] Definition of Done respectée
- [ ] Version SemVer, changelog à jour
- [ ] Plan de rollback testé
- [ ] Monitoring actif, alerting calibré
- [ ] Documentation à jour, post-mortem si incident

## Règles de sortie visuelle

- Diagrammes pour l'architecture (Mermaid via Kroki)
- Tableaux pour les comparaisons
- Code avec syntax highlighting
- Pas de visuel si le texte suffit (YAGNI)
"""

# ============================================================================
# Quick reference: Guide checklist → SFD phase checklist
# ============================================================================

def get_phase_checklist(sfd_phase: str) -> list[str]:
    """Get the checklist items for a given SFD phase, derived from the guide."""
    items = []
    for guide_phase, mapping in GUIDE_TO_SFD_PHASE_MAP.items():
        if mapping["sfd_phase"] == sfd_phase:
            items.extend(mapping["checklist"])
    return items


def get_phase_actions(sfd_phase: str) -> list[str]:
    """Get the actions for a given SFD phase, derived from the guide."""
    actions = []
    for guide_phase, mapping in GUIDE_TO_SFD_PHASE_MAP.items():
        if mapping["sfd_phase"] == sfd_phase:
            actions.extend(mapping["actions"])
    return actions


def get_full_checklist() -> list[str]:
    """Get the complete checklist across all phases."""
    items = []
    for mapping in GUIDE_TO_SFD_PHASE_MAP.values():
        items.extend(mapping["checklist"])
    return items
