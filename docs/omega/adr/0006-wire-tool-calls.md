# ADR-0006 — Transport des tool-calls dans le wire (découverte S1)

**Date** : 2026-10-19 · **Statut** : Accepté · **Décidé par** : chef de projet
**Déclencheur** : exécution S1 (banc P0-4) — `xfail_raison` démontré par sortie, pas par hypothèse.

## Découverte (observed)

S1 a révélé que la boucle d'outils (P1-1) est inconcevable sur le wire actuel :

- `WireMessage {role, content}` (`omega-providers/src/ollama.rs:22`) ne transporte
  **ni `tool_calls` ni résultats d'outil** ;
- le domaine `Message` (`omega-types/src/lib.rs:41`) n'a pas de champ outil ;
- conséquence : P1-1 = **deux travaux empilés** — (a) étendre le schéma wire,
  (b) brancher la boucle. Estimation passe de `L` à `XL` → règle 11 du plan
  (« au-delà de ×2 sans porte verte : ADR ») déclenchée par ce document.

## Décision

**A + C ensemble, une seule décision** (P1-5 du plan le prévoit déjà :
« appel natif *ou* repli ReAct selon la capacité déclarée du modèle ») :

1. **A — extension de schéma** : `WireMessage` gagne
   `tool_calls: Option<Vec<ToolCall>>` et le rôle `"tool"` est déjà mappé
   (`role_to_wire`, ollama.rs:57). Champ **optionnel** → compatible ;
   le contrat `omega-protocol` prend un bump mineur, rétrocompatible.
2. **C — repli ReAct texte** : pour les modèles sans appel natif, le modèle
   produit un JSON texte canonique (`{"tool": "...", "arguments": {...}}`),
   parseur unique côté système. Le `ScriptedProvider` scriptera **les deux formes**
   (test du chemin natif ET du repli) — c'est une seule décision de surface.
3. **Rejeté — B (canal parallèle hors body)** : divergent des conventions
   Ollama/OpenAI, fragile, invisible pour l'écosystème.

## Conséquences

- Le `xfail_raison` de S1 passe de 2 à 3 causes (wire, boucle, séquence scriptée
  à deux formes) — amendement via cet ADR, scénario **reste xfail**.
- P1-1 est rescopé : (a) schéma + (b) boucle + (c) deux formes de scripting.
- Jusqu'au bump de schéma, aucun scénario ne peut tester un appel d'outil réel :
  S2+ (sous-agents) dépendent de P1-1 — consigné avant de planifier P1.
- Le pont vers le socle Python (P0-9/D0) doit poser la même question côté
  Python (`tool_calls` dans son protocole ?) — question ajoutée au spike.
