# Sources des classements OpenCode (Go et gratuits)

- **Date de consultation : 2026-10-17**
- Fichiers liés : `opencode-go_classement_2026-10-17.md`, `opencode-gratuits_classement_2026-10-17.md`
- Les sources officielles priment pour les listes, prix, quotas et politiques de données. Les benchmarks viennent de sources tierces ou des éditeurs eux-mêmes.

## 1. Sources officielles OpenCode

| Source | Sert à |
|---|---|
| https://opencode.ai/v2/docs/console/go | Liste Go (29 modèles), prix du plan, limites, requêtes estimées, endpoints, confidentialité par modèle |
| https://opencode.ai/docs/zen/ | Liste Zen, prix, modèles gratuits, exceptions de confidentialité, modèles dépréciés |
| https://opencode.ai/inference/go/v1/models | Liste en direct des modèles Go (pour la mise à jour) |
| https://opencode.ai/zen/v1/models | Liste en direct des modèles Zen (pour la mise à jour) |
| https://thenewstack.io/?p=22840157 | Contexte OpenCode 2.0 |
| https://opencode.ai/data/compare/meituan/longcat-2-5-preview/nvidia/nemotron-3-5-lightning | Statistiques d'usage OpenCode (LongCat 2.5, Nemotron 3.5 Lightning) |
| https://opencode.ai/data/compare/meituan/longcat-2-5-preview/nvidia/nemotron-3-ultra | Statistiques d'usage OpenCode (Nemotron 3 Ultra) |

## 2. Benchmarks et comparatifs de modèles

| Source | Sert à |
|---|---|
| https://www.morphllm.com/best-open-source-coding-model-2026 | SWE-bench Verified (DeepSeek V4 Pro 80,6 ; MiniMax M3 80,5 ; Kimi K3 93,4 Vals) |
| https://www.morphllm.com/swe-bench-pro | SWE-bench Pro, Verified (septembre 2026) |
| https://huggingface.co/tsinghua-sigs-robot-lab/VeriLoop-E2 | Tableau comparatif TB 2.1, TB 3.0, DeepSWE, SWE-bench Pro (Kimi K3, GLM-5.3, Qwen3.8 Max, DeepSeek V4 Pro) |
| https://kingy.ai/blog/qwen3-8-max-vs-kimi-k3-vs-deepseek-v4-flash/ | DeepSWE et TB 2.1 (Qwen3.8 Max, Kimi K3, DeepSeek V4 Flash) |
| https://www.vals.ai/comparisons/meta_muse_spark_1_3-vs-xiaomi_mimo-v2.6-pro | Muse Spark 1.3 contre MiMo-V2.6-Pro (coding, Terminal-Bench 4.0, juridique) |
| https://www.vals.ai/comparisons/meta_muse_spark_1_3-vs-xiaomi_mimo-v2.6-flash | Muse Spark 1.3 contre MiMo-V2.6-Flash |
| https://benchlm.ai/compare/gpt-5-6-luna-vs-grok-4-5 | GPT-5.6 Luna et Grok 4.5 (SWE-bench Pro, TB 2.0) |
| https://benchlm.ai/compare/gpt-6-luna-vs-grok-4-6 | DeepSWE GPT 6 Luna (66,6) |
| https://benchlm.ai/md/compare/gpt-6-luna-vs-grok-4-7.md | Grok 4.7 TB 4.0 et TB 2.1 Vals |
| https://benchlm.ai/compare/longcat-2-0-vs-mimo-v2-5-pro | LongCat-2.0 (agentique 70,8 ; coding 59,5) |
| https://artificialanalysis.ai/ja/articles/recent-open-weights-model-launches | Index Artificial Analysis (Kimi K2.6, MiMo-V2.5-Pro, DeepSeek V4 Pro, avril 2026) |
| https://docs.qoder.com/qoder/model-selector.md | Positionnement officiel de Kimi K3, Qwen3.8 Max, MiniMax M3, DeepSeek V4 Pro |

## 3. Modèles documentés lors de la MAJ

| Modèle | Source | Sert à |
|---|---|---|
| Hy4 preview | https://www.tencent.com/tencent-releases-and-open-sources-ten | Annonce Tencent, évaluation interne 2,99/4 |
| | https://openrouter.ai/models/tencent/hy4-preview | Fiche, usages, prix, contexte 1 M |
| | https://wavect.io/blog/tencent-hy4-preview-coding-agent-review/ | Test, DeepSWE 64,3, recommandation de pilote |
| | https://datanorth.ai/news/tencent-releases-hy4-preview | Résumé et limites annoncées |
| Step 5 Preview | https://openrouter.ai/stepfun/step-5-preview/api | Fiche, usages, prix |
| | https://www.mindstudio.ai/blog/step-5-preview-stepfun-coding-test | Test Kingbench dans OpenCode (83,75 %) |
| | https://runtimewire.com/article/stepfun-step-5-preview-600b-agent-model-pricing | DeepSWE 67,7, StepCodeBench 49, ProgramBench 80,5 |
| | https://www.eesel.ai/blog/stepfun-step-5-review | Avis : parité avec les modèles ouverts, avantage de coût |
| | https://ai-tldr.dev/models/step-5-preview/ | Points forts, multimodal |
| Gemini 4 Argon | https://www.cometapi.com/models/google/gemini-4-argon/ | Tableau de benchmarks éditeur |
| | https://www.deeplearning.ai/the-batch/argon-a-low-hallucination-model-build-for-knowledge-work | Points forts, faible hallucination, limites en codage agentique |
| | https://propakistani.pk/2026/10/01/gemini-4-argon-beats-gpt-6-astra-at-40-the-cost/ | Migrations C/C++ vers Rust, Vals Index |
| | https://blog.google/innovation-and-ai/products/gemini-app/gemini-4-availability | Disponibilité (13 octobre) |
| DeepSeek V4.1 Flash | https://thenextweb.com/news/deepseek-v4-1-flash-launch-v4-pro-retired-price-cut | Retrait de V4-Pro, chiffres éditeur non vérifiés |
| | https://unifically.com/models/deepseek-4.1-flash | DeepSWE 74,2, usages |
| | https://openrouter.ai/deepseek/deepseek-v4.1-flash | Fiche, prix, contexte 1 M |
| | https://techstrong.ai/?p=68152 | HLE 36,8 contre 56,3 (Opus 5) |
| Space Bunny | https://www.theneuron.ai/blog/who-made-space-bunny-minimax-clue/ | Origine, politiques de données, Big Pickle |
| | https://openrouter.ai/stealth/space-bunny-alpha | Fiche : rapide, codage, multimodal, 1 M |
| | https://technews.tw/?p=1621958 | Usage réel sur OpenCode |
| | https://www.kucoin.com/news/flash/openrouter-launches-anonymous-model-space-bunny-suspected-to-be-minimax-m3-1 | Soupçon MiniMax M3.1 (non confirmé) |
| Nemotron 3.5 Lightning | https://www.together.ai/models/nemotron-3-5-lightning | Fiche : étapes d'agents à fort volume |
| Big Pickle | https://www.theneuron.ai/blog/who-made-space-bunny-minimax-clue/ | Contexte 200K, sortie 32K, ancienneté |

## 4. Autres sources utiles (contexte, quotas, avis)

| Source | Sert à |
|---|---|
| https://www.bitdoze.com/opencode-go-plan/ | Avis d'usage sur plusieurs modèles Go |
| https://codingplan.org/en/plans/opencode-go | Comparaison de plans (liste de 32 modèles non officielle) |
| https://docs.docker.com/ai/docker-agent/providers/opencode-zen/ | Liste de modèles gratuits Zen (période antérieure) |
| https://itsfree.ai/provider/opencode-zen/ | Liste de gratuits Zen (période antérieure) |
| https://github.com/anomalyco/opencode/issues/28129 | Incident « solde insuffisant » (mai) |
| https://github.com/anomalyco/opencode/issues/38257 | Incident 401 (juillet) |
| https://github.com/anomalyco/opencode/issues/27598 | Erreurs DeepSeek via Go |
| https://github.com/anomalyco/opencode/issues/36506 | Modèles payants Zen en échec, gratuits fonctionnels |

## 5. Réserves

- Les sections 1 et 3 ont été relues le 2026-10-17 ; les pages tierces peuvent changer sans préavis.
- Certains chiffres de la section 2 sont publiés par les éditeurs eux-mêmes et ne sont pas mesurés de façon comparable entre modèles.
- Plusieurs sources tierces datent de plusieurs semaines : revérifier avant de décider d'un abonnement.

## 6. Procédure de mise à jour des sources

1. Copier ce fichier vers `opencode_sources_AAAA-MM-JJ.md` avec la nouvelle date.
2. Revérifier les liens de la section 1, qui sont la référence.
3. Ajouter chaque nouvelle source avec ce qu'elle a servi à établir.
4. Noter les sources qui ne répondent plus ou qui se contredisent.
