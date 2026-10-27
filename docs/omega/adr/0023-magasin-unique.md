# ADR-0023 — Magasin mémoire unique pour l'outil et l'injection (B2c)

## Statut
Accepté (2026-10-27).

## Contexte
L'outil `memory_store`/`recall`/`forget` (`omega-tools`) et l'injection C4
(`omega-context`, câblée dans `omega-server/src/state.rs`) lisaient/écrivaient
deux magasins disjoints : l'outil écrivait `.omega/memory.jsonl` dans le
workspace (jail), l'injection lisait `event_dir/memory.jsonl` via
`FileMemoryStore`, et `omega-memory::JsonlMemoryStore` restait orphelin en prod.
Résultat : S4 ne pouvait pas passer (la preuve devait traverser les processus).

La tentative précédente (env `OMEGA_MEMORY_STORE`) échouait : `ToolCtx::resolve`
confine tout chemin au workspace, donc un chemin absolu hors workspace est
rejeté ("chemin hors workspace").

## Décision
- Un seul magasin : `FileMemoryStore` (`omega-app`) derrière le port
  `MemoryStore`, instance construite **une fois** dans `build_app_state_with`
  (`state.rs`), partagée par l'injection C4.
- Le même fichier (`event_dir/memory.jsonl`) est aussi exposé au `ToolCtx` des
  outils via `ToolCtx::memory_path` (chemin absolu fourni par le serveur, hors
  jail : c'est le serveur qui choisit, jamais le modèle).
- `FileMemoryStore` relit le fichier à chaque `recall` (pas de cache en
  mémoire) et accepte les deux formes JSONL (chaîne brute et objet
  `{"content","provenance"}`), pour que l'outil et l'injection lisent le même
  fichier.
- `omega-memory::JsonlMemoryStore` reste l'adaptateur de conformité A1 et
  fournit la cascade BM25/vecteur testée par G2.h ; il n'est pas le magasin
  de production (dette nommée : le câbler derrière le port si besoin de BM25
  en production).

## Conséquences
- S4 passe honnêtement : fait stocké par l'outil en session A retrouvé dans la
  requête reçue par le LLM en session B (nouveau `run_id`).
- Le contrôle négatif (magasin vidé) rend S4 rouge.
- Mutation (R5) doit encore couvrir `omega-tools::memory` et `omega-app::runtime_ports`.