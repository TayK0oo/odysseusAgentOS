# ADR 0010 — Convergence Plugin : architecture "Everything is a Plugin"

**Date :** 2026-10-19
**Statut :** Accepté
**Contexte :** Convergence de 51 harnais disséqués (910 fonctionnalités) + 148 vidéos TikTok (3 comptes) + travail existant odysseusAgentOS.

## Décision

Adopter le modèle "Everything is a Plugin" comme architecture fondamentale du système AgentOS Omega. Toute brique logicielle est un plugin qui se branche sur un port. Le noyau ne connaît que les ports, jamais les implémentations.

## Contexte

L'analyse combinatoire de 51 harnais et 148 vidéos TikTok révèle 104 capacités distinctes réparties sur 12 domaines. L'architecture actuelle (omega) a 16 ports et 16 crates, mais 8 nouveaux ports sont proposés pour couvrir les capacités manquantes.

## Décision détaillée

1. **16 ports existants** sont conservés et enrichis avec de nouveaux adaptateurs
2. **8 nouveaux ports** sont créés :
   - `Registry` : registre de plugins (manifest, entry info, slot/ensemble, quarantaine)
   - `UiSurface` : surface UI (slots, thème par surface, projection DOM)
   - `ConfigSource` : source de configuration (layers + precedence, profiles, schema validation)
   - `SkillStore` : magasin de skills (discover/load, policy, experiences comme savoirs)
   - `AgentLoop` : boucle d'agent (chat/step/recover, pipeline d'étapes, repair loop)
   - `AgentStore` : magasin d'agents (discover/load/save, roster, presets)
   - `WorkflowEngine` : moteur de workflow (build/compile/run, route/send, checkpoint)
   - `SandboxBackend` : backend sandbox (WebContainer, E2B, namespace Linux, par branche)

3. **Chaque port a des adaptateurs** (implémentations interchangeables) qui sont des plugins
4. **Le registre de plugins** gère l'installation, l'activation, la désactivation et le rechargement atomique
5. **Le noyau (omega-core)** ne connaît que les ports, jamais les implémentations

## Conséquences

- **Positives :** Modularité totale, testabilité par port, remplacement sans casser, conformité "everything is a plugin"
- **Négatives :** Complexité accrue, overhead d'abstraction, courbe d'apprentissage
- **Risques :** Performance (indirection), maintenance des adaptateurs, compatibilité inter-ports

## Alternatives considérées

1. **Architecture monolithique** : Rejetée — contraire au principe V4 (modularité totale)
2. **Microservices purs** : Rejetée — overhead réseau, complexité déploiement
3. **Plugins sans ports** : Rejetée — pas de contrat formel, couplage implicite

## Références

- `docs/omega/reverse-eng/CONVERGENCE-TOUT.md` — document de convergence complet
- `docs/omega/reverse-eng/OMEGA-COMPLET.md` — corpus combiné (51 harnais, 910 fonctionnalités)
- `docs/master-ref/10-ARCHITECTURE-MODULAIRE.md` — analyse combinatoire odysseusAgentOS
- `docs/master-ref/01-SFD-v3.1.md` — spécification fonctionnelle de référence
