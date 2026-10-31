//! Contrats de bord de `truncate_compact` / `truncate_compact_with_pinned`.
//!
//! Cible : les 11 mutants survivants du fichier `compaction.rs`
//! (`mutants.out-context-fresh`). Les budgets sont construits **exactement**
//! via l'estimateur public (`ceil(chars/4)`, sans overhead par message).
//!
//! Deux mutants (`:62` et `:355`, gardes `omitted > 0`) sont **équipotents** :
//! passé l'early-return initial, `est(tous) > budget` impose qu'au moins un
//! message du tail échoue au test d'inclusion, donc `omitted >= 1`
//! systématiquement (pigeonniel : garder TOUS les messages signifierait
//! `prefix + marqueur + tail <= budget`, contradiction avec l'early-return).
//! Aucun test comportemental ne peut les tuer — registre Livrable D.
//!
//! Contrainte de design des fixtures `with_pinned` : la fonction réserve la
//! place du marqueur PIRE CAS `marker(non_system.len())`, donc pour éviter
//! l'early-return « tout tient », il faut `Σ(tokens omis) > reserve` :
//! les messages non gardés sont dimensionnés en conséquence.

use omega_context::{
    compaction_marker, truncate_compact, truncate_compact_with_pinned, PinnedFact, PinnedFacts,
    TokenEstimator,
};
use omega_types::{Message, Role};

fn sys(text: &str) -> Message {
    Message::new(Role::System, text.to_string())
}

fn usr(text: &str) -> Message {
    Message::new(Role::User, text.to_string())
}

/// Faits épinglés : un fait dont le message système injecté est connu.
fn pinned_fixture() -> (PinnedFacts, String) {
    let pf = PinnedFacts::new();
    pf.upsert(PinnedFact::new("f1", "I prefer stable output", "pref"));
    // Contrat public de `PinnedFact::to_system_message`.
    let expected = "[PinnedFact:pref] I prefer stable output".to_string();
    (pf, expected)
}

/// tue exactement `:42` (`>` -> `>=` : à l'égalité, le mutant rendrait le
/// système seul SANS marqueur, alors que le marqueur `marker(1)` rentre pile).
#[test]
fn truncate_compact_at_exact_system_plus_marker_budget_keeps_marker() {
    let est = TokenEstimator::new();
    let m1 = est.estimate(&compaction_marker(1));

    let msgs = vec![sys("zzzz"), usr(&"w".repeat(4 * (m1 + 1) - 1))];
    // system = 1 token ; tail = m1 + 1 tokens (non nul donc non tenu au budget).

    let budget = 1 + m1;
    let est_all = est.estimate_messages(&msgs);
    assert_eq!(
        est_all,
        budget + 1,
        "fixture: early-return doit être inopérant"
    );

    let out = truncate_compact(&msgs, budget);
    // Contract : système + marqueur exactement (rien d'autre ne tient).
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].content, "zzzz");
    assert_eq!(out[1].content, compaction_marker(1));
}

/// tue exactement `:334` (`>` -> `>=` : à l'égalité `prefix + marqueur`,
/// le mutant rendrait le préfixe seul sans marqueur).
#[test]
fn truncate_compact_with_pinned_at_exact_prefix_plus_marker_budget_keeps_marker() {
    let est = TokenEstimator::new();
    let (pf, pinned_content) = pinned_fixture();
    let m1 = est.estimate(&compaction_marker(1));

    let sys_text = "s".repeat(32); // 8 tokens
    let tail_usr = usr(&"t".repeat(4 * (m1 + 1) - 1)); // m1 + 1 tokens
    let tail_tokens = est.estimate(&tail_usr.content);
    let msgs = vec![sys(&sys_text), tail_usr];

    let prefix_tokens = est.estimate(&sys_text) + est.estimate(&pinned_content);
    let budget = prefix_tokens + m1;
    // Garde-fous de fixture : pas d'early-return (`est(tous) > budget`), et le
    // message du tail ne tient pas (sinon le contrat testé ne se joue pas).
    let est_all = est.estimate_messages(&msgs);
    assert!(est_all + est.estimate(&pinned_content) > budget);
    assert!(tail_tokens > m1);

    let out = truncate_compact_with_pinned(&msgs, budget, &pf);
    // Contract : préfixe (système + fait épinglé) + marqueur exactement.
    assert_eq!(out.len(), 3);
    assert_eq!(out[0].content, sys_text);
    assert_eq!(out[1].content, pinned_content);
    assert_eq!(out[2].content, compaction_marker(1));
}

/// tue `:340` (`+` -> `-` / `*`) et `:343` (`+` -> `-` / `*`) : originel
/// exactement `{c}` tenu ; le mutant `-`/`-` en garde plus (aucun marqueur),
/// le mutant `*` en garde moins (marker(3)).
#[test]
fn with_pinned_selection_matches_exactly_the_suffix_that_fits() {
    let est = TokenEstimator::new();
    let (pf, pinned_content) = pinned_fixture();
    // Réserve PIRE CAS de la fonction : marker(non_system.len()) = marker(3).
    let m3 = est.estimate(&compaction_marker(3));
    assert_eq!(m3, 8, "fixture : la taille du marqueur en tokens");

    let sys_text = "s".repeat(32); // 8 tokens (P >= 18 avec le fait épinglé)
                                   // non_system chronologique : a = 1 TK, b = 9 TK, c = 3 TK.
    let msgs = vec![
        sys(&sys_text),
        usr("a000"),
        usr(&"b".repeat(35)),
        usr(&"c".repeat(11)),
    ];

    let prefix_tokens = est.estimate(&sys_text) + est.estimate(&pinned_content);
    let budget = prefix_tokens + m3 + 3;
    // Early-return évacué : est(tous) = P + 13 > budget = P + 11 (13 > 8 + 3).

    // Originel : c tient pile (equality), b échoue (+ 9), break → kept = {c}.
    let out = truncate_compact_with_pinned(&msgs, budget, &pf);
    assert_eq!(out.len(), 4);
    assert_eq!(out[0].content, sys_text);
    assert_eq!(out[1].content, pinned_content);
    assert_eq!(out[2].content, compaction_marker(2));
    assert_eq!(out[3].content, "c".repeat(11));

    // Mutants :
    // - initialisation `prefix - marker` ou check `kept - t` : a et b admis
    //   aussi (aucun marqueur, et le contenu se rallonge) ;
    // - initialisation `prefix * marker` ou check `kept * t` : c échouerait,
    //   kept vide → marker(3) inséré.
}

/// tue `:344` (`+=` -> `-=` / `*=`) : deux messages tenus originellement
/// (`{b, c}`) ; `-=`, comme `*=` ici, ne peut reproduire ni l'ensemble ni la
/// séquence — `-=` en garde un troisième, `*=` un seul.
#[test]
fn with_pinned_accumulator_grows_across_two_kept_messages() {
    let est = TokenEstimator::new();
    let (pf, pinned_content) = pinned_fixture();
    let m3 = est.estimate(&compaction_marker(3));
    assert_eq!(m3, 8, "fixture : la taille du marqueur en tokens");

    let sys_text = "s".repeat(32);
    // a = 9 TK (36 chars), b = c = 3 TK (9 chars) : Σ(omis) = 9 > m3.
    let msgs = vec![
        sys(&sys_text),
        usr(&"a".repeat(35)),
        usr(&"b".repeat(9)),
        usr(&"c".repeat(9)),
    ];

    let prefix_tokens = est.estimate(&sys_text) + est.estimate(&pinned_content);
    let budget = prefix_tokens + m3 + 6;
    // Early-return évacué : est(tous) = P + 15 > budget = P + 14 (15 > 8 + 6).

    // Originel : c (+3), b (+3) tiennent ; a échoue → kept = {b, c}.
    let out = truncate_compact_with_pinned(&msgs, budget, &pf);
    assert_eq!(out.len(), 5);
    assert_eq!(out[0].content, sys_text);
    assert_eq!(out[1].content, pinned_content);
    assert_eq!(out[2].content, compaction_marker(1));
    assert_eq!(out[3].content, "b".repeat(9));
    assert_eq!(out[4].content, "c".repeat(9));

    // Mutant `+= -> -=` : kept décroît → a serait admis (pas de marqueur).
    // Mutant `+= -> *=` : après c, kept = (P+M)*3 → b échoue → marker(2).
}

/// tue exactement `:351` (`-` -> `+` dans le compte omis) : le marqueur doit
/// indiquer `non_system.len() - kept.len()` (= 2), pas la somme (5 + 3 = 8).
#[test]
fn with_pinned_marker_counts_exactly_the_omitted_messages() {
    let est = TokenEstimator::new();
    let (pf, pinned_content) = pinned_fixture();
    // Réserve PIRE CAS de la fonction : marker(non_system.len()) = marker(5).
    let m5 = est.estimate(&compaction_marker(5));
    assert_eq!(m5, 8, "fixture : la taille du marqueur en tokens");

    let sys_text = "s".repeat(32);
    // a = b = 5 TK (17 chars) → Σ(omis) = 10 > m5 ; c/d/e = 1 TK.
    let msgs = vec![
        sys(&sys_text),
        usr(&"m".repeat(17)),
        usr(&"n".repeat(17)),
        usr("o000"),
        usr("p000"),
        usr("q000"),
    ];

    let prefix_tokens = est.estimate(&sys_text) + est.estimate(&pinned_content);
    let budget = prefix_tokens + m5 + 3;
    // Early-return évacué : est(tous) = P + 13 > budget = P + 11.

    let out = truncate_compact_with_pinned(&msgs, budget, &pf);
    // Contract : marqueur compte EXACTEMENT les 2 messages omis.
    assert!(out.iter().any(|m| m.content == compaction_marker(2)));
    assert!(
        !out.iter().any(|m| m.content == compaction_marker(8)),
        "compte muté (somme) visible : marker(8) au lieu de marker(2)"
    );
    // Les trois plus récents sont gardés, dans l'ordre.
    for name in ["o000", "p000", "q000"] {
        assert!(out.iter().any(|m| m.content == name));
    }
    assert_eq!(out.len(), 2 + 1 + 3); // prefix(2) + marqueur + 3 gardés
}
