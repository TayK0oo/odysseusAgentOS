# 📚 Documentation — Odysseus AgentOS

> Hub central de la documentation. Chaque section renvoie aux fichiers détaillés.

## 🚀 Démarrage rapide

| Vous voulez… | Lire |
|---|---|
| Installer et lancer le projet | [→ Setup / Installation](setup/installation.md) |
| Comprendre l'architecture | [→ Architecture / Vue d'ensemble](architecture/overview.md) |
| Comprendre la boucle agent | [→ Architecture / Boucle agent](architecture/agent-loop.md) |
| Configurer les variables d'environnement | [→ Setup / Configuration](setup/configuration.md) |
| Déployer en production | [→ Operations / Déploiement](operations/deployment.md) |
| Comprendre la sécurité | [→ Sécurité / Modèle de menace](security/threat-model.md) |
| Contribuer au code | [→ Development / Conventions](development/guidelines.md) |
| Lancer les tests | [→ Development / Tests](development/testing.md) |

## 📖 Documentation par thème

### 🏗 Architecture
- [Vue d'ensemble](architecture/overview.md) — schéma global, couches, flux
- [Boucle agent et orchestration](architecture/agent-loop.md) — 7 phases canoniques, kill-switches, LangGraph
- [Services](architecture/services.md) — description de chaque service (Meilisearch, Qdrant, n8n, OPA…)
- [Flux de données](architecture/data-flow.md) — diagrammes de séquence

### ⚙️ Setup
- [Prérequis](setup/prerequisites.md) — Docker, Python 3.14, Node.js
- [Installation](setup/installation.md) — Docker Compose, natif, GPU
- [Configuration](setup/configuration.md) — variables d'env, `model-routing.json`, `phase-lock.yaml`

### 💻 Development
- [Conventions](development/guidelines.md) — code style, Git, PR
- [Tests](development/testing.md) — stratégie, taxonomie, CI
- [Debug](development/debugging.md) — logs, traces, observabilité

### 🚀 Operations
- [Déploiement](operations/deployment.md) — Docker, Traefik, GPU
- [Sauvegarde/Restauration](operations/backup-restore.md) — `scripts/odysseus-backup`
- [Observabilité](operations/observability.md) — LangFuse, OpenTelemetry, dashboards

### 🔒 Sécurité
- [Modèle de menace](security/threat-model.md) — synthèse du `THREAT_MODEL.md`
- [Authentification et permissions](security/auth-permissions.md) — rôles, gates, phase-lock
- [Sandboxing](security/sandboxing.md) — Docker hardening, gVisor, OPA

### 🔌 Intégrations
- [Email / Outlook](integrations/email-outlook.md)
- [Workflows n8n](integrations/n8n-workflows.md)
- [Autres outils](integrations/other-tools.md) — Discord, Telegram, CalDAV, CardDAV

### 📋 Référence
- [Glossaire](reference/glossary.md) — terminologie partagée
- [Résumé SFD](reference/sfd-summary.md) — spécification fonctionnelle en bref
- [Index maître](reference/index-master.md) — table des matières ultime du projet

---

## 📁 Documents racine (conservés)

| Document | Rôle |
|---|---|
| `SFD.md` | Spécification fonctionnelle détaillée v3.0 |
| `README.md` | Page d'accueil officielle Odysseus |
| `CONTRIBUTING.md` | Guide de contribution |
| `SECURITY.md` | Politique de sécurité |
| `THREAT_MODEL.md` | Modèle de menace complet |
| `ROADMAP.md` | Roadmap publique |
| `loop-canonique.md` | Spécification des 7 phases |
| `permission-matrix.md` | Matrice outils × phases |
| `ACKNOWLEDGMENTS.md` | Crédits open source |
| `LICENSE` | AGPL-3.0 |

## 📦 Archive

L'historique complet des analyses, inventaires, audits et plans passés est préservé dans [`docs/archive/`](archive/).

---

*→ Retour à l'[INDEX-MAITRE](../.planning/INDEX-MAITRE.md)*
