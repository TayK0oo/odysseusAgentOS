# Politique de modèles — RÈGLE GLOBALE (toutes conversations, tous espaces de travail)

> **Autorité :** cette règle s'applique à **tout** agent, sous-agent, commande et
> session OpenCode, dans **tous** les projets. Elle prime sur les préférences
> locales. Machine : `docs/omega/rules/model-policy.json` · vérification :
> `omega/scripts/model_policy.py` · veille : `docs/omega/rules/MODEL-WATCH.md`.

## 0. La règle en une phrase

**On lance les sous-agents sur des modèles GRATUITS. On ne passe sur un modèle
payant qu'en exception justifiée, et uniquement parmi ceux inclus dans
l'abonnement Go (`opencode-go/*`).**

## 1. Pourquoi

L'abonnement **OpenCode Go (10 $/mois)** est **plafonné** : 20 % du quota mensuel
par fenêtre de 5 h, 50 % par semaine, 100 % par mois, **par modèle**. Un
sous-agent lancé sur un modèle cher peut consommer le quota de la session
principale et bloquer le travail en cours. Les modèles **gratuits** restent
utilisables même quand le quota est épuisé.

## 2. Les deux pièges à éviter (constatés dans notre propre configuration)

1. **`opencode/<modèle>` payant = facturé au solde (Zen), PAS inclus dans Go.**
   Seuls les `opencode/*-free` sont gratuits côté Zen. Toute exception payante
   doit viser **`opencode-go/*`**.
2. **Les modèles gratuits changent** (« limited time ») : un modèle cité dans un
   document de veille peut avoir disparu. **Toujours vérifier sur la liste live**
   (`omega/scripts/model_policy.py --list-free`) avant de s'y fier.

## 3. Tier 1 — GRATUIT (défaut absolu)

| Usage type | Premier choix | Repli | Données |
|---|---|---|---|
| Sous-agent léger (recherche, extraction, édition courte, tâches répétitives) | `opencode/mimo-v2.6-flash-free` | `opencode/glm-5.3-flash-free` | amélioration ⚠ |
| Sous-agent code (écriture, dissection, refactor local) | `opencode/space-bunny-free` | `opencode/deepseek-v4.1-flash-free` | **0 j** |
| Raisonnement / analyse / planification | `opencode/deepseek-v4.1-flash-free` | `opencode/space-bunny-free` | **0 j** |
| Contexte énorme (1 M) | `opencode/deepseek-v4.1-flash-free` | `opencode/space-bunny-free`, `opencode/longcat-2.5-preview-free` | 0 j |
| Multimodal (image en entrée) | `opencode/space-bunny-free` | `opencode/longcat-2.5-preview-free` | 0 j |
| **Code confidentiel** | `opencode/space-bunny-free` | `opencode/deepseek-v4.1-flash-free`, `opencode/longcat-2.5-preview-free` | **0 j exigé** |
| Repli général (si indisponible) | `opencode-go/beam-free` | `opencode-go/longcat-2.5-preview-free`, `opencode/beam-free` | beam : 30 j |

**À ne pas utiliser pour du code confidentiel** (les données servent à
l'amélioration ou sont journalisées) : `opencode/mimo-v2.6-flash-free`
(amélioration), `opencode/nemotron-3.5-super-vl-free` (essai NVIDIA),
`opencode/beam-free` (30 j).

## 4. Tier 2 — PAYANT, exception justifiée (`opencode-go/*` uniquement)

À n'utiliser que si **un modèle gratuit a échoué** sur la tâche (échec constaté,
pas supposé), et en **documentant la raison** dans le message de délégation.

| Usage type | Modèle | Quota estimé | Pourquoi |
|---|---|---|---|
| Défaut payant (meilleur rapport qualité/quota) | `opencode-go/gpt-6-luna` | 21 130 req/mois | polyvalent, 1,05 M ctx, DeepSWE 66,6 |
| Gros volume payant | `opencode-go/deepseek-v4.1-flash` | 173 330 req/mois | le plus de requêtes, 1 M ctx |
| Tâche **très** difficile / gros refactoring | `opencode-go/glm-5.3` → `opencode-go/kimi-k3` | 1 080 / 490 | TB 88,2 / DeepSWE 67,5 — **rare** |
| Sous-agents payants (si gratuit insuffisant) | `opencode-go/glm-5.3-flash`, `opencode-go/qwen3.8-flash`, `opencode-go/claude-haiku-5-5` | 31 580 / 27 000 / 19 230 | palier rapide, bon quota |
| Migration massive de code | `opencode-go/gemini-4-argon-contributor` | 2 120 | ⚠ **entraîne sur tes données** |
| Multimodal payant | `opencode-go/step-5-preview`, `opencode-go/minimax-m3` | 8 520 / 16 000 | images/vidéo |

**Ordre de préférence à quota égal :** privilégier le `req/mois` le plus élevé.

## 5. Interdits

- ❌ **Modèles `opencode/*` payants** (Zen = solde hors Go) : `opencode/deepseek-v4-pro`,
  `opencode/minimax-m3`, `opencode/gpt-5.5`, `opencode/grok-4.7`, `opencode/gpt-6-astra`, etc.
- ❌ **`opencode/deepseek-v4-pro`** : retiré chez DeepSeek (redirigé vers V4.1-Flash) — doublon mort.
- ❌ Lancer un sous-agent sur un modèle payant **sans** échec gratuit préalable constaté.
- ❌ Se fier à une liste de modèles recopiée d'un document sans vérification live.
- ❌ Modèle « entraîne » (`muse-spark-*`, `gemini-4-argon-contributor`) pour du code confidentiel.

## 6. Application automatique (déjà en place)

| Niveau | Fichier | Effet |
|---|---|---|
| Global (toutes conversations) | `~/.config/opencode/opencode.json` → `agents.general.model`, `agents.explore.model` | les sous-agents intégrés naissent sur un modèle gratuit |
| Projet | `.opencode/opencode.json` → `agents.*.model` | idem, plus spécifique au projet |
| Agents nommés | `.opencode/agents/<nom>.md` → frontmatter `model:` | chaque agent SFD est épinglé sur un modèle gratuit |
| Règle de session | `.opencode/rules/model-policy.md` | rappel lu par l'agent à chaque session |

**Contrat de délégation** (à respecter par l'orchestrateur) : dans chaque appel
`subagent`, soit le modèle est gratuit, soit le message **cite l'échec gratuit
constaté** qui justifie le passage au payant.

## 7. Vérification

```bash
python3 omega/scripts/model_policy.py --check       # tous les modèles cités existent-ils ?
python3 omega/scripts/model_policy.py --list-free   # liste live des gratuits
python3 omega/scripts/model_policy.py --for subagent-code   # quel modèle utiliser ?
```

## 8. Révision

- Les gratuits sont **temporaires** : revérifier **au moins tous les 15 jours**
  (`--check` le signale).
- L'accord « zéro rétention » DeepSeek est renouvelé mensuellement (fin le 31/10/2026).
- Sources : `opencode-go_classement_2026-10-17.md`,
  `opencode-gratuits_classement_2026-10-17.md`, `opencode_sources_2026-10-17.md`
  (dans `~/Téléchargements/`), + liste live Zen/Go.
