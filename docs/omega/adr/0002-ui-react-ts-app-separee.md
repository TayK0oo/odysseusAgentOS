# ADR-0002 — UI : nouvelle application React/TS isolée

- **Statut :** accepté
- **Date :** 2026-10-15
- **Provenance :** `[stated]` (choix utilisateur)

## Contexte

L'UI actuelle est un `static/` HTML/CSS/JS classique, fragile (positionnement, CSS
« île de Calypso », surcharges mobiles). La vision demande un **cockpit clair et intuitif**
(plugin explorer, router dashboard, trace viewer).

## Décision

Créer une **nouvelle application `omega/ui/`** (Vite + React + TypeScript), isolée,
consommée via une **API typée** exposée par `omega-server`. Le `static/` existant reste
tel quel (legacy) et n'est pas migré.

## Conséquences

- ✅ DX moderne, composants réutilisables, accessibilité, thème.
- ✅ Séparation nette UI ↔ logique (contrats générés depuis les schémas serveur).
- ⚠️ Deux UI coexistent temporairement dans le dépôt.
- ⚠️ Toute donnée affichée doit exister côté API (pas de logique métier dans l'UI).

## Alternatives écartées

- **Refonte incrémentale du `static/`** : coût de démantèlement, pas de gain structurel.
- **Maquette d'abord** : utile, mais ne remplace pas la décision d'architecture.

## Garde-fous

- L'UI ne parle **que** via l'API typée (aucun accès direct au système de fichiers/DB).
- Composants accessibles (cf. skill `magic-ui`) : focus, contrastes, mouvement réduit.
