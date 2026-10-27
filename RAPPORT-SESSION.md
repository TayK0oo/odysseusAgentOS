# RAPPORT-SESSION — v10 (2026-10-27)

SHA départ: `31a68dd9` → HEAD `021e0830` (pushé).

## Verdict G2
VRAI VERT (REV indépendante : sabotage G2.b/G2.c/G2.h/G2.d/G2.e ⇒ rouge attendu ; tautologies absentes ; orphelins ≤ baseline ; clippy -D warnings 0 ; fmt OK ; verify VERT ×2 ; CI omega-gates vert).

## Baseline mutation
- omega-memory: 11 mutants, 0 caught, 11 missed.
- omega-context: 334 mutants, 240 caught, 94 missed.
- omega-policy: 123 mutants, 78 caught, 28 missed, 17 unviable.
Fichier: `omega/docs/omega/plan/baselines/mutation-2026-10-27.json`.

## ADR-0023 (résumé ≤10 lignes)
Un seul magasin mémoire (`event_dir/memory.jsonl`) partagé par l'outil et l'injection C4, instance unique dans `state.rs`, chemin fourni au `ToolCtx` hors jail. `FileMemoryStore` relit à chaque `recall`. `omega-memory::JsonlMemoryStore` reste adaptateur de conformité (dette nommée). Conséquence : S4 passe honnêtement (fait stocké en A retrouvé en B) et le contrôle négatif (magasin vidé) rend S4 rouge.

## Dettes
- Survivants mutation (provenance/confidence helpers sans tests) à traiter ou justifier.
- Seuils : ADR-0021/0022 (choisi, dette nommée).
- Plugin `deleg` actif (constaté, non touché).

## Prochaine porte rouge
P3 (si autorisé) : ADR-0020 + inventaire + gate P3 rouge sincère. Sinon : traiter survivants mutation.

## Décisions au Chef
1. Autoriser P3 ? 2. Traiter survivants mutation avant P3 ?