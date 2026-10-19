# ADR-0003 — Architecture « everything is a plugin » : capacités, ports, manifestes

- **Statut :** accepté
- **Date :** 2026-10-15
- **Provenance :** `[stated]` (axe demandé) + `[observed]` (patterns du RE des harnais)

## Contexte

L'axe unique demandé : chaque brique doit être interchangeable, configurable et
remplaçable intelligemment (SOLID, architecture « pieuvre »/hexagonale, microservices là
où justifié). Le RE des harnais (OpenClaw, Hermes, Goose, Codex, OpenCode) montre deux
patterns dominants : **manifeste + registre** et **slot vs ensemble**.

## Décision

Adopter un modèle unifié :

1. **Capacité** — besoin nommé du système (`llm-provider`, `context-engine`, `memory-store`,
   `tool`, `sandbox-backend`, `router-policy`, `catalog-source`, `ui-panel`…).
2. **Port** — un trait Rust (noyau) + une interface TS (UI) par capacité.
3. **Adaptateur** — une implémentation d'un port (natif Rust, WASM, ou service distant).
4. **Manifeste** — descripteur déclaratif, schema-validé (id, capacité, version, config
   schema, permissions, entrypoint).
5. **Registre** — découverte dynamique + résolution. **Deux régimes** :
   - *slot exclusif* (une implémentation active : `context-engine`, `memory-store`) ;
   - *ensemble concurrent* (N actives : `llm-provider`, `tool`).
6. **Policy par code** — le noyau applique les permissions déclarées ; un plugin non
   fiable ne peut pas court-circuiter les credentials ni escalader les privilèges.

## Conséquences

- ✅ Étendre = **ajouter** un plugin, jamais **modifier** le noyau (OCP).
- ✅ Le noyau ne dépend que d'abstractions (DIP) ; adaptateurs substituables (LSP).
- ✅ Ports fins et spécialisés (ISP) ; une responsabilité par plugin (SRP).
- ⚠️ Coût d'un contrat stable et versionné par capacité (à ne pas multiplier sans raison).

## Interdits associés (garde-fous CI)

- ❌ Nouvelle dépendance `runtime → adaptateur`. ❌ Import en dur d'un fournisseur.
- ❌ Un « port-dieu » qui regroupe plusieurs capacités.

## Références

- `docs/omega/00-VISION-OMEGA.md` §3-4.
- `docs/omega/reverse-eng/` (patterns observés).
