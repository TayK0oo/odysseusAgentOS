# SCENARIOS-README — le banc d'acceptation P1 (A1)

> Quel scénario pour quoi, quel verdict/exit attendu, quel moment il devient vert.
> Sémantique complète des verdicts : `omega/acceptance/runner.py` (docstring).

## Scénarios

| id | nom | vérifie | verdict aujourd'hui | devient vert quand |
|----|-----|---------|---------------------|--------------------|
| S0 | demarrage_propre | moteur démarré, `/api/health` ok + contrôle négatif port pris | PASS | (vert, phase 0) |
| S1 | outil_reel_canary | le run écrit une trace avec un `tool_call` réel, l'observation est réinjectée (7 canaries hex, noms aléatoires, écrites dans un tmpdir) | XFAIL | P1-1 / A2 : boucle d'outils + wire tool_calls (ADR-0006) |
| S2 | limites_budgets | arrêt sur budget (steps/tokens) observable dans la réponse ET la trace | XFAIL | P1-3 : budgets appliqués dans la boucle |
| S3 | reprise_crash_append | run scripté multi-étapes avec append_file, kill -9 au milieu, relance même data dir, reprise ; fichier append exactement une ligne, journal cohérent (readback fs) ; contrôle négatif : reprise sans journal ⇒ effet dupliqué détecté | PASS | P2-C2 : idempotence outil + reprise crash |

## Pourquoi il n'existe pas de « S1-wire » distinct

La mission P1-A1 prévoyait de renommer S1 (« wire sans outils ») ou d'ajouter un
S1-tool distinct. **Fait observé en lisant l'historique** : S1 est déjà le scénario
canary outil réel — `abe4d890` l'a créé (`outil_reel_canary`, canaries hex, attendus
canaries + événement outil) et `24d86a7c` y a amendé le `xfail_raison` aux 3 causes
de l'ADR-0006 (wire, boucle, scripting en deux formes). Renommer n'aurait rien
m'ordu ; un « S1-wire » séparé dupliquerait la même émission rouge. Choix : **S1
garde son id et son contenu**, documenté ici.

## Eléments de trajectory du runner

- Exit sans `--attendu` : `PASS` ou `XFAIL` → 0 ; `FAIL`/`XPASS`/`ERREUR` → 1.
  Utilisé tel quel par les critères G0 de la phase 0 (`S1 rouge = XFAIL nommé = exit 0`).
- Exit avec `--attendu <V>` (critères G1 de `gates.toml`) : 0 ssi **chaque** run a
  produit exactement le verdict `V`. `XFAIL` sous `--attendu PASS` → exit 1 :
  le second degré n'est plus acceptable là où la porte exige un vert sec.
- `--scenario S0,S1,S2` : liste comma-séparée ; `--runs 5` : répète la liste.
- `XPASS` reste un bug de scénario attendu-rouge qui est devenu vert sans que son
  `xfail_raison` soit retiré — contrôle négatif du contrôle (testé dans
  `tests/test_omega_gate_phase1.py`).

## Scripting LLM

- `ScriptedProvider` : contrat réel du provider (openai-compatible
  `POST {base}/v1/chat/completions` + `GET {base}/api/tags`), cf. `scripted_provider.py`.
- S1/S0 : séquence echo (`seq_echoreq_reply`) — le modèle n'a **aucun** moyen
  légitime de connaître les noms de canaries : absence = norme tant que la surface
  d'outils n'existe pas (contrôle du contrôle).
- S2 (`llm_scripte: react`) : séquence `seq_react_toolcall` — le modèle répète la
  forme texte ReAct canonique d'ADR-0006 (`{"tool": ..., "arguments": ...}`), base
  pour mesurer un budget de steps. Si un jour un mot du vocabulaire budget
  apparaît dans l'écho/dans la trace sans mécanisme réel, l'attendu devient
  mensonger : les mots-marqueurs (`budget`, `limite`, `arret`) sont volontairement
  absents des messages scriptés ; l'ajout d'un moteur réel devra les produire
  **dans la trace**, pas dans la prose.
