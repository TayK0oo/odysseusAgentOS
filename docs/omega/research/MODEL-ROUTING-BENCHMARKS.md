# MODEL-ROUTING-BENCHMARKS — État de l'art : notation de modèles & conception d'un routeur

> **Objet.** Documenter les sources de benchmarks fiables, définir un **score composite
> multi-critères**, une **taxonomie de types de tâches** et l'**architecture d'un routeur
> de modèles** pour AgentOS Omega (pilier « Routing intelligent »).
>
> **Date de rédaction / de relevé des chiffres :** 2026-10-15.
> **Méthode d'acquisition :** `webfetch` / `curl` sur les sites et API cités (la recherche
> web par moteur était indisponible — quota épuisé).
> **Conventions de provenance :** `[stated]` (affirmé par la source) · `[observed]`
> (vérifié dans notre relevé) · `[inferred]` (déduit) · `[todo]` (à faire).
> **Statut :** recherche amont — nourrit un futur ADR (`docs/omega/adr/`) et le port
> `routing`. Aucun code de production n'est concerné par ce document.

> ⚠️ **Avertissement de fiabilité.** Les classements communautaires sont utiles mais
> **bruités** : contamination des jeux de test dans les corpus d'entraînement,
> auto-déclaration par les éditeurs, différences de *scaffolding* (l'agent compte autant
> que le modèle), et saturation progressive des benchmarks anciens (MMLU, MATH…). Un score
> composite doit être **relatif** (rang/percentile), **daté**, et **jamais** utilisé seul
> pour une décision critique sans garde-fou de fallback.

---

## 1. Sources de benchmarks

### 1.1 Synthèse (vue composite)

| Source | Ce qu'elle mesure | Accès (URL / API) | MAJ | Fiabilité pour un composite |
|---|---|---|---|---|
| **Artificial Analysis** | Index d'« intelligence », coût/tâche, vitesse (tok/s), latence (TTFT), fenêtre de contexte, ~250 modèles | `https://artificialanalysis.ai/leaderboards/models` (+ API JSON sur clé) | Continue (quasi hebdo) | **Très haute** — agrège plusieurs evals + mesure d'infra réelle |
| **LMArena / Chatbot Arena** | Préférence humaine (Elo/Bradley-Terry) sur batailles à l'aveugle | `https://lmarena.ai/leaderboard`, `https://huggingface.co/spaces/lmarena-ai/chatbot-arena-leaderboard` | Continue | **Haute** pour l'« elicitation/qualité perçue » ; biais de style, peu corrélé au code |
| **HuggingFace Open LLM Leaderboard** (v2) | Évals standardisées de modèles **open-weights** (IF, raisonnement, math, knowledge) | `https://huggingface.co/spaces/open-llm-leaderboard/open_llm_leaderboard` ; jeux : `.../datasets/open-llm-leaderboard/results` | Par vagues (runs soumis) | **Moyenne-haute** — idéal pour classer les modèles self-hosted entre eux |
| **SWE-bench / SWE-bench Verified** | Résolution d'issues GitHub réelles (Python) ; Verified = 500 instances validées par humains | `https://www.swebench.com/`, sous-page `verified.html` ; paper `arXiv:2310.06770` | Continue (soumissions) | **Très haute** pour le code agentique ; variante *Bash Only* (mini-SWE-agent) pour comparer les LLM |
| **Terminal-Bench 4.0** | Tâches d'agent en **terminal sandboxé** (systèmes, scripts, ops) | `https://www.tbench.ai/leaderboard` ; code `github.com/harbor-framework/terminal-bench` | Continue | **Haute** pour l'agentique sys/ops ; scaffold très déterminant |
| **Aider Polyglot** | 225 exercices Exercism (C++, Go, Java, JS, Python, Rust) — édition de code + respect du format | `https://aider.chat/docs/leaderboards/` | Continu (soumissions) | **Haute** pour l'**édition** de code multi-langages ; coûts inclus |
| **LiveBench** | Contamination-free : questions récentes/mensuelles (raisonnement, math, code, data-analysis, IF) | `https://livebench.ai/` ; code `github.com/LiveBench/LiveBench` | **Mensuelle** (nouveau jeu chaque mois) | **Haute** car renouvelée → limite l'overfit |
| **GPQA / GPQA Diamond** | Raisonnement scientifique niveau doctorat (QCM, « Google-proof ») | `arXiv:2311.12022` ; suivi sur `epoch.ai/benchmarks`, AA | À chaque nouveau modèle | **Haute** et peu saturée ; sensible au formatage |
| **MMLU-Pro** | Connaissances académiques multi-domaines (10 options, plus dur que MMLU) | `arXiv:2406.01574` ; suivi HF / Epoch | À chaque modèle | **Moyenne-haute** ; commence à saturer en tête |
| **BigCodeBench** | Génération de code « réaliste » (appels de bibliothèques, tâches composites) | `https://bigcode-bench.github.io/` | Par vagues | **Moyenne-haute** pour la génération de code (≠ édition) |
| **Epoch AI — Benchmarking Hub** | Base **indépendante** : GPQA-D, MATH L5, FrontierMath, SWE-bench Verified, Mock AIME… (~90 benchmarks, ~410 modèles) | `https://epoch.ai/benchmarks` ; téléchargement CSV ; client `epochai` (Airtable) | Continue | **Très haute** — méthodo publique, intervalles de confiance, licence CC-BY |
| **lm-evaluation-harness** (EleutherAI) | Harnais standard pour **ré-exécuter soi-même** MMLU, GSM8K, HellaSwag, etc. | `https://github.com/EleutherAI/lm-evaluation-harness` | Continu | **Référence d'implémentation** — la clé pour ne pas dépendre des scores des autres |
| **HLE / SimpleQA / τ-bench / FrontierMath** | Frontière (raisonnement dur), factualité & hallucinations, outillage agentique, maths de recherche | Voir §8 | Variable | **Complémentaires** ; utiles par axe, pas comme socle |

### 1.2 Détail par source

- **Artificial Analysis** `[observed]` — Le tableau public liste, pour chaque modèle et
  chaque **effort de raisonnement** (low/medium/high/xhigh/max), un *Intelligence Index*,
  un *coût par tâche* (USD), une vitesse médiane (tok/s), un **TTFT**, une latence totale et
  la fenêtre de contexte. C'est aujourd'hui la source la plus directement exploitable par un
  routeur car elle mêle **qualité + performance d'infra + prix** dans une même vue. La page
  est du HTML statique lisible en `webfetch` `[observed]`. Une API JSON existe (clé requise)
  `[inferred]`.
  *Limite :* l'Index est une **moyenne pondérée propriétaire** → ne pas tenter de le
  recomposer à l'identique depuis ses composants.

- **LMArena** `[observed]` — Renvoie **403 aux robots** sur `/leaderboard` (protection
  anti-bot) mais la page est publique au navigateur ; le *Space* HuggingFace de la
  leaderboard reste accessible `[observed]`. Utile pour la **qualité conversationnelle
  perçue** ; faible signal pour le code/l'agentique.

- **HuggingFace Open LLM Leaderboard v2** `[observed]` — Le *Space* répond (200) et pilote
  des évals (IFEval, BBH, MATH L5, GPQA, MUSR, MMLU-Pro). C'est le bon juge **relatif** pour
  les modèles **open-weights** que l'on veut auto-héberger. Les résultats bruts sont
  publiés côté datasets : `open-llm-leaderboard/results` `[observed]`.

- **SWE-bench & Verified** `[observed]` — Verified = **500 instances** filtrées par des
  humains (en collaboration avec OpenAI) ; la vue **« Bash Only »** évalue *les LLM*
  eux-mêmes dans un environnement minimal (`mini-SWE-agent`, boucle ReAct, sans outils
  exotiques) → idéal pour comparer des modèles de manière « apples-to-apples ». La famille
  s'étend à *Multilingual* (9 langages), *Multimodal*, *Lite* et **ProgramBench**
  (mai 2026, création de logiciels from scratch) `[observed]`.

- **Terminal-Bench 4.0** `[observed]` — Benchmark d'agents en terminal (hébergé par
  Stanford / Harbor / Laude Institute). Point clé de fiabilité : le **résultat dépend
  fortement de l'agent** (ex. Claude Code, Codex), pas seulement du modèle → distinguer
  « score modèle » et « score harnais ».

- **Aider Polyglot** `[observed]` — Mesure deux choses : le **taux de réussite** (Pass@2)
  et le **respect du format d'édition** (« percent cases well formed »), plus le **coût
  total** et le **temps par cas**. Précieux pour l'édition ciblée et le coût réel.

- **LiveBench** `[observed]` — Jeu **renouvelé mensuellement** à partir d'actualités et de
  problèmes récents, expressément conçu pour **limiter la contamination**. Signal de
  raisonnement « frais ».

- **Epoch AI** `[observed]` — Hub indépendant : **90 benchmarks**, **410 modèles**,
  **ECI** (Epoch Capabilities Index, échelle latente unifiée). Met à disposition les
  définitions de tâches (Inspect) et un **CSV** + client Python, **licence CC-BY**.
  Avertit explicitement sur contamination et sur les écarts liés au prompt/température.

- **lm-evaluation-harness** `[observed]` — Hors ligne, ré-exécutable : la seule façon de
  produire des scores **maison**, comparables, sans dépendre de l'auto-déclaration.

---

## 2. Comment agréger : un score composite multi-critères

### 2.1 Principe

Un routeur ne doit pas ranger les modèles sur **un seul axe**. On distingue :

1. **Axes de qualité** (ce que le modèle sait faire) — moyenne pondérée de benchmarks,
   chacun normalisé en percentile.
2. **Axes d'efficience** (ce que ça coûte en temps/argent) — vitesse, latence, coût,
   fenêtre de contexte.
3. **Axes opérationnels** (est-ce utilisable, ici, maintenant) — outils/function-calling,
   multimodalité, taux de limites, disponibilité, licence.

Tout score est **relatif au registre** (percentile 0→1) pour être robuste aux échelles.

### 2.2 Normalisations proposées `[inferred]`

Pour un modèle `m` dans un registre `R` :

- **Qualité par axe** (sous-score 0→1). On mélange des benchmarks mesurant la même
  compétence, puis on prend le **rang percentile** dans `R` :
  ```
  Q_axis(m) = percentile_R( 0.5·S1(m) + 0.3·S2(m) + 0.2·S3(m) )
  ```
  avec, par axe :
  | Axe | Benchmarks amalgamés | Poids internes |
  |---|---|---|
  | `Q_reason` (raisonnement) | AA Intelligence Index, GPQA-Diamond, AIME/MATH L5, LiveBench (raisonnement) | 0.4 / 0.25 / 0.2 / 0.15 |
  | `Q_code` | SWE-bench Verified (Bash Only), Aider Polyglot, BigCodeBench, LiveCodeBench | 0.4 / 0.3 / 0.15 / 0.15 |
  | `Q_agent` | Terminal-Bench, SWE-bench Verified, τ-bench (outils) | 0.45 / 0.35 / 0.20 |
  | `Q_knowledge` | MMLU-Pro, GPQA-Diamond, SimpleQA (factualité) | 0.45 / 0.3 / 0.25 |
  | `Q_chat` | LMArena (classement de préférence), IFEval | 0.75 / 0.25 |
  | `Q_vision` | Benchmarks VLM (MMMU, etc.) via AA/HF | — |
  *(un benchmark absent = axe estimé par héritage de famille de modèles `[inferred]`, ou marqué `unknown`.)*

- **Vitesse** (tok/s médian) — saturation logique pour borner l'influence des valeurs
  extrêmes (les moteurs diffusion/SSM montent à >1500 tok/s `[observed]`) :
  ```
  V(m) = clamp( log(1 + tok_s(m)) / log(1 + 400), 0, 1 )
  ```
- **Latence** (TTFT) — d'autant meilleure qu'elle est faible :
  ```
  L(m) = 1 / (1 + ttft_s(m))            # ∈ (0,1]
  ```
- **Coût** — log-normalisé sur l'échelle du registre (évite qu'un modèle gratuit écrase
  tout) ; coût = moyenne pondérée entrée/sortie (p. ex. 1/3 entrée, 2/3 sortie) :
  ```
  C(m) = ( log(1+c_max) - log(1+c(m)) ) / ( log(1+c_max) - log(1+c_min) )
  ```
- **Contexte** — adéquation au besoin plutôt que valeur absolue :
  ```
  K(m, besoin) = clamp( ctx(m) / besoin, 0, 1 )        # 1 dès que ça suffit
  ```
- **Opérationnel** — booléens → score :
  ```
  O(m) = 0.4·[tools] + 0.2·[json/structured] + 0.2·[vision|audio selon besoin] + 0.2·[open_weights]
  ```
- **Fiabilité** `R(m)` — disponibilité observée (uptime glissant), taux de 429/5xx,
  nombre de providers redondants `[todo]` (à mesurer en production).

### 2.3 Formule du score composite, par type de tâche

Pour une tâche de type `t` et un modèle `m` :

```
Score(t, m) =  w_q(t)·Q_eff(t,m)
             + w_v(t)·V(m)
             + w_l(t)·L(m)
             + w_c(t)·C(m)
             + w_k(t)·K(m, besoin_ctx(t))
             + w_o(t)·O(m, capacités requises(t))
             + w_r(t)·R(m)
```
avec les poids `w(t)` ≥ 0 et `Σ w = 1`, et
```
Q_eff(t,m) = Σ_axis  α_axis(t) · Q_axis(m)      # mélange d'axes propre à la tâche
```

> **Intention directrice** `[stated]` : *tâches simples/rapides → favoriser la VITESSE
> (et le coût) ; tâches complexes → favoriser la RÉFLEXION (qualité, contexte).*
> C'est exactement ce que traduisent les `w(t)` ci-dessous.

### 2.4 Profils de pondération (par type de tâche) `[inferred]`

| Profil | `Q_eff` (axes, α) | w_q | w_v | w_l | w_c | w_k | w_o | w_r |
|---|---|---|---|---|---|---|---|---|
| **Triviale / complétion** | `Q_chat` | 0.15 | **0.45** | 0.15 | **0.20** | 0.05 | 0.00 | 0.00 |
| **Édition ciblée** | `Q_code` | 0.40 | 0.20 | 0.05 | 0.15 | 0.05 | **0.15** | 0.00 |
| **Refactor / multi-fichiers** | `Q_code`+`Q_reason` | **0.55** | 0.05 | 0.05 | 0.10 | 0.05 | 0.15 | 0.05 |
| **Raisonnement long / plan** | `Q_reason`+`Q_knowledge` | **0.70** | 0.05 | 0.05 | 0.10 | 0.05 | 0.00 | 0.05 |
| **Agentique (tools/terminal)** | `Q_agent` | **0.55** | 0.10 | 0.05 | 0.05 | 0.05 | **0.15** | 0.05 |
| **Recherche / synthèse** | `Q_knowledge`+`Q_reason` | 0.50 | 0.05 | 0.05 | 0.10 | **0.15** | 0.10 | 0.05 |
| **Vision** | `Q_vision` | **0.60** | 0.10 | 0.05 | 0.15 | 0.05 | 0.05 | 0.00 |
| **Transcription / audio** | `Q_audio` | 0.55 | 0.15 | 0.10 | 0.15 | 0.00 | 0.05 | 0.00 |
| **Embeddings** | (qualité MTEB) | 0.55 | 0.15 | 0.05 | 0.20 | 0.05 | 0.00 | 0.00 |
| **Génération média** | (qualité FID/humaine) | 0.60 | 0.10 | 0.05 | 0.20 | 0.00 | 0.05 | 0.00 |
| **Chat généraliste** | `Q_chat`+`Q_knowledge` | 0.40 | 0.20 | 0.10 | 0.15 | 0.05 | 0.05 | 0.05 |

### 2.5 Efficience et Pareto (altitude du choix)

- **Frontière de Pareto** : on filtre les modèles **non dominés** sur le couple
  (qualité `Q_eff`, coût `C`) → le routeur ne choisit que parmi eux pour un budget donné.
- **Ratio qualité/prix** : `QP(m) = Q_eff(m) / (coût_par_1M(m) + ε)`. Very utile pour
  trouver « le génie pas cher » (ex. un modèle à 39 d'Index à 0,27 $/tâche vs un modèle à
  58 à 5,98 $ `[observed]` — voir §7).
- **Contrainte dure** : budget plafond par appel (règle RULES-OMEGA §3.3) → si le meilleur
  score dépasse le plafond, on retombe sur le **meilleur score Pareto-compatible**.

### 2.6 Exemple de calcul (tâche « triviale », profil vitesse) `[inferred]`

Registre restreint à 3 modèles (données §7) :

| Modèle | Q_reason→`Q_chat` | V (tok/s) | L (TTFT s) | C ($/tâche) | Score vitesse |
|---|---|---|---|---|---|
| `deepseek-v4.1-flash` (max) | 0.65 | 0.96 (187) | 0.51 (0.96s) | 0.70 | **≈ 0.72** |
| `gemini-3.5-flash-lite` | 0.35 | 1.00 (408) | 0.11 (8,2s) | 0.55 | ≈ 0.66 |
| `claude-opus-5.5` (max) | 0.95 | 0.86 (95) | 0.002 (650s) | 0.05 | ≈ 0.42 |

→ Pour une complétion triviale, le **routeur choisit le petit modèle rapide**, alors que le
profil « raisonnement long » inverserait le résultat. C'est la démonstration de la bascule
**vitesse ↔ réflexion**.

### 2.7 Biais à corriger explicitement

- **Contamination** — préférer LiveBench (renouvelé) + harnais maison pour amortir.
- **Scaffold vs modèle** — ne comparer SWE-bench/Terminal-Bench qu'à agent constant.
- **Effort de raisonnement** — un même modèle « max » vs « low » a des scores et des
  latences très différents `[observed]` → la notation doit être **par variante d'effort**.
- **Auto-déclaration** — ne jamais faire confiance à un chiffre éditeur sans corroboration
  (Epoch AI documente des écarts significatifs entre annonces et ré-exécution).
- **Mode de réponse** — TTFT « max reasoning » peut dépasser 600 s `[observed]` : à exclure
  des profils interactifs.

---

## 3. Taxonomie des types de tâches (pour le routeur)

| # | Type de tâche | Signaux déclencheurs (heuristique déterministe) | Archétype de modèle optimal | Poids dominant |
|---|---|---|---|---|
| T1 | **Complétion triviale / small-talk** | < ~8 mots, salutations, « ok/merci » | Petit modèle ultra-rapide (flash-lite, Nano, SSM/diffusion) | Vitesse + coût |
| T2 | **Résumé / reformatage** | « résume », « liste », « convertis », texte moyen | Modèle « utility » bon marché, longue fenêtre | Coût + vitesse |
| T3 | **Édition ciblée** (1 fichier, patch simple) | patch, « corrige », diff, petit contexte | Modèle code « éditeur » (respect strict du format) | Qualité code + tools |
| T4 | **Refactor / multi-fichiers** | « refactorise », plusieurs fichiers, contrat | Modèle code/reasoning de forte capacité | Qualité code + raisonnement |
| T5 | **Implémentation from scratch** | « crée », « implémente », scaffolding | Fort raisonnement + code (agentique) | Qualité code + agent |
| T6 | **Raisonnement long / plan** | « conçois », « architecte », « stratégie », > 40 mots | Modèle *reasoning* haut de gamme, effort élevé | Raisonnement |
| T7 | **Maths / sciences dures** | formules, preuves, GPQA-like | Modèle raisonnement (thinking) + outils calcul | Raisonnement |
| T8 | **Recherche / synthèse web** | « cherche », « compare », sources | Modèle knowledge + long contexte + tools web | Connaissance + tools |
| T9 | **Agentique terminal / ops** | commandes shell, scripts, CI | Modèle agent fort (Terminal-Bench/SWE) | Agent |
| T10 | **Vision / OCR / UI** | image en entrée | VLM multimodal | Vision |
| T11 | **Transcription / audio** | fichier audio | STT (Whisper, Voxtral, gpt-audio) `[observed]` | Audio |
| T12 | **Embeddings / RAG** | indexation, similarité | Modèle d'embedding (MTEB) — **jamais** un LLM | Embeddings |
| T13 | **Génération média** (image/vidéo/audio) | « génère une image », « TTS » | Modèle média dédié (Gemini image, Lyria, etc.) `[observed]` | Qualité média |
| T14 | **Chat généraliste** | conversation sans signal fort | Modèle « standard » polyvalent | Équilibré |

**Règles de routage structurantes** `[inferred]` :

- Le **classifieur de tâche** est **déterministe** (regex + longueur + présence de pièces
  jointes + mots-clés) : *« si ça s'écrit en 20 lignes, ce n'est pas un appel LLM »*
  (RULES-OMEGA §3.2). Un LLM ne doit **pas** décider du routage (coût + latence + non
  déterminisme).
- Chaque type mappe un **profil de pondération** (§2.4) et une **liste de capacités
  requises** (outils ? vision ? contexte minimal ?).
- **Escalade** : si le type est ambigu ou si la première tentative échoue le test de
  vérification, on **monte d'un cran** vers un profil plus « réflexion ».

---

## 4. Métriques de décision

| Métrique | Définition | Où la trouver | Usage dans le score |
|---|---|---|---|
| **Latence (TTFT)** | Temps jusqu'au 1er token, en s | AA (`Latency First Chunk`) `[observed]` | `L(m)` ; critique pour T1/T2 |
| **Débit** | Tokens/s médian en sortie | AA (`Median Tokens/s`) `[observed]` | `V(m)` ; T1/T2 |
| **Latence totale** | Temps de réponse complet | AA (`Total Response`) `[observed]` | Budget interactif |
| **Coût** | USD / 1M tokens **entrée** et **sortie** | OpenRouter `pricing`, models.dev `cost` `[observed]` | `C(m)` ; budget |
| **Coût par tâche** | USD moyen observé pour une tâche type | AA (`Cost per Task`) `[observed]` | Alternative plus réaliste au $/1M |
| **Fenêtre de contexte** | Tokens d'entrée+sortie max | OpenRouter `context_length`, models.dev `limit.context` `[observed]` | `K(m)` ; filtre dur |
| **Tokens de sortie max** | Plafond de génération | OpenRouter `top_provider.max_completion_tokens` `[observed]` | Filtre dur |
| **Rate limits** | RPM/TPM par provider | Docs providers ; mesures maison | `R(m)` ; déclenche fallback |
| **Fiabilité / dispo** | Uptime, taux 429/5xx, latence p95 | Télémétrie maison `[todo]` | `R(m)` |
| **Support outils** | Function-calling / tool_choice | OpenRouter `supported_parameters` (`tools`), models.dev `tool_call` `[observed]` | Filtre dur + `O(m)` |
| **Sortie structurée** | JSON schema / response_format | OpenRouter `supported_parameters`, models.dev `structured_output` `[observed]` | `O(m)` ; agents |
| **Multimodalité** | Entrée/sortie image, audio, vidéo, fichier | OpenRouter `architecture.modality` / `input_modalities`, models.dev `modalities` `[observed]` | Filtre dur |
| **Reasoning natif** | Modèle « thinking » pilotable (budget de raisonnement) | models.dev `reasoning`, OpenRouter variantes `:low/:high` `[observed]` | Choix du profil |
| **Poids ouverts** | `open_weights` → candidat self-host | models.dev `open_weights` `[observed]` | `O(m)` ; politique gratuite |

**Granularité importante** `[observed]` : la notation doit être **par variante d'effort**
(ex. `claude-opus-5-5` en `low/medium/high/xhigh/max`, `deepseek-v4.1-flash` en
`reasoning/non-reasoning`) car vitesse, latence, coût et qualité changent radicalement entre
niveaux. Le registre doit donc indexer des **points de service** `(modèle × effort × provider)`,
pas seulement des modèles.

---

## 5. Sources open-source de catalogues de modèles (alimentation automatique du registre)

**Everything is a plugin** : chaque catalogue est un **adaptateur de synchronisation** vers
le même contrat `ModelRecord`.

| Catalogue | URL / endpoint | Contenu | Relevé 2026-10-15 |
|---|---|---|---|
| **OpenRouter** | `https://openrouter.ai/api/v1/models` | `id`, `context_length`, `pricing.{prompt,completion}`, `architecture.modality`, `supported_parameters` (dont `tools`), `top_provider`, `knowledge_cutoff` | **462 modèles**, **17 entièrement gratuits** `[observed]` |
| **models.dev** | `https://models.dev/api.json` | **229 providers**, champs `cost.{input,output}`, `limit.{context,output}`, `tool_call`, `reasoning`, `attachment`, `open_weights`, `modalities`, `release_date`, `knowledge` | **8 818 modèles** ; 7 734 `tool_call`, 6 524 `reasoning`, 3 911 `open_weights` `[observed]` |
| **Ollama library** | `https://ollama.com/library` ; local `GET http://localhost:11434/api/tags` | Modèles GGUF prêts à l'emploi, tailles de quantif, familles | Catalogue self-host `[observed]` |
| **HuggingFace Hub** | `https://huggingface.co/api/models?sort=trendingScore&limit=N`, `.../api/models/{id}` | Métadonnées modèles (tags, pipeline_tag, downloads, likes, license) | Page trend `[observed]` |
| **Fournisseurs directs** | `{base_url}/v1/models` (OpenAI-compat, Anthropic, Gemini…) | Liste des identifiants disponibles sur le compte | À agréger `[todo]` |
| **LM Studio / vLLM / llama.cpp** | `http://localhost:.../v1/models` | Backends locaux OpenAI-compat | Self-host `[inferred]` |
| **LiteLLM** | Repo `model_prices_and_context_window.json` | Table prix/contexte multi-providers | Alternative/redondance `[inferred]` |

**Design du synchroniseur** `[inferred]` :
1. Chaque source → un `CatalogAdapter` (fetch + schéma de mapping vers `ModelRecord`).
2. Normalisation : `cost` en USD/1M, `context` en tokens, capacités en booléens, `license`.
3. **Déduplication** par `(famille, version, variante)` ; **union** des providers (un modèle
   peut exister chez 5 providers → redondance → meilleur taux de succès).
4. **Fusion des scores** : joindre par `id`/alias aux scores de benchmark (§2).
5. Cadence : catalogues quotidiens ; benchmarks hebdo ; **cache versionné** + diff/alertes.

---

## 6. Architecture d'un routeur (proposition)

### 6.1 Composants (tous plugins derrière des ports)

```
                       ┌──────────────────────────── PORT : CatalogSource ───────────────────────────┐
                       │  openrouter · models.dev · ollama · huggingface · provider /v1/models       │
                       └───────────────────────────────────┬─────────────────────────────────────────┘
                                                           v  (sync + normalise)
                  ┌──────────────────────────── Registre de capacités ─────────────────────────────┐
                  │  ModelRecord* : id, provider, effort, ctx, coût, tok/s, TTFT,                 │
                  │  tool_call, structured_output, modalités, open_weights, licence, santé        │
                  └───────────────────────────────────┬─────────────────────────────────────────────┘
                                                      v
   [Tâche] ──► Classifieur ──► Profil de pondération ──► Moteur de scoring ──► Filtre capacités ──► Sélecteur
   (texte+   (déterministe)   (par type T1..T14)        (composite §2 +       (outils/vision/    (argmax puis
    pièces)                                             Pareto §2.5)          contexte/licence)  tie-break prix/lat)
                                                      ▲                            │
                                                      │                            v
             ┌──────────────────── Cache de décision ─┴────────┐         Adaptateur provider (LLM)
             │  clé = hash(tâche, profil, état santé registre)  │                 │
             └──────────────────────────────────────────────────┘                 v
                                                                        Exécution ──► télémétrie
                                                                           │              │
                             ┌───────────────────── 429/5xx/timeout ──────┤              │ succès
                             v                                             │              v
                    Échelle de fallback (niveau N+1)                       │      Boucle de feedback
                    (même tier → tier sup. → self-hosted)                 │      (bandit : qualité observée,
                             │                                             │       coût réel, latence p95)
                             └──────────── met à jour santé + cooldown ◄───┘              │
                                                                                          v
                                                                              Mise à jour des priors de scoring
```

### 6.2 Composants en détail

1. **Classifieur de tâche** (T1..T14, §3) — 100 % déterministe, scriptable/testable.
2. **Registre de capacités** — source de vérité des modèles, synchronisée depuis les
   catalogues (§5). Expose : capacités, coûts, limites, **état de santé** (circuit breaker).
3. **Moteur de scoring** — composite §2, recalculé (cache mémoire) à chaque changement de
   catalogue ou de santé.
4. **Sélecteur** — `argmax Score(t,m)` sur les candidats **éligibles** (filtres durs) ;
   égalité → coût, puis latence, puis préférence self-hosted.
5. **Cache de décision** — évite de re-scorer à chaque token ; invalidé si l'état santé
   change. Clé : `hash(prompt_features, profil, version_registre)`.
6. **Politique de fallback (échelle de robustesse)** `[stated]` :
   ```
   429/5xx/timeout  →  (1) même tier, autre provider/model        (redondance catalogue)
                       (2) tier supérieur de qualité              (escalade réflexion)
                       (3) self-hosted (Ollama/vLLM)              (jamais bloqué)
                       (4) file d'attente + backoff exponentiel
   ```
   Circuit breaker : après N échecs consécutifs → `blacklist` + `cooldown`. Puis remise en
   service progressive (half-open).
7. **Boucle de feedback / bandit** — récompense observée :
   ```
   reward(m) = q_obs(m) − λ·coût_norm(m) − μ·latence_norm(m)
   ```
   Algorithmes candidats : ε-greedy (simple), **UCB1** (exploration bornée), ou
   **Thompson sampling** (robuste au non-stationnaire). Mise à jour des priors → « le système
   apprend de chaque run » `[stated]` (pilier 6).
8. **Garde budgétaire** — plafond par appel et par session ; refus/escalade contrôlée.
9. **Télémétrie** — trace chaque décision (modèle, effort, provider, coût, latence, tokens,
   issue) pour audit et ré-entraînement des priors.

### 6.3 Contrats (ports) minimaux `[inferred]`

```
trait CatalogSource   { fn fetch(&self) -> Vec<RawModel>; }
trait ModelRegistry   { fn records(&self, q: CapabilityQuery) -> Vec<ModelRecord>; fn health(&self, id) -> Health; }
trait TaskClassifier  { fn classify(&self, req: Request) -> TaskType; }
trait ScoringPolicy   { fn weights(&self, t: TaskType) -> Weights; fn score(&self, t, m) -> f32; }
trait Selector        { fn pick(&self, t, candidates) -> Decision; }
trait FallbackPolicy  { fn next(&self, failed: Decision, state) -> Option<Decision>; }
trait ProviderAdapter { fn invoke(&self, m: ModelRecord, req) -> Result<Response>; }
trait FeedbackStore   { fn record(&self, ev: Telemetry); fn priors(&self, m) -> Priors; }
```

> Aucun fournisseur en dur : le registre est **données + plugins** (RULES-OMEGA §1, §7).

---

## 7. Chiffres réels récents (datés)

> **Source des scores « Intelligence / vitesse / coût » :** Artificial Analysis, page
> *LLM Leaderboard*, **relevé le 2026-10-15** `[observed]`. L'« Intelligence Index » est un
> indice propriétaire (plus haut = meilleur). « Coût/tâche » = USD par tâche type.
> « tok/s » = médiane en sortie. « TTFT » = « first chunk » (inclut le raisonnement sur les
> variantes max → peut être très élevé).
> **Source des prix $/1M :** OpenRouter `GET /api/v1/models`, **relevé le 2026-10-15** `[observed]`.
> **Source Aider Polyglot :** `aider.chat/docs/leaderboards/` (runs **2025**, datés par run).

### 7.1 Frontière (qualité maximale)

| Modèle (effort) | Intel. Index | Coût/tâche | tok/s | TTFT (s) | Contexte | $/1M in→out | Source/date |
|---|---|---|---|---|---|---|---|
| Claude Opus 5.5 (max) | **58** | $5.98 | 95 | 650 | 1M | $4→$20 | AA / OR 2026-10-15 |
| Claude Sonnet 5.5 (max) | 56 | $5.46 | 137 | 435 | 1M | $2→$10 | AA / OR 2026-10-15 |
| Claude Opus 5.5 (xhigh) | 56 | $3.46 | 82 | 145 | 1M | — | AA 2026-10-15 |
| Claude Fable 5.1 (max) | 53 | $7.63 | 71 | 250 | 1M | $10→$50 (in) | AA / OR 2026-10-15 |
| GPT-6 Astra (max) | 53 | $3.26 | 61 | 284 | 1M | $10→$50 | AA / OR 2026-10-15 |
| GPT-6.1 Sol (max) | 52 | $0.72 | 59 | 279 | 1M | $2→$10 | AA / OR 2026-10-15 |
| Grok 4.7 (xhigh) | 46 | $3.74 | 86 | 101 | 500k | $2→$6 | AA / OR 2026-10-15 |
| Gemini 4 Argon (high) | 53 | $1.99 | — | — | 1M | $2→$10 | AA / OR 2026-10-15 |

### 7.2 Meilleurs ratios qualité/prix & open-weights (self-host friendly)

| Modèle | Intel. Index | Coût/tâche | tok/s | Contexte | Note | Date |
|---|---|---|---|---|---|---|
| **GLM-5.3-Flash** (Z AI) | 42 | **$0.28** | 67 | 1M | très bon ratio | 2026-10-15 `[observed]` |
| **DeepSeek V4.1 Flash** (max) | 39 | **$0.27** | 187 | 1M | TTFT **0,96 s** — idéal « rapide + capable » | 2026-10-15 |
| Ling 3.1 Flash (InclusionAI) | 41 | $1.00 | 209 | 1M | rapide | 2026-10-15 |
| DeepSeek V4 Pro 0813 (max) | 36 | $0.67 | 99 | 1M | — | 2026-10-15 |
| Qwen3.8 27B (xhigh) | 34 | $1.01 | 45 | 256k | famille Qwen | 2026-10-15 |
| MiniMax-M3 | 29 | $0.51 | 107 | 1M | — | 2026-10-15 |
| Inkling Small (Thinking Machines) | 26 | $0.09 | 182 | 1M | audio+image en entrée | 2026-10-15 |
| Nemotron 3.5 Lightning (NVIDIA) | 13 | $0.09 | **302** | 1M | très rapide | 2026-10-15 |
| **gpt-oss-120b** (high, OpenAI OSS) | 12 | $0.11 | 233 | 131k | open-weights | 2026-10-15 |
| **gpt-oss-20b** | 9 | **$0.01** | 219 | 131k | open-weights, presque gratuit | 2026-10-15 |
| **Gemma 4 31B** (Google) | 15 | **$0.00** | 35 | 256k | open-weights / endpoint gratuit OR | AA / OR 2026-10-15 |
| Gemma 4 26B A4B | 17* | $0.08 (paid) | 70 | 262k | gratuit sur OR `:free` | AA / OR 2026-10-15 |
| Gemma 4 12B | 14* | — | 111 | 262k | — | AA 2026-10-15 |
| **Llama 4 Maverick** (Meta) | 10 | — | 44 | 1M | open-weights | AA 2026-10-15 |
| **Llama 4 Scout** (Meta) | 8 | — | 68 | **10M** | plus grand contexte du classement | AA 2026-10-15 |
| Llama 3.3 70B | 8 | — | 93 | 128k | open-weights | AA 2026-10-15 |
| Qwen3.5 122B A10B | 16 | $0.32 | 127 | 262k | open | AA 2026-10-15 |
| Mistral Small 4 | 11 | $0.02 | 165 | 256k | open | AA / OR 2026-10-15 |
| Command A+ (Cohere) | 13 | $0.00 | 187 | 192k | endpoint à coût nul | AA / OR 2026-10-15 |
| Granite 4.2 8B (IBM) | 11 | $0.02 | 59 | 131k | open | AA / OR 2026-10-15 |
| Trinity Large Thinking (Arcee) | 11 | $0.12 | **352** | 262k | rapide + reasoning | AA / OR 2026-10-15 |

### 7.3 Curiosités « vitesse extrême » (profil T1)

| Modèle | tok/s | Coût/tâche | Note |
|---|---|---|---|
| **Celeris-1** | **1 549** | $0.05 | le plus rapide du classement AA |
| **Mercury 2.5** (Inception) | 762 | $0.12 | LLM « diffusion », TTFT court |

### 7.4 Aider Polyglot (édition de code) — runs **2025** `[observed]`

| Modèle (effort) | Correct (Pass@2) | Coût total (225 cas) | Format bien formé |
|---|---|---|---|
| gpt-5 (high) | 88.0 % | $29.08 | 91.6 % |
| gpt-5 (medium) | 86.7 % | $17.69 | 88.4 % |
| o3-pro (high) | 84.9 % | $146.32 | 97.8 % |
| gemini-2.5-pro (32k think) | 83.1 % | $49.88 | 99.6 % |
| **DeepSeek-V3.2-Exp (Reasoner)** | 74.2 % | **$1.30** | 97.3 % |
| **DeepSeek-V3.2-Exp (Chat)** | 70.2 % | **$0.88** | 98.2 % |
| Kimi K2 | 59.1 % | $1.24 | 92.9 % |
| **gpt-oss-120b (high)** | 41.8 % | **$0.74** | 79.1 % |
| Qwen3 32B | 40.0 % | $0.76 | 83.6 % |

> **Lecture routeur** : DeepSeek-V3.2 (2025) réalisait ~74 % du résultat de gpt-5 pour
> **~4 % du coût** → parfait archétype « code efficace ». Les scores AA 2026 (DeepSeek V4.1
> Flash : Index 39, 0,27 $/tâche) confirment cette trajectoire économique.

### 7.5 SWE-bench / Terminal-Bench

- **SWE-bench Verified** : 500 instances validées `[observed]`. La vue **Bash Only**
  (mini-SWE-agent) isole la performance du **LLM** du harnais → à privilégier pour la
  notation. Le dataset complet et les résultats sont sur `swebench.com` et la collection
  HF `SWE-bench/benchmarks` `[observed]`. *(Les tableaux sont chargés dynamiquement : les
  valeurs exactes doivent être extraites par un adaptateur dédié `[todo]`.)*
- **Terminal-Bench 4.0** : leaderboard dynamique (`tbench.ai`) ; les runs officiels
  `tb-4-0-0-*` (Opus 5 / Sonnet 5 / Fable 5 + Claude Code ; GPT-5.6 Sol/Luna/Terra + Codex)
  sont publiés dans `github.com/harbor-framework/terminal-bench/leaderboard/runs/`
  `[observed]`. **Le score dépend de l'agent** → ne jamais attribuer le score au seul modèle.

### 7.6 Epoch Capabilities Index (échelle unifiée) `[observed]`

- **Top ECI : Claude Opus 5.5 = 167**, devant GPT-6 Astra ; Claude Sonnet 5.5 ≈ Fable 5.1
  (165). Relevé Epoch AI du **2026-10-01**. L'ECI agrège de nombreux benchmarks en une
  échelle latente unique — utile comme **méta-score** de qualité, indépendant d'Artificial
  Analysis (bonne source de corroboration croisée).

---

## 8. Sources (URLs)

**Benchmarks & classements**
- Artificial Analysis — Leaderboard : https://artificialanalysis.ai/leaderboards/models
- Artificial Analysis — Méthodologie : https://artificialanalysis.ai/methodology
- LMArena / Chatbot Arena : https://lmarena.ai/leaderboard (et Space HF : https://huggingface.co/spaces/lmarena-ai/chatbot-arena-leaderboard)
- HuggingFace Open LLM Leaderboard v2 : https://huggingface.co/spaces/open-llm-leaderboard/open_llm_leaderboard
- HF Open LLM Leaderboard — résultats : https://huggingface.co/datasets/open-llm-leaderboard/results
- SWE-bench : https://www.swebench.com/ · Verified : https://www.swebench.com/verified.html
- SWE-bench (paper) : https://arxiv.org/abs/2310.06770
- mini-SWE-agent : https://github.com/SWE-agent/mini-swe-agent
- Terminal-Bench 4.0 : https://www.tbench.ai/leaderboard · code : https://github.com/harbor-framework/terminal-bench
- Aider Polyglot Leaderboard : https://aider.chat/docs/leaderboards/ · notes : https://aider.chat/docs/leaderboards/notes.html
- LiveBench : https://livebench.ai/ · code : https://github.com/LiveBench/LiveBench
- GPQA (paper) : https://arxiv.org/abs/2311.12022
- MMLU-Pro (paper) : https://arxiv.org/abs/2406.01574
- BigCodeBench : https://bigcode-bench.github.io/
- Epoch AI — Capabilities & benchmarking : https://epoch.ai/benchmarks · ECI : https://epoch.ai/eci · données : https://epoch.ai/benchmarks/use-this-data
- lm-evaluation-harness (EleutherAI) : https://github.com/EleutherAI/lm-evaluation-harness

**Catalogues de modèles**
- OpenRouter API : https://openrouter.ai/api/v1/models
- models.dev : https://models.dev/ · API : https://models.dev/api.json
- Ollama library : https://ollama.com/library
- HuggingFace Hub API : https://huggingface.co/api/models

**Standards / outillage complémentaires**
- Model Context Protocol : https://github.com/modelcontextprotocol
- LiteLLM (table prix/contexte) : https://github.com/BerriAI/litellm

---

## Annexe A — Trame de `ModelRecord` (registre) `[inferred]`

```yaml
id: deepseek-v4.1-flash              # identifiant canonique
aliases: [deepseek/deepseek-v4.1-flash]
family: deepseek
variants:                            # un point de service par effort
  - effort: reasoning
    provider: opencode_zen
    tokens_per_s: 187
    ttft_s: 0.96
    cost_usd_per_1m: { input: 0.02, output: 0.89 }
  - effort: non-reasoning
    provider: opencode_zen
    tokens_per_s: 193
    ttft_s: 1.05
    cost_usd_per_1m: { input: 0.02, output: 0.89 }
capabilities: { tool_call: true, structured_output: true, reasoning: true,
                vision: false, audio: false, video: false }
context: { window: 1048576, max_output: null }
open_weights: false
license: proprietary
benchmarks:                          # datés, sourcés
  aa_intelligence_index: { value: 39, date: 2026-10-15, source: artificial_analysis }
  aider_polyglot_v3_1:   { value: null }
health: { status: ok, consecutive_failures: 0, uptime_7d: null }
scores:                              # recalculés par le moteur §2
  Q_reason: 0.65
  Q_code: 0.62
  V: 0.96
  L: 0.51
  C: 0.70
```

## Annexe B — Backlog de recherche `[todo]`

1. Écrire des adaptateurs d'extraction pour les tableaux JS (SWE-bench, Terminal-Bench,
   LMArena) — le HTML statique ne suffit pas.
2. Valider l'API JSON d'Artificial Analysis (clé requise) et formaliser un cache.
3. Construire un **harnais maison** (lm-evaluation-harness) sur 2-3 tâches représentatives
   pour un ancrage non contaminé.
4. Spécifier la **politique de santé** (circuit breaker, cooldown, half-open) en ADR.
5. Définir le format de télémétrie et la récompense du bandit (test offline sur traces).
6. Trancher : cache de décision (TTL vs invalidation événementielle) en ADR.
```

