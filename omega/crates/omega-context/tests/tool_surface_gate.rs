//! Tests comportementaux ciblés pour `omega_context::tool_surface`.
//!
//! Baseline fresh (`omega/mutants.out-context-fresh/mutants.out/missed.txt`) :
//! 4 mutants survivants sur ce fichier. Un seul est killable par un test
//! comportemental ; les 3 autres sont équipotents (preuves formelles
//! ci-dessous, registre Livrable D de la passation v3).
//!
//! - `:99` (garde externe R6 `eaux`) et `:101` (garde interne R6) :
//!   **équipotents** — pour tout token finissant par `eaux` (longueur >= 4
//!   obligatoire), la règle R7 (`x`) s'applique avec le même strip d'un
//!   caractère : retourner la même racine.
//! - `:201:84` (tie-break `t < best_tok`) : **équipotent** — `t <= best_tok`
//!   ne diffère que si `t == best_tok`, auquel cas `best` est réécrit avec le
//!   même couple `(df, token)` : aucun état observable ne change.
//! - `:182` (IDF `(df + 1)` → `(df * 1)`) : **killable** — le mutant produit
//!   une IDF **plus grande** que l'originale pour `df >= 1`
//!   (`ln((N+1)/df) > ln((N+1)/(df+1))`) ; le test ci-dessous place le seuil
//!   `min_score` **entre les deux valeurs** : la cible n'est admise que par
//!   l'IDF lissée originale, jamais par le mutant.

use omega_context::{ToolDoc, ToolSurfaceIndex};

fn tool(name: &str, desc: &str) -> ToolDoc {
    ToolDoc::new(name, desc)
}

#[test]
fn idf_smoothing_admits_doc_only_at_original_larger_df_denominator() {
    // Deux docs ; `aaa` et `zzz` ont chacun df=1, N=2.
    // IDF originale = ln((N+1)/(df+1)) + 1 = ln(3/2) + 1 ≈ 1.405.
    // Mutant `(df + 1)` → `(df * 1)` : ln(3/1) + 1 ≈ 2.099 (plus grand).
    // Gate = "aaa" (tie lexical sur df égaux) ; "target" ne contient pas la
    // gate, son admission passe par le seuil : score >= 2.0 — au-dessus de
    // 1.405 (l'originale ne l'admet pas) mais sous 2.099 (le mutant l'admet).
    let idx = ToolSurfaceIndex::from_docs(vec![tool("filler", "aaa"), tool("target", "zzz")])
        .with_min_score(2.0);

    let names: Vec<&str> = idx
        .search("aaa zzz", 10)
        .iter()
        .map(|d| d.name.as_str())
        .collect();

    // Seul "filler" est admis (il contient la gate, sans passer par le score).
    assert_eq!(names, vec!["filler"]);

    // Au mutant : score(target) ≈ 2.099 >= 2.0 → "target" serait admis aussi
    // (score égal au mutant de "filler" → départage au nom : ["filler", "target"]).
}
