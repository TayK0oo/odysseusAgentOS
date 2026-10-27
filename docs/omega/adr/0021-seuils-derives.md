# ADR-0021 — Seuils dérivés : latence mémoire p95

**Date** : 2026-10-27 · **Statut** : Accepté · **Décidé par** : N0 (chef projet)

## Contexte

ADR-0018 a gelé `G2.i.latence` à **p95 < 50 ms** en le déclarant « choisi, non dérivé » (marge ×4.5 sur une mesure de 11 ms, sans formule reproductible). Cet ADR écrit la formule qui rendrait le seuil dérivable, puis tente la dérivation sur le test réel.

## Formule

```
seuil_p95 = max(50 ms, ceil(p95_mesure × 10))
```

- **×10** : absorbe la variance machine / charge / CI (5 runs seulement, bruit d'ordonnancement, résolution en millisecondes entières via `as_millis()`).
- **plancher 50 ms** : quand `p95_mesure` vaut 0–2 ms, `×10` donnerait 0–20 ms, seuil que le bruit de mesure ferait échouer sans rien prouver.
- Unités : millisecondes, arrondi supérieur entier (le test compare des entiers).
- Règle d'application : `p95_mesure ≤ 5 ms → 50 ms` ; `p95_mesure > 5 ms → 10 × p95_mesure`.

## Mesure

Commande exacte (sacrifie le build release, 1000 souvenirs, 5 runs `recall`) :

```bash
cd omega && cargo test -p omega-memory --release -- --nocapture latency_memory_p95_under_threshold
```

Sortie brute, 1 ligne utile :

```
cargo test: 1 passed, 19 filtered out (4 suites, 0.64s)
```

Note de méthode : le filtre `latence` ne sélectionne rien (`0 passed, 20 filtered out`) — le test s'appelle `latency_memory_p95_under_threshold` (`omega/crates/omega-memory/tests/adversarial.rs:233`).

**p95_mesure = INCONNU** (motif : le test ne journalise pas la valeur ; il n'assert que `p95 < 50`. Seule la borne est observée : test PASS ⇒ `p95 < 50 ms` sur cette machine, 1000 souvenirs, 5 runs). Aucun chiffre n'est reconstitué ni estimé ici.

## Décision

Seuil `G2.i.latence` inchangé : **50 ms**, `bloquant = true`. Statut : **choisi (dette nommée)** — la formule est écrite mais son entrée n'existe pas, donc elle n'est pas évaluable. ADR-0018 reste valide, non modifié, et fait foi pour le chiffre.

**Dette nommée** : émettre `p95` dans le test (`println!("p95 = {p95} ms")` — fichier `.rs`, hors périmètre de cet ADR) puis rejouer la commande ci-dessus. La formule tranche alors mécaniquement ; le seuil ne devient « dérivé » qu'à ce moment-là.

## Conséquences

- `omega/gates.toml` non modifié par cet ADR (aucune valeur de seuil à changer tant que la dérivation est bloquée).
- Tant que `p95_mesure` n'est pas émis, tout chiffre autre que 50 ms est indéfendable : le seuil reste une décision, assumée comme telle.
- La dérivation complète exige deux entrées : `p95` journalisé **et** 5 runs baseline (plan §9).
