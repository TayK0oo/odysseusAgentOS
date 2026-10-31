//! Contrats comportementaux de `MemoryInjection` (module `injection.rs`).
//!
//! Cible : les 9 mutants survivants du fichier `injection.rs`
//! (`mutants.out-context-fresh`). Fixtures calibrées sur l'estimateur canonique
//! `ceil(chars/4)` (voir `token.rs`) et sans message Assistant pour éviter
//! `deja_compris`, qui réduirait le pouvoir discriminant des mutants `keywords`.
//!
//! Doubles de `MemoryStore` :
//! - `StaticMemory` : filtre par mots ≥ 4 caractères (copie des tests du module) ;
//! - `EchoStore` : retourne **tous** les faits, quelle que soit la requête ;
//! - `FailingStore` : `recall` retourne `Err(NoImplementation(MemoryStore))`.

use std::sync::Arc;

use async_trait::async_trait;
use omega_context::{ContextInjector, MemoryInjection};

use omega_ports::MemoryStore;
use omega_types::{Capability, Message, OmegaError, OmegaResult, Role};

fn user_msg(s: &str) -> Message {
    Message::new(Role::User, s.to_string())
}

// --- doubles ----------------------------------------------------------------

struct StaticMemory {
    facts: Vec<String>,
}
#[async_trait]
impl MemoryStore for StaticMemory {
    fn name(&self) -> &str {
        "static-test"
    }
    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        let q = query.to_lowercase();
        let hits: Vec<String> = self
            .facts
            .iter()
            .filter(|f| {
                q.split_whitespace()
                    .filter(|w| w.chars().count() >= 4)
                    .any(|w| f.to_lowercase().contains(w))
            })
            .take(limit)
            .cloned()
            .collect();
        Ok(hits)
    }
    async fn remember(&self, _fact: &str) -> OmegaResult<()> {
        Ok(())
    }
}

struct EchoStore {
    facts: Vec<String>,
}
#[async_trait]
impl MemoryStore for EchoStore {
    fn name(&self) -> &str {
        "echo-test"
    }
    async fn recall(&self, _query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        Ok(self.facts.iter().take(limit).cloned().collect())
    }
    async fn remember(&self, _fact: &str) -> OmegaResult<()> {
        Ok(())
    }
}

struct FailingStore;
#[async_trait]
impl MemoryStore for FailingStore {
    fn name(&self) -> &str {
        "failing-test"
    }
    async fn recall(&self, _query: &str, _limit: usize) -> OmegaResult<Vec<String>> {
        Err(OmegaError::NoImplementation(Capability::MemoryStore))
    }
    async fn remember(&self, _fact: &str) -> OmegaResult<()> {
        Ok(())
    }
}

fn static_inj(facts: &[&str]) -> MemoryInjection {
    MemoryInjection::new(Arc::new(StaticMemory {
        facts: facts.iter().map(|s| s.to_string()).collect(),
    }))
}

fn echo_inj(facts: &[&str]) -> MemoryInjection {
    MemoryInjection::new(Arc::new(EchoStore {
        facts: facts.iter().map(|s| s.to_string()).collect(),
    }))
}

// --- tue :98 ×2 (`name` -> "" et "xyzzy") ----------------------------------
#[tokio::test]
async fn injector_name_is_memory_injection() {
    let inj = static_inj(&[]);
    assert_eq!(inj.name(), "memory-injection");
}

// --- tue :43 (`is_empty` -> true) ------------------------------------------
#[tokio::test]
async fn is_empty_tracks_presence_of_injected_lines() {
    let inj = static_inj(&["OSAKA: le run budget se lit via RunBudget.usage()"]);
    let msgs = vec![user_msg("explique le run budget d'OSAKA stp")];
    let (_out, trace) = inj.inject(msgs, 50).await.unwrap();
    assert!(
        !trace.is_empty(),
        "mutation `is_empty -> true` rendrait vide"
    );
    assert_eq!(trace.lines.len(), 1);
}

// --- tue :81 (`journal` -> vec![Default::default()]) -------------------------
#[tokio::test]
async fn journal_records_distinct_traces_per_inject() {
    // Deux injections : la premiere injecte, la seconde est couverte par un
    // message Assistant (deja compris) -> trace vide. Le journal doit garder
    // les DEUX traces, distinctes.
    let inj = static_inj(&["OSAKA: le run budget se lit via RunBudget.usage()"]);
    let msgs1 = vec![user_msg("explique le run budget d'OSAKA stp")];
    inj.inject(msgs1, 50).await.unwrap();
    let deja = vec![
        user_msg("explique le run budget d'OSAKA stp"),
        Message::new(Role::Assistant, "OSAKA budget: cf doc"),
    ];
    inj.inject(deja, 50).await.unwrap();

    let j = inj.journal();
    assert_eq!(
        j.len(),
        2,
        "mutation `journal -> vec![Default::default()]` rendrait 1"
    );
    assert_ne!(j[0], j[1], "les deux traces ne sont pas identiques");
}

// --- tue :86 (`keywords` -> vec![String::new()]) ----------------------------
#[tokio::test]
async fn keywords_shorter_than_four_chars_do_not_trigger_injection() {
    // Query dont TOUS les mots font < 4 chars. Originel : keywords vide -> pas
    // d'injection. Mutant : keywords [""] -> entre, rappelle, fact.contains("")
    // = true -> injecte.
    // EchoStore retourne le fait quoi qu'il arrive ; pas de message Assistant
    // (sinon deja_compris couvrirait tout).
    let inj = echo_inj(&["a fact that should never be injected"]);
    let msgs = vec![user_msg("aa bb")];
    let (out, trace) = inj.inject(msgs, 50).await.unwrap();
    assert!(
        trace.is_empty(),
        "mutation `keywords -> [\"\"]` injecterait"
    );
    assert_eq!(out.len(), 1);
}

// --- tue :116 ×2 (`&&` -> `||` et `>` -> `>=`) ------------------------------
#[tokio::test]
async fn budget_zero_prevents_recall_and_injection() {
    // FailingStore : si le garde laissait entrer, inject retournerait Err.
    // Originel : `!keywords.is_empty() && budget_tokens > 0` est faux avec
    // budget 0 -> pas d'appel recall -> Ok + trace vide.
    // Mutant `&& -> ||` : entre (keywords non vides) -> Err.
    // Mutant `> -> >=` : `budget_tokens >= 0` vrai -> entre -> Err.
    let inj = MemoryInjection::new(Arc::new(FailingStore));
    let msgs = vec![user_msg("hello world")];
    let result = inj.inject(msgs, 0).await;
    match result {
        Ok((out, trace)) => {
            assert_eq!(out.len(), 1);
            assert!(trace.is_empty());
        }
        Err(_) => panic!("le mutant laisserait entrer recall et propagerait l'erreur"),
    }
}

// --- tue :142 ×2 (`>` -> `>=` et `+` -> `*`) -------------------------------
#[tokio::test]
async fn budget_arithmetic_admits_exactly_two_facts_of_three_tokens() {
    // 3 faits de 3 tokens chacun (10-12 chars) ; budget = 6.
    // Originel (`+`, `>`): 0+3<=6 accept ; 3+3<=6 accept ; 6+3>6 break.
    //   -> 2 faits, tokens_used = 6.
    // Mutant `+ -> *` : 0*3<=6 ; 3*3=9>6 break -> 1 fait, tokens_used=0.
    // Mutant `> -> >=` : 0+3>=6 F accept ; 3+3>=6 T break -> 1 fait.
    let facts = ["fact one A", "fact two B", "fact thr C"];
    let inj = echo_inj(&facts);
    let msgs = vec![user_msg("common fact")];
    let (out, trace) = inj.inject(msgs, 6).await.unwrap();
    assert_eq!(
        trace.lines.len(),
        2,
        "mutation +/* ou >/>= changerait la sélection"
    );
    assert_eq!(trace.tokens_used, 6);
    assert_eq!(out.len(), 2); // bloc System inséré
}
