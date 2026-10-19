# Règle globale — modèles (sous-agents)

> Référence complète : `docs/omega/rules/MODEL-POLICY.md` · machine :
> `docs/omega/rules/model-policy.json` · vérification : `omega/scripts/model_policy.py`.

## La règle

**Tout sous-agent part sur un modèle GRATUIT.** Le payant est une **exception
justifiée**, uniquement parmi `opencode-go/*` (inclus dans l'abonnement Go), et
seulement après un **échec gratuit constaté** — cité dans le message de délégation.

## Gratuits vérifiés (17/10/2026)

| Usage | Modèle |
|---|---|
| Sous-agent léger, volume | `opencode/nemotron-3.5-lightning-free`, `opencode/mimo-v2.6-flash-free` |
| Écriture de code | `opencode/space-bunny-free` |
| Analyse / plan | `opencode/deepseek-v4.1-flash-free` |
| Multimodal, 1 M de contexte | `opencode/space-bunny-free`, `opencode/longcat-2.5-preview-free` |
| Réponses typées (oui/non, score) | `opencode/jev-1.13-free` |
| Repli | `opencode/glm-5.3-flash-free`, `opencode/beam-free`, `opencode-go/beam-free`, `opencode-go/longcat-2.5-preview-free` |

## Interdits

- ❌ Les modèles `opencode/*` **payants** (Zen = solde, hors abonnement Go).
- ❌ `opencode/deepseek-v4-pro` (retiré chez DeepSeek).
- ❌ Un sous-agent payant **sans** échec gratuit constaté préalable.
- ❌ Modèles « entraîne » pour du **code confidentiel** (`muse-spark-*`,
  `gemini-4-argon-contributor`) ; éviter aussi `mimo-v2.6-flash-free`,
  `nemotron-*` (journalisés), `beam-free` (30 j).

## Quota Go (à ne pas épuiser)

20 % du quota mensuel par fenêtre de **5 h**, 50 % par **semaine**, 100 % par
**mois**, **par modèle**. Les gratuits restent utilisables quand le quota tombe.

## Application

`~/.config/opencode/opencode.json` et `.opencode/opencode.json` épinglent déjà
`general`, `explore`, `executor`, `reviewer`, `planner`, `auto-evolve`,
`sfd-orchestrator` sur des modèles gratuits. Vérifier avant tout changement :

```bash
python3 omega/scripts/model_policy.py --check
python3 omega/scripts/model_policy.py --for subagent-code
```
