# Classement des modèles OpenCode Go

- **Date de mise à jour : 2026-10-17** (version 2)
- **Prochaine MAJ suggérée : 2026-11-01** (l'accord de zéro rétention DeepSeek expire le 31 octobre ; les offres « limited time » changent vite)
- Fichiers liés : `opencode-gratuits_classement_2026-10-17.md`, `opencode_sources_2026-10-17.md`

## Le plan

| Plan | Prix | Remarques |
|---|---|---|
| Go | 10 $/mois | Espace de travail à un seul membre. |
| Go Plus | 40 $/mois | Mêmes modèles, limites plus hautes. |

- Limites par modèle : 20 % du plafond mensuel par fenêtre de 5 h, 50 % par semaine, 100 % par mois.
- Quand la limite est atteinte : les modèles gratuits restent utilisables ; l'option « Use balance » permet de basculer sur le solde pay-as-you-go.
- Liste officielle : **29 modèles** = 27 payants + 2 gratuits temporaires (en bas de page).

## Légende

- **✓** = benchmark ou retour indépendant / tiers.
- **É** = chiffre publié par l'éditeur du modèle (non vérifié indépendamment).
- **?** = non évalué ou déduit de la fiche produit. À tester soi-même.
- **Req/mois** = estimation officielle pour le plan Go à 10 $ (varie selon l'usage réel).
- **Données** : « 0 j » = zéro rétention, pas d'entraînement. « 30 j » = rétention 30 jours, pas d'entraînement. « Entraîne » = tes prompts servent à entraîner le modèle.

## Méthode du classement global

Pondération : qualité 50 %, quota de requêtes 25 %, vie privée 15 %, maturité et disponibilité 10 %. Le rang est un jugement de synthèse, pas une mesure officielle. Beaucoup de chiffres viennent des éditeurs.

## Classement des 27 modèles payants

| # | Modèle | Usage type / points forts | Qualité (preuves) | Req/mois | $ in / out par M | Données |
|---|---|---|---|---|---|---|
| 1 | GPT 6 Luna | Modèle par défaut polyvalent : tâches agentiques difficiles à faible coût, très long contexte (1,05 M), gros volume de requêtes. | ✓ DeepSWE 66,6 | 21 130 | 0,10 / 0,50 (≤272K) | 30 j |
| 2 | DeepSeek V4.1 Flash | Agents de code longue durée et gros volume : tâches routinières d'agent, grosses bases de code (1 M de contexte, KV cache compact), sous-agents. Plus faible en raisonnement pur (HLE 36,8 contre 56,3 pour Opus 5). | É DeepSWE 74,2 (devant Kimi K3 selon l'éditeur, non vérifié) | 173 330 | 0,15 / 0,60 (creux) ; 0,30 / 1,20 (pointe) | 0 j* |
| 3 | GLM-5.3 | Longues sessions terminal et agent : devant Kimi K3 sur Terminal-Bench 3.0 (28,3 contre 17,4). Remplaçant du flagship GLM pour agents de code. | ✓ TB 2.1 88,2 ; DeepSWE 66,9 | 1 080 | 1,40 / 4,40 | 0 j |
| 4 | Kimi K3 | Ingénierie logicielle lourde, travail de connaissance, raisonnement profond. Très fort sur SWE-Marathon (48,1), mais plus faible que GLM-5.3 sur les terminaux très longs. À garder pour les tâches les plus dures. | ✓ TB 2.1 88,3 ; DeepSWE 67,5 ; Verified 93,4 (Vals) | 490 | 3,00 / 15,00 | 0 j |
| 5 | MiniMax M3 | « Cheval de trait » agentique du quotidien : codage de niveau frontier, multimodal natif, 1 M de contexte, bon volume. | É Verified 80,5 | 16 000 | 0,30 / 1,20 | 0 j |
| 6 | MiMo-V2.6-Pro | Codage agentique et terminal économique : devant Muse Spark 1.3 de plus de 20 points sur Terminal-Bench 4.0 (Vals). | ✓ coding Vals 39,9 % | 16 300 | 0,435 / 0,87 | 0 j |
| 7 | Qwen3.8 Max | Génie logiciel, travail bureautique, raisonnement profond (2,4 T de paramètres). Bon sur les tâches type issue GitHub (SWE-bench Pro), moins sur le long horizon (DeepSWE plus bas). | É SWE-bench Pro 67,7 ; TB 2.1 86,6 ; ✓ DeepSWE 56,6 | 810 | 2,00 / 6,00 | 0 j |
| 8 | GLM-5.2 | Ancien flagship GLM pour agents de code, bon compromis qualité/quota. | É SWE-bench Pro 62,1 | 4 300 | 1,40 / 4,40 | 0 j |
| 9 | DeepSeek V4 Pro | Raisonnement complexe, génération de code, connaissance générale (1,6 T / 49 B actifs, 1 M de contexte). Attention : DeepSeek a redirigé V4-Pro vers V4.1-Flash sur sa propre API depuis le 14 septembre ; ce que sert exactement OpenCode est à vérifier. | ✓ TB 2.1 87,9 ; DeepSWE 62,7 ; SWE-bench Pro 55,4 | 5 200 | 0,66 / 1,98 (creux) ; 1,32 / 3,96 (pointe) | 0 j* |
| 10 | Gemini 4 Argon Contributor | Migrations de code massives (C/C++ vers Rust sur plus de 800 000 lignes selon Google), refactoring long horizon, finance, droit, cybersécurité défensive, faible hallucination, 1 M de tokens en sortie. Limité par le quota et par l'entraînement sur tes données. | É DeepSWE 77,9 ; TB 4.0 57,4 ; Vals Index n° 1 (68,9) | 2 120 (×2 temporaire) | 4,00 / 20,00 | Entraîne (Google) |
| 11 | Grok 4.7 | Tâches difficiles ponctuelles ; le quota se consomme très vite. Modèle par défaut de Grok Build. | ✓ TB 4.0 38 % ; TB 2.1 Vals 73,4 | 845 | 2,00 / 6,00 (≤200K) | 30 j |
| 12 | Step 5 Preview | Génie logiciel, finance, analyse de longs documents, multimodal (images et vidéo), 1 M de contexte. | É DeepSWE 67,7 ; ✓ Kingbench 83,75 % dans OpenCode (GLM-5.3 : 91,25 %) | 8 520 (×4 temporaire) | 1,00 / 2,70 | 0 j |
| 13 | MiMo-V2.5-Pro | Modèle agent phare de Xiaomi (1 T / 42 B actifs, 1 M de contexte), codage correct à petit prix. | ✓ Index Artificial Analysis 54 (avril, au niveau du top open) | 16 300 | 0,435 / 0,87 | 0 j |
| 14 | Kimi K2.7 Code | Variante Kimi réglée pour le code, bien plus de requêtes que K3 (6 750 contre 490). | ? non évalué | 6 750 | 0,95 / 4,00 | 0 j |
| 15 | Qwen3.7 Plus | Palier intermédiaire Qwen : codage généraliste à volume correct (tarif plus haut au-delà de 256K). | ? (famille 3.7 Max : Verified 80,4) | 21 600 | 0,40 / 1,60 | 0 j |
| 16 | Space Bunny | Modèle anonyme rapide, codage annoncé fort, multimodal (texte, image, vidéo), 1 M de contexte. N° 1 en usage sur OpenCode fin septembre. Identité non confirmée (soupçon MiniMax M3.1, non prouvé). | ? aucun benchmark | 15 630 | 0,15 / 0,60 | 0 j |
| 17 | GLM-5.3-Flash | Codage bon marché à gros volume (MoE 320 B / 18 B, licence MIT, 1 M de contexte). | É TB 2.1 84,3 dans Claude Code | 31 580 | 0,15 / 0,50 | 0 j |
| 18 | Hy4 preview | Agents de codage longue durée, très long contexte (1 M), prototypes de jeux, tâches bureautiques et finance. Les tests tiers conseillent un pilote contrôlé plutôt qu'un défaut de production (modèle en « preview »). | É DeepSWE 64,3 ; évaluation interne experts 2,99/4 (GLM-5.3 : 2,92 ; K3 : 2,94) | 6 770 | 0,834 / 2,501 | 0 j |
| 19 | GPT 5.6 Luna | **Redondant** : génération précédente, plus cher et moins de requêtes que GPT 6 Luna. | ✓ SWE-bench Pro 62,7 ; TB 2.0 84,7 | 10 250 | 0,20 / 1,20 | 30 j |
| 20 | LongCat-2.0 | Tâches agentiques et terminal, long contexte (1 M), très gros volume. | ✓ BenchLM : agentique 70,8 ; coding 59,5 | 57 200 | 0,30 / 1,20 | 0 j |
| 21 | Qwen3.8 Flash | Palier rapide et bon marché de Qwen : sous-agents, tâches simples. | ? non évalué | 27 000 | 0,15 / 0,47 | 0 j |
| 22 | MiMo-V2.6-Flash | Gros volume, code léger et agents bon marché. | ✓ coding Vals 38,5 % | 150 400 | 0,14 / 0,28 | 0 j |
| 23 | Claude Haiku 5.5 | Petit modèle rapide : sous-agents, tâches simples, titres de session (OpenCode utilise des modèles de ce palier pour les titres). Tarif plus haut au-delà de 100K de contexte. | ? aucun benchmark trouvé pour la 5.5 | 19 230 | 0,10 / 0,50 (≤100K) | 30 j |
| 24 | MiMo-V2.5 | Génération précédente de MiMo, gros volume, tâches simples. | ? | 150 400 | 0,14 / 0,28 | 0 j |
| 25 | Hy3 | Palier bas de Tencent, bon marché. | ? non évalué | 21 500 | 0,14 / 0,58 | 0 j |
| 26 | Muse Spark 1.3 Contributor | Très gros volume ; plutôt agents finance et droit (Harvey Legal 22,9 % contre 10,8 % pour MiMo-V2.6-Pro). Codage plus faible. Régions limitées. | ✓ coding Vals 33,1 % | 226 600 | 0,10 / 0,20 | Entraîne (Meta) |
| 27 | Muse Spark 1.2 Contributor | Idem, génération précédente. | ? | 226 600 | 0,10 / 0,20 | Entraîne (Meta) |

\* DeepSeek : l'accord de zéro rétention est renouvelé chaque mois ; l'accord actuel court jusqu'au 31 octobre 2026.

### Les 2 modèles gratuits inclus dans Go

| Modèle | Usage type | Limites | Données |
|---|---|---|---|
| LongCat 2.5 Preview Free | Test gratuit d'un modèle raisonnement avec 1 M de contexte, texte et image, appels d'outils. Pas de benchmark code trouvé. | Illimité, temporaire | 0 j |
| Beam Free | Non documenté. Tâches légères ou essai uniquement. | Illimité, temporaire, indisponible dans une longue liste de pays | 30 j |

## Quel modèle pour quel usage ?

| Besoin | Modèles à essayer en premier |
|---|---|
| Défaut du quotidien | GPT 6 Luna, DeepSeek V4.1 Flash, MiniMax M3 |
| Tâche très difficile, gros refactoring | GLM-5.3, Kimi K3, Qwen3.8 Max |
| Sous-agents, tâches simples et répétitives | DeepSeek V4.1 Flash, MiMo-V2.6-Flash, GLM-5.3-Flash, Qwen3.8 Flash, Haiku 5.5 |
| Grosse base de code (1 M de contexte) | DeepSeek V4.x, MiniMax M3, GPT 6 Luna, Step 5, LongCat-2.0, Space Bunny |
| Images ou vidéo en entrée | Step 5 Preview, Space Bunny, MiniMax M3, DeepSeek V4.1 Flash (images) |
| Finance, droit, travail de connaissance | Gemini 4 Argon Contributor, Step 5 Preview, Muse Spark |
| Migration massive de code | Gemini 4 Argon Contributor |
| Code confidentiel | tout modèle « 0 j » ; éviter Muse Spark et Gemini 4 Argon Contributor |

## Changements par rapport à la première version (réponse du 2026-10-17)

| Modèle | Avant | Maintenant | Raison |
|---|---|---|---|
| DeepSeek V4.1 Flash | 14 | 2 | Chiffres éditeur très forts (DeepSWE 74,2) pour 173 330 req/mois ; reste à confirmer indépendamment. |
| DeepSeek V4 Pro | 2 | 9 | DeepSeek l'a retiré de son API au profit de V4.1 Flash. |
| Gemini 4 Argon Contributor | 27 | 10 | Chiffres éditeur parmi les plus hauts (DeepSWE 77,9), freiné par le quota et l'entraînement. |
| Step 5 Preview | 24 | 12 | DeepSWE 67,7 (éditeur) et test Kingbench dans OpenCode. |
| Space Bunny | 23 | 16 | Très fort usage réel, 1 M de contexte, multimodal. |
| Hy4 preview | 21 | 18 | Chiffres éditeur corrects, mais statut « preview » à piloter. |

## Points à vérifier

- Ce que sert exactement OpenCode sous « DeepSeek V4 Pro » depuis la redirection côté DeepSeek.
- Les chiffres éditeur (DeepSeek V4.1 Flash, Gemini 4 Argon, Step 5, Hy4) n'ont pas de confirmation indépendante.
- Step 5 Preview : date de sortie contradictoire selon les sources (18 septembre ou 8 octobre).
- Des utilisateurs ont signalé en juillet et août des erreurs « solde insuffisant » ou 401 côté Go ; je n'ai pas vérifié si c'est résolu.

## Comment mettre à jour ce fichier

1. Relire la doc Go : https://opencode.ai/docs/go/ (liste, limites, tableau de confidentialité).
2. Lister les modèles en direct : `curl https://opencode.ai/inference/go/v1/models`.
3. Pour chaque nouveau modèle, chercher benchmarks (DeepSWE, Terminal-Bench, SWE-bench Pro) et usages, puis le marquer ✓, É ou ?.
4. Recalculer le rang avec la pondération ci-dessus, compléter la section « Changements », puis enregistrer un nouveau fichier daté `opencode-go_classement_AAAA-MM-JJ.md`.
5. Ajouter les nouvelles sources dans `opencode_sources_AAAA-MM-JJ.md`.
