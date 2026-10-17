# Classement des modèles gratuits OpenCode (Zen)

- **Date de mise à jour : 2026-10-17** (version 2)
- **Prochaine MAJ suggérée : 2026-10-31** (tous les gratuits sont « limited time » et changent souvent)
- Fichiers liés : `opencode-go_classement_2026-10-17.md`, `opencode_sources_2026-10-17.md`

## Accès

- Compte OpenCode Zen avec clé API, sans carte pour les gratuits.
- Endpoint : `https://opencode.ai/zen/v1/chat/completions` (liste en direct : `https://opencode.ai/zen/v1/models`).
- Dans OpenCode : `/connect`, choisir OpenCode Zen, puis `/models`. Identifiants au format `opencode/<id>`.
- Les 2 gratuits inclus dans Go (LongCat 2.5 Preview Free, Beam Free) sont listés dans le fichier Go.

## Légende

- **✓** = benchmark ou retour indépendant / tiers. **É** = chiffre de l'éditeur. **?** = non évalué ou déduit.
- **Données** : « 0 j » = zéro rétention sans entraînement. « Amélioration » = les données collectées pendant la période gratuite peuvent servir à améliorer le modèle. « Entraîne » = entraînement sur tes prompts.
- Le classement est estimatif : très peu de ces modèles ont des benchmarks indépendants.

## Classement des 12 modèles de chat gratuits

| # | Modèle (identifiant Zen) | Usage type / points forts | Preuves | Données | Confiance |
|---|---|---|---|---|---|
| 1 | Step 5 Preview Free (`step-5-preview-free`) | Génie logiciel et finance, longs documents, multimodal (images et vidéo), 1 M de contexte. | É DeepSWE 67,7 ; ✓ Kingbench 83,75 % dans OpenCode (au-dessus de GLM-5.3-Flash et des MiMo de référence) | 0 j | moyenne |
| 2 | Space Bunny Free (`space-bunny-free`) | Modèle anonyme rapide, codage annoncé fort, multimodal, 1 M de contexte. Très utilisé sur OpenCode depuis son lancement. Identité non confirmée. | ? aucun benchmark | 0 j | faible |
| 3 | MiMo-V2.6-Flash Free (`mimo-v2.6-flash-free`) | Codage et agents légers à gros volume ; devant Muse Spark 1.3 sur Terminal-Bench 4.0 selon Vals. | ✓ coding Vals 38,5 % | Amélioration | moyenne |
| 4 | Nemotron 3 Ultra Free (`nemotron-3-ultra-free`) | Gros modèle de raisonnement NVIDIA (550 B). Très utilisé (environ 2,5 % des tokens sur OpenCode selon les stats publiques). Endpoint d'essai NVIDIA : pas de données confidentielles. | ? pas de benchmark code | Logué par NVIDIA | faible |
| 5 | Muse Spark 1.3 Contributor Free (`muse-spark-1.3-contributor-free`) | Agents finance et droit, tâches parallélisables ; codage plus faible que MiMo. | ✓ coding Vals 33,1 % | Entraîne (Meta) | moyenne |
| 6 | LongCat 2.5 Preview Free (`longcat-2.5-preview-free`) | Raisonnement, appels d'outils, texte et image, 1 M de contexte. Successeur de LongCat-2.0 (agentique 70,8, coding 59,5 sur BenchLM). | ? pas de benchmark pour la 2.5 | 0 j | faible |
| 7 | Big Pickle (`big-pickle`) | Modèle « stealth » ancien (au catalogue depuis octobre 2025), 200K de contexte, 32K de sortie. Tâches légères. | ? | Amélioration | faible |
| 8 | MiMo-V2.5 Free (`mimo-v2.5-free`) | Génération précédente de MiMo : tâches simples et gros volume. | ? | Amélioration | faible |
| 9 | Nemotron 3.5 Lightning Free (`nemotron-3.5-lightning-free`) | Étapes répétitives d'agents à fort volume, sous-agents rapides (30 B / 3 B actifs, distillé de Nemotron 3 Ultra, entraîné pour les harnais d'agents). | ? | Logué par NVIDIA | faible |
| 10 | Exo Free (`exo-free`) | Non documenté. À tester. | ? | Amélioration | très faible |
| 11 | Ling 3.1 Flash Free (`ling-3.1-flash-free`) | Non documenté. Palier « Flash » rapide, à tester. | ? | Amélioration | très faible |
| 12 | Ling 3.0 Flash Fin Free (`ling-3.0-flash-fin-free`) | Probablement orienté finance (déduit du nom « Fin »), pas du code. | ? | Amélioration | très faible |

Hors classement : **Jev 1.13 Free** (`jev-1.13-free`). Ce n'est pas un modèle de chat : il répond à des questions typées (oui/non, choix, score) et renvoie des valeurs et des probabilités exploitables par du code.

## Quel gratuit pour quel usage ?

| Besoin | Essayer en premier |
|---|---|
| Codage agentique sérieux sans payer | Step 5 Preview Free, Space Bunny Free, MiMo-V2.6-Flash Free |
| Code confidentiel | Step 5 Preview Free, Space Bunny Free, LongCat 2.5 Preview Free (seuls gratuits à zéro rétention) |
| Contexte énorme (1 M) | Step 5, Space Bunny, LongCat 2.5 |
| Images ou vidéo en entrée | Step 5, Space Bunny, LongCat 2.5 (images) |
| Sous-agents rapides | Nemotron 3.5 Lightning Free, MiMo-V2.6-Flash Free |
| Finance ou droit | Step 5, Muse Spark 1.3 Contributor Free |

## Changements par rapport à la première version

| Modèle | Avant | Maintenant | Raison |
|---|---|---|---|
| Step 5 Preview Free | 5 | 1 | DeepSWE 67,7 (éditeur), test Kingbench dans OpenCode, zéro rétention. |
| Space Bunny Free | 6 | 2 | Usage réel très élevé, codage et multimodal annoncés, zéro rétention. |
| MiMo-V2.6-Flash Free | 1 | 3 | Reste le mieux mesuré, mais les deux précédents ont de meilleurs signaux et une meilleure confidentialité. |
| Nemotron 3 Ultra Free | 3 | 4 | Inchangé en substance ; toujours aucun benchmark code. |
| Muse Spark 1.3 Contributor Free | 2 | 5 | Entraînement sur tes données, codage plus faible. |

## Points à vérifier

- Tous les gratuits sont temporaires et peuvent disparaître sans préavis.
- Space Bunny : sur OpenRouter le fournisseur peut conserver les prompts (sans entraîner) ; sur OpenCode la politique indiquée est zéro rétention. Les deux routes diffèrent.
- Les endpoints NVIDIA sont des essais : ne pas y envoyer de données personnelles ou confidentielles.

## Comment mettre à jour ce fichier

1. Relire https://opencode.ai/docs/zen/ (section « The free models » et « Privacy »).
2. Lister en direct : `curl https://opencode.ai/zen/v1/models` et repérer les identifiants se terminant par `-free` ainsi que `big-pickle`.
3. Pour les nouveaux venus, chercher benchmarks et usages, puis marquer ✓, É ou ?.
4. Mettre à jour le rang et la section « Changements », puis enregistrer un nouveau fichier daté `opencode-gratuits_classement_AAAA-MM-JJ.md`.
5. Ajouter les nouvelles sources dans `opencode_sources_AAAA-MM-JJ.md`.
