//! Contrats comportementaux de `LlmCompactionEngine` (module `compaction.rs`).
//!
//! Cible : les mutants survivants de la baseline fresh sur `LlmCompactionEngine`.
//! Les 3 mutants `:166` (`>` -> `==` / `<` / `>=`) sont **équipotents** :
//! `result` ne contient que des messages `System` (préfixe + marqueur + résumé),
//! donc `truncate_compact(&result, max_tokens)` retourne `result` inchangé
//! (early-return `est(all) <= budget` puis garde `system_tokens + marker > budget`
//! qui rend le système tel quel). Aucun test comportemental ne les tue —
//! registre Livrable D.

use std::sync::Arc;

use async_trait::async_trait;
use omega_context::{DefaultContextEngine, LlmCompactionEngine};
use omega_ports::{ContextEngine, Provider};
use omega_types::{ChatRequest, ChatResponse, Message, OmegaError, OmegaResult, Role, Usage};

struct EchoProvider;
#[async_trait]
impl Provider for EchoProvider {
    fn id(&self) -> &str {
        "echo"
    }
    async fn complete(&self, req: ChatRequest) -> OmegaResult<ChatResponse> {
        let last = req
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();
        Ok(ChatResponse {
            message: Message::new(Role::Assistant, format!("SUMMARY: {}", last)),
            usage: Usage {
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cache_write_tokens: 0,
            },
        })
    }
}

struct FailProvider;
#[async_trait]
impl Provider for FailProvider {
    fn id(&self) -> &str {
        "fail"
    }
    async fn complete(&self, _req: ChatRequest) -> OmegaResult<ChatResponse> {
        Err(OmegaError::NoImplementation(
            omega_types::Capability::LlmProvider,
        ))
    }
}

fn engine(inner: DefaultContextEngine, provider: Arc<dyn Provider>) -> LlmCompactionEngine {
    LlmCompactionEngine::new(Arc::new(inner), provider)
}

// --- tue :108 ×2 (`name` -> "" et "xyzzy") -------------------------------
#[tokio::test]
async fn name_returns_llm_compaction() {
    let e = engine(DefaultContextEngine::new(20), Arc::new(EchoProvider));
    assert_eq!(e.name(), "llm-compaction");
}

// --- tue :112 (`owns_compaction` -> false) ---------------------------------
#[tokio::test]
async fn owns_compaction_is_true() {
    let e = engine(DefaultContextEngine::new(20), Arc::new(EchoProvider));
    assert!(e.owns_compaction());
}

// --- tue :116 ×2 (`max_tokens` -> 0 et 1) ---------------------------------
#[tokio::test]
async fn max_tokens_delegates_to_inner() {
    let e = engine(DefaultContextEngine::new(42), Arc::new(EchoProvider));
    assert_eq!(e.max_tokens(), 42);
}

// --- tue :130 ×2 (`!=` -> `==` et delete `!`) ----------------------------
// Fixture : 2 préfixes System + 1 User omis. Original : dropped = 1 (> 0) ->
// résumé présent. Les deux mutants rendent dropped = 0 -> pas de résumé.
#[tokio::test]
async fn assemble_inserts_summary_when_messages_are_dropped() {
    let inner = DefaultContextEngine::new(10);
    let e = engine(inner, Arc::new(EchoProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS1"),
        Message::new(Role::System, "SYS2"),
        Message::new(Role::User, "a".repeat(33)),
    ];
    let out = e.assemble(msgs).await.unwrap();
    assert!(out.iter().any(|m| m.content.starts_with("Summary:")));
}

// --- tue :143 (`==` -> `!=` sur system_count) -----------------------------
// Mutant : system_count = 0 -> le résumé est inséré à l'index 0, avant le
// préfixe System. L'original le place après le préfixe.
#[tokio::test]
async fn assemble_keeps_system_prefix_before_summary() {
    let inner = DefaultContextEngine::new(10);
    let e = engine(inner, Arc::new(EchoProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS1"),
        Message::new(Role::System, "SYS2"),
        Message::new(Role::User, "a".repeat(33)),
    ];
    let out = e.assemble(msgs).await.unwrap();
    assert_eq!(out[0].content, "SYS1");
    assert!(out.iter().any(|m| m.content.starts_with("Summary:")));
    let summary = out
        .iter()
        .find(|m| m.content.starts_with("Summary:"))
        .unwrap();
    assert!(
        !summary.content.contains("SYS1"),
        "le mutant :143 inclurait le préfixe dans le résumé"
    );
}

// --- tue :150 ×2 (`!=` -> `==` et delete `!`) ----------------------------
// Fixture : 2 préfixes System + 3 User dont 1 survit à la troncature.
// Original : dropped = 2 (kept = 1) -> résumé sur 2 messages (a, b) ;
//   le résumé ne contient pas le message survivant (c).
// Mutant `!=` -> `==` : kept_non_system = 1 (compte les System) ->
//   omitted = 3 - 1 = 2 (identique) -> passe (pas tué par ce test seul).
// Mutant delete `!` : kept_non_system = 0 (aucun marqueur non-System) ->
//   omitted = 3 -> résumé sur 3 messages (a, b, c) -> contient "ccc".
#[tokio::test]
async fn assemble_summary_omits_only_dropped_messages() {
    let inner = DefaultContextEngine::new(19);
    let e = engine(inner, Arc::new(EchoProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS1"),
        Message::new(Role::System, "SYS2"),
        Message::new(Role::User, "a".repeat(33)),
        Message::new(Role::User, "b".repeat(33)),
        Message::new(Role::User, "c".repeat(33)),
    ];
    let out = e.assemble(msgs).await.unwrap();
    let summary = out
        .iter()
        .find(|m| m.content.starts_with("Summary:"))
        .expect("un résumé doit être inséré");
    assert!(
        summary.content.contains("aaa"),
        "le résumé doit contenir le 1er omis"
    );
    assert!(
        !summary.content.contains("ccc"),
        "le message survivant ne doit pas figurer dans le résumé"
    );
}

// --- tue :161 (`==` -> `!=` sur system_count dans result) -----------------
// Même fixture que le test précédent : le mutant insère le résumé à l'index 0.
#[tokio::test]
async fn assemble_summary_inserted_after_system_prefix_not_before() {
    let inner = DefaultContextEngine::new(19);
    let e = engine(inner, Arc::new(EchoProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS1"),
        Message::new(Role::System, "SYS2"),
        Message::new(Role::User, "a".repeat(33)),
        Message::new(Role::User, "b".repeat(33)),
        Message::new(Role::User, "c".repeat(33)),
    ];
    let out = e.assemble(msgs).await.unwrap();
    assert_eq!(out[0].content, "SYS1");
    assert!(out.iter().any(|m| m.content.starts_with("Summary:")));
}

// --- tue :130:55 (delete `!`) ---------------------------------------------
// Fixture sans troncature (budget large) : original `dropped = 0` -> pas de
// résumé. Le mutant compte les marqueurs (rôle System, donc 0) au lieu de les
// exclure -> `dropped > 0` -> insère un résumé.
#[tokio::test]
async fn assemble_no_summary_when_nothing_dropped() {
    let inner = DefaultContextEngine::new(1000);
    let e = engine(inner, Arc::new(EchoProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS"),
        Message::new(Role::User, "hi"),
    ];
    let out = e.assemble(msgs).await.unwrap();
    assert!(!out.iter().any(|m| m.content.starts_with("Summary:")));
}

// --- défensif : fallback sur échec provider -------------------------------
#[tokio::test]
async fn assemble_falls_back_when_provider_fails() {
    let inner = DefaultContextEngine::new(20);
    let e = engine(inner, Arc::new(FailProvider));
    let msgs = vec![
        Message::new(Role::System, "SYS"),
        Message::new(Role::User, "a".repeat(40)),
        Message::new(Role::User, "b".repeat(40)),
    ];
    let out = e.assemble(msgs).await.unwrap();
    assert!(out.iter().any(|m| m.content.starts_with("…[compacted")));
    assert!(!out.iter().any(|m| m.content.starts_with("Summary:")));
}
