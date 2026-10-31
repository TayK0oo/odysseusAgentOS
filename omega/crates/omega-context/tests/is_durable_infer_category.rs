//! Contrats comportementaux de l'heuristique deterministe `is_durable` et
//! `infer_category` atteints via la surface publique `PinnedFacts::extract`.
//!
//! Chaque test vise un groupe homogene de mutants : chaines `||`/`&&` de
//! l'heuristique, bornes exactes (cle <= 40, valeur <= 200), arithmetique de
//! decoupage `idx + 2`, deduplication et compteur d'identifiants, categories
//! moteur (preference/decision/constraint/project/fact).
//!
//! Les mutants de la chaine de marqueurs explicites (lignes 220-227 de
//! `pinned_facts.rs`) sont des **mutants equivalents** : tout marqueur
//! contient deja son mot-cle (`[PINNED]` contient `pinned`, `Decision:`
//! contient `decision`, etc.), donc la garde des mot-cles revele `true`
//! avant d'atteindre cette chaine pour toute entree qui la foudroyerait.
//! Voir le registre des risques (Livrable D).

use omega_context::PinnedFacts;
use omega_types::{Message, Role};

fn extract_line(line: &str) -> PinnedFacts {
    PinnedFacts::extract(&[Message::new(Role::User, line.to_string())])
}

fn durable(line: &str) -> bool {
    extract_line(line).len() == 1
}

fn first_category(line: &str) -> String {
    extract_line(line).list()[0].category.clone()
}

// --- Mots-cles durables ------------------------------------------------------

#[test]
fn keyword_preference_pins_fact_with_category_preference() {
    let pf = extract_line("i prefer tea over coffee");
    assert_eq!(pf.len(), 1, "keyword 'prefer' => fait durable");
    assert_eq!(pf.list()[0].category, "preference");
    assert_eq!(pf.list()[0].content, "i prefer tea over coffee");
}

#[test]
fn non_durable_plain_text_is_never_pinned() {
    assert!(!durable("the weather is sunny today"));
}

// --- Categories du moteur ----------------------------------------------------

#[test]
fn containing_decision_yields_category_decision_even_mid_sentence() {
    // Tue `contains("decision") || starts_with("decision:")` -> `&&`
    // (l'entree contient mais ne demarre pas par le marqueur).
    assert_eq!(first_category("the decision was final"), "decision");
}

#[test]
fn keyword_must_yields_category_constraint() {
    assert_eq!(first_category("you must restart the server"), "constraint");
}

#[test]
fn keyword_always_yields_category_constraint() {
    assert_eq!(first_category("tests always run first"), "constraint");
}

#[test]
fn keyword_never_yields_category_constraint() {
    assert_eq!(first_category("never deploy on friday"), "constraint");
}

#[test]
fn keyword_only_project_is_category_project() {
    // Tue `contains("project") || contains("goal")` -> `&&`
    // (les deux branches sont testees separement).
    assert_eq!(first_category("the project kickoff is monday"), "project");
}

#[test]
fn keyword_only_goal_is_category_project_too() {
    assert_eq!(first_category("our goal is shipped"), "project");
}

#[test]
fn non_keyword_key_value_pair_defaults_to_category_fact() {
    // Bascule dans le cas `fact` (aucun mot-cle) via l'heuristique cle: valeur.
    assert_eq!(first_category("status: done"), "fact");
}

// --- Heuristique cle: valeur -------------------------------------------------

#[test]
fn key_value_pair_without_keyword_is_durable() {
    // Tue `is_durable -> false` (deja couvert par les tests mots-cles) et le
    // mutant `delete !` sur `!val.is_empty()`.
    assert!(durable("status: done"));
}

#[test]
fn empty_key_is_not_durable() {
    // Tue la bascule `&&` en `||` sur les deux premiers joints du
    // preambule (cle vide rendrait le mutant durable).
    assert!(!durable(": value"));
}

#[test]
fn key_up_to_40_chars_is_durable() {
    // Borne exacte 40 : tue `key.chars().count() <= 40` -> `>`.
    let key = "a".repeat(40);
    assert!(durable(&format!("{key}: v")));
}

#[test]
fn key_over_40_chars_is_not_durable() {
    // Au-dela de la borne : tue le mutant `&&` en `||` sur la derniere jointure.
    let key = "a".repeat(41);
    assert!(!durable(&format!("{key}: v")));
}

#[test]
fn value_up_to_200_chars_is_durable() {
    // Borne exacte 200 : tue `val.chars().count() <= 200` -> `>`.
    let val = "b".repeat(200);
    assert!(durable(&format!("k: {val}")));
}

#[test]
fn value_over_200_chars_is_not_durable() {
    assert!(!durable(&format!("k: {}", "b".repeat(201))));
}

#[test]
fn short_two_segment_key_is_durable() {
    // Cle a un seul caractere : tue `idx + 2` -> `-` (retablissement negatif).
    assert!(durable("a: bcd"));
}

#[test]
fn long_core_colon_value_is_durable() {
    // Tue `idx + 2` -> `*` (la valeur recalculee serait vide).
    assert!(durable("aaaaa: bcd"));
}

// --- Deduplication et comptage ----------------------------------------------

#[test]
fn identical_lines_are_extracted_once() {
    // Tue `if !seen.insert(...)` -> `if seen.insert(...)` sur les lignes en double.
    let pf = extract_line("i prefer tea\ni prefer tea");
    assert_eq!(pf.len(), 1);
}

#[test]
fn each_unique_durable_line_is_counted() {
    // Tue les mutants `counter += 1` -> `-= / *=` (ids ecrases ou blancs).
    let pf = extract_line("i prefer tea\nyou must restart the server");
    assert_eq!(pf.len(), 2);
}

#[test]
fn free_functions_extraction_return_extracted_facts() {
    let msgs = [Message::new(Role::User, "i prefer tea".to_string())];
    assert_eq!(omega_context::extract_pinned_facts(&msgs).len(), 1);
    assert_eq!(omega_context::extract_durable_facts(&msgs).len(), 1);
    assert_eq!(omega_context::extract_facts(&msgs).len(), 1);
}

#[test]
fn extraction_aliases_return_extracted_facts() {
    let msgs = [Message::new(Role::User, "i prefer tea".to_string())];
    assert_eq!(PinnedFacts::from_messages(&msgs).len(), 1);
    assert_eq!(PinnedFacts::extract_from(&msgs).len(), 1);
    assert_eq!(PinnedFacts::extract_from_messages(&msgs).len(), 1);
    assert_eq!(PinnedFacts::extract_durable(&msgs).len(), 1);
    assert_eq!(PinnedFacts::extract_durable_facts(&msgs).len(), 1);
}
