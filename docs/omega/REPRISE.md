# REPRISE sur une nouvelle machine

> Objet : pointer vers la mécanique existante, pas redire le projet.
> La vision et l'état : `docs/omega/PROJET-ET-VISION.md` · plan vivant : `docs/omega/PLAN-MAITRE.md` · journal append-only : `docs/omega/plan/journal.md`.

```bash
git clone --branch feat/agentos-omega https://github.com/TayK0oo/odysseusAgentOS.git odysseusAgentOS-omega
cd odysseusAgentOS-omega

# Debian: python>=3.12, cargo, ruff auto-installé au bench est figé 0.6.9
python3 --version && cargo --version

# Ce que tu peux lancer sans rien installer de plus (depuis la racine)
python3 omega/scripts/verify.py --skip-tests   # pré-commit rapide (doit exit 0)
python3 omega/scripts/measure.py               # STATUS.json calculé (banc inclus ~30 s)
python3 omega/scripts/verify.py                # complet : pytest + cargo test (~10-20 min)

python3 omega/acceptance/runner.py --scenario S0   # doit être PASS
python3 omega/acceptance/runner.py --scenario S1   # doit être XFAIL nommé (G0.c)
python3 omega/acceptance/runner.py --scenario S1 --engine rust --mode reel  # futur
```

- **Adapters** : `omega/acceptance/engines/rust.py` exige le binaire `omega/target/debug/omega-server` — il se build tout seul au premier run (`cargo build -p omega-server`, lancé depuis `omega/`).
- **Chemins machine aussi** : `measures_advanced.py` sonde `/home/tayk0oo/omega-re` (62 clones) et `/home/tayk0oo/Projets/odysseusAgentOS` — si absents, les clés passent à `INCONNU(chemin_absent)`, jamais un faux 0.
- **LLM réel** (post-scripts) : `ollama pull qwen2.5-coder:7b` + `OLLAMA_NUM_CTX=32768 ollama serve`. Le banc passe toujours `OLLAMA_URL` au serveur (env contract `omega-server/src/config.rs` — aucune variable `OMEGA_LLM_URL`, elle n'existe pas).
- **Règle des modèles** : tout sous-agent sur un modèle gratuit — `python3 omega/scripts/model_policy.py --check`.
- Chiffres vivants (`<!--m:key-->` doc-truth) : réconciliation P0-3 en cours, voir ADR-0004 à 0006.
