# ADR-0007 — Doc-truth : le tag porte le chiffre

**Date** : 2026-10-19 · **Statut** : Accepté · **Décidé par** : chef de projet
**Déclencheur** : P0-3b — le plan (§4) exige « tout chiffre marqué &lt;!--m:cle=VALEUR--&gt;
dans un document doit égaler `STATUS.json` », sans dire comment le chiffre
attendu voyage dans le markdown.

## Constat (observed)

Trois conventions étaient possibles. Le coût qui décide de tout n'est pas
l'écriture, c'est la **preuve** :

| Convention | Exemple (échappé : un tag littéral serait un vrai marqueur, §6) | Avantage | Coût |
|---|---|---|---|
| **A** — tag porteur | **22**. &lt;!--m:socles.rust.crates=22--&gt; | validateur trivial (regex sur le tag) ; le tag survit à un reformatage de la prose | le chiffre apparaît deux fois dans le source |
| **B** — tag marqueur | **22** &lt;!--m:socles.rust.crates--&gt; | un seul chiffre dans le source | le validateur doit matcher la prose (`\*\*([\d.,]+)\*\*`) — une virgule de phrase casse le match ; plusieurs tags par paragraphe deviennent ambigus |
| **C** — placeholder | &lt;!--m:socles.rust.crates--&gt; | zéro duplication, zéro drift interne | le `.md` brut n'affiche plus le chiffre ; il faut tester le rendu, hors périmètre P0 |

Un garde-fou que rien n'observe redevient décoratif (leçon L2). Le contrôle
négatif de **A** est une ligne : un fichier bidon avec `=99` doit produire
`exit 1`. Celui de **B** exige de casser aussi la prose — on peut se tromper,
croire le test bon, et garder six mois un validateur qui ne détecte rien.
Celui de **C** exige de tester le rendu.

## Décision

1. **Convention A** : &lt;!--m:CLE=VALEUR--&gt;, tag **auto-suffisant**.
2. `CLE` est un chemin pointé dans `STATUS.json`. Si la cible est de la forme
   `{"valeur": ..., "methode": ...}` (les mesures de `measure.py`), c'est
   `valeur` qui est comparée — `methode` décrit, elle ne chiffre pas.
3. Comparaison après normalisation : `1 114`, `1,114` et `1114` sont le même
   chiffre. Un chiffre se compare ; il ne se recopie pas à l'identique.
4. **Tolérant en P0-3b, strict en P0-3c.** Une clé absente de `STATUS.json` est
   un WARNING par défaut, une ERREUR avec `--strict`. Le basculement se fait à
   la fin de P0-3c, quand tous les documents sont marqués — pas avant : une
   porte bloquante rendrait le marquage impossible.
5. **Périmètre par défaut** : `docs/**/*.md` et `*.md` à la racine, extensible
   par `--paths`. Les règles `.opencode/rules/*.md` ne sont pas dans le
   périmètre par défaut ; les y ajouter est un choix explicite.
6. **Un tag dans du code est de la documentation, pas un marqueur.** Les
   régions de code — blocs délimités (triple backtick ou triple tilde) et spans
   inline (texte entre backticks) — sont **hors-champ**. Ce n'est pas une
   exception : en Markdown, le code n'est pas du contenu actif. Un exemple peut
   donc être montré tel quel dans un span ou un bloc, et le validateur le
   **compte** comme ignoré au lieu de le valider ou de le signaler comme
   divergence. Les deux occurrences existantes (`PLAN-MAITRE.md:140`,
   `REPRISE.md:27`) sont dans des spans inline : elles deviennent valides sans
   retoucher les documents.
   *Limite assumée* : un tag de mesure placé par accident dans du code n'est
   pas vérifié — le rapport donne le nombre et la position des tags ignorés,
   donc c'est visible, pas silencieux. Un bloc non fermé court jusqu'à la fin du
   document (CommonMark) ; un span multi-ligne est traité comme du texte (le
   contraire — masquer un tag par une paire de backticks lointaine — serait
   pire).
7. **Rejeté — B** (fragile, non testable négativement sans casser la prose) et
   **C** (cache le chiffre au source et déplace la preuve vers le rendu).

## Conséquences

- Mettre à jour un chiffre = `STATUS.json` (auto) **et** la valeur du tag
  (manuel, même commit). C'est voulu : publier un chiffre est une décision,
  pas une conséquence de l'exécution d'un script.
- Le validateur ne peut pas se déclarer vert sans `STATUS.json` lisible :
  `exit 1` si la référence manque ou est invalide.
- Le coût de A est un drift *interne* au document (texte vs `=VALEUR`) — que le
  validateur détecte, puisqu'il compare le tag à `STATUS.json`, pas le texte au
  tag. Le texte reste éditorial, le tag est contractuel.
- `verify` exécute l'étape en mode toléré (`"bloquant": false`) jusqu'à P0-3c ;
  le verdict brut reste affiché pour qu'un humain le voie.

## Amendement P0-3c

1. **`--strict` devient le défaut**, `--tolerant` reste l'option de secours
   (marquage en cours — jamais le défaut). Le mode toléré a servi exactement le
   temps du marquage.
2. La tolérance correspondante dans `verify` est **supprimée** : l'étape 10 est
   bloquante comme les autres, et le mécanisme `"bloquant": false` disparaît
   faute d'utilisateur. Une tolérance qui reste est une porte qui s'ouvre toute
   seule.
3. **Limite structurelle, énoncée une fois** : ce validateur prouve que les
   chiffres *marqués* sont vrais. Il ne prouve pas qu'un document n'a pas de
   chiffre vivant **non marqué** — ça se voit en revue, pas en porte. D'où la
   règle éditoriale : tout chiffre que `STATUS.json` mesure porte un tag, et un
   chiffre qui n'est pas mesuré ne se présente pas comme une mesure.
