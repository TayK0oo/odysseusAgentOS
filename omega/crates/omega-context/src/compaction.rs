//! Compaction deterministe (sans LLM) : tronque les plus anciens messages et insere
//! un marqueur de synthese structuree.

use omega_types::{Message, Role};

use crate::pinned_facts::PinnedFacts;
use crate::token::TokenEstimator;

/// Marqueur de compaction : `k` = nombre de messages omis.
pub fn compaction_marker(omitted: usize) -> String {
    format!("…[compacted {omitted} messages omis]…")
}

/// Compacte une conversation sous un budget de tokens :
/// - preserve le prefixe `System` (prompt) ;
/// - garde les messages les plus recents (suffixe maximal) ;
/// - insere un marqueur `…[compacted k messages omis]…` si `k > 0`.
///
/// Deterministe et sans LLM (RULES-OMEGA §3.2). La garantie de budget vaut pour un budget
/// superieur a `systeme + marqueur` ; sinon le systeme (intouchable) est preserve tel quel,
/// **sans marqueur mensonger** (aucun marqueur n'est insere quand il ne tient pas).
pub fn truncate_compact(messages: &[Message], budget: usize) -> Vec<Message> {
    let estimator = TokenEstimator::new();
    if estimator.estimate_messages(messages) <= budget {
        return messages.to_vec();
    }

    let system_count = messages
        .iter()
        .take_while(|m| m.role == Role::System)
        .count();
    let system = &messages[..system_count];
    let tail = &messages[system_count..];
    let system_tokens = estimator.estimate_messages(system);

    // Reserve la place du marqueur (borne superieure = pire cas : omitted = tail.len()).
    let marker_reserve = estimator.estimate(&compaction_marker(tail.len()));

    // Budget trop etroit pour le systeme + marqueur : pas de marqueur mensonger, on
    // preserve le systeme tel quel (primaute du systeme, W6 §4.6). Le depassement est
    // assume et documente (le systeme est intouchable).
    if system_tokens + marker_reserve > budget {
        return system.to_vec();
    }

    // Suffixe maximal des messages recents tenant dans le budget restant.
    let mut kept: Vec<Message> = Vec::new();
    let mut kept_tokens = system_tokens + marker_reserve;
    for msg in tail.iter().rev() {
        let t = estimator.estimate(&msg.content);
        if kept_tokens + t <= budget {
            kept_tokens += t;
            kept.push(msg.clone());
        } else {
            break;
        }
    }
    kept.reverse();
    let omitted = tail.len() - kept.len();

    let mut result: Vec<Message> = system.to_vec();
    if omitted > 0 {
        result.push(Message::new(Role::System, compaction_marker(omitted)));
    }
    result.extend(kept);
    result
}


// --- Double-layer LLM compaction (NOYAU Phase C) ------------------------------

use async_trait::async_trait;
use omega_ports::{ContextEngine, Provider};
use omega_types::OmegaResult;
use std::sync::Arc;

/// Moteur de compaction LLM double couche :
/// 1. Couche 1 (truncation deterministe) : `truncate_compact` ci-dessus, reserve un budget
///    pour le resume LLM.
/// 2. Couche 2 (LLM) : appelle un `Provider` pour resumer les messages omis en un seul
///    message systeme `Summary: ...`.
///
/// Si le provider echoue ou n'est pas dispo, on replie sur la couche 1 seulement
/// (fail-closed : absence de provider = truncation simple).
pub struct LlmCompactionEngine {
    inner: Arc<dyn ContextEngine>,
    provider: Arc<dyn Provider>,
    summary_budget_ratio: f32,
}

impl LlmCompactionEngine {
    pub fn new(inner: Arc<dyn ContextEngine>, provider: Arc<dyn Provider>) -> Self {
        Self {
            inner,
            provider,
            summary_budget_ratio: 0.25,
        }
    }

    pub fn with_summary_budget(mut self, ratio: f32) -> Self {
        self.summary_budget_ratio = ratio.clamp(0.1, 0.5);
        self
    }
}

#[async_trait]
impl ContextEngine for LlmCompactionEngine {
    fn name(&self) -> &str {
        "llm-compaction"
    }

    fn owns_compaction(&self) -> bool {
        true
    }

    fn max_tokens(&self) -> usize {
        self.inner.max_tokens()
    }

    async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
        let max_tokens = self.inner.max_tokens();
        let estimator = TokenEstimator::new();

        // 1. Tentative avec truncation simple (couche 1)
        let truncated = truncate_compact(&messages, max_tokens);
        let dropped = {
            // Compter les messages non-systeme originaux vs resultats
            let orig_non_system = messages.iter().filter(|m| m.role != Role::System).count();
            let result_non_system = truncated
                .iter()
                .filter(|m| m.role != Role::System && !m.content.starts_with("…[compacted"))
                .count();
            orig_non_system.saturating_sub(result_non_system)
        };

        // Si pas de messages omis, retour direct
        if dropped == 0 {
            return Ok(truncated);
        }

        // 2. Construire le resume LLM des messages omis
        let system_count = messages
            .iter()
            .take_while(|m| m.role == Role::System)
            .count();
        let non_system = &messages[system_count..];

        // Les messages non-systeme conserves (sans le marqueur)
        let kept_non_system = truncated
            .iter()
            .filter(|m| m.role != Role::System && !m.content.starts_with("…[compacted"))
            .count();
        let omitted_count = non_system.len().saturating_sub(kept_non_system);
        let omitted_messages = &non_system[..omitted_count.min(non_system.len())];

        let summary = self.summarize_omitted(omitted_messages).await;

        // 3. Inserer le resume dans la liste tronquee
        let mut result = truncated;
        if let Some(summary_text) = summary {
            let summary_msg = Message::new(Role::System, format!("Summary: {summary_text}"));
            let system_count = result.iter().take_while(|m| m.role == Role::System).count();
            result.insert(system_count, summary_msg);
        }

        // 4. Re-verifier le budget
        if estimator.estimate_messages(&result) > max_tokens {
            return Ok(truncate_compact(&result, max_tokens));
        }

        Ok(result)
    }
}

impl LlmCompactionEngine {
    async fn summarize_omitted(&self, messages: &[Message]) -> Option<String> {
        if messages.is_empty() {
            return None;
        }
        let content: String = messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Resume les messages suivants en 1-2 phrases maximum, en conservant les faits cles, decisions et questions ouvertes:\n\n{}",
            content
        );
        let _prompt_len = prompt.len();

        let req = omega_types::ChatRequest {
            model: omega_types::ModelId::new("auto"),
            messages: vec![Message::new(Role::User, prompt.clone())],
        };

        match self.provider.complete(req).await {
            Ok(resp) => Some(resp.message.content),
            Err(_e) => None,
        }
    }
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod llm_compaction_tests {
    use super::*;
    use crate::engine::DefaultContextEngine;
    use omega_types::{Message, Role};

    struct EchoProvider;
    #[async_trait]
    impl Provider for EchoProvider {
        fn id(&self) -> &str {
            "echo"
        }
        async fn complete(
            &self,
            req: omega_types::ChatRequest,
        ) -> OmegaResult<omega_types::ChatResponse> {
            let last = req
                .messages
                .last()
                .map(|m| m.content.clone())
                .unwrap_or_default();
            Ok(omega_types::ChatResponse {
                message: Message::new(Role::Assistant, format!("SUMMARY: {}", last)),
                usage: Default::default(),
            })
        }
    }

    struct FailProvider;
    #[async_trait]
    impl Provider for FailProvider {
        fn id(&self) -> &str {
            "fail"
        }
        async fn complete(
            &self,
            _: omega_types::ChatRequest,
        ) -> OmegaResult<omega_types::ChatResponse> {
            Err(omega_types::OmegaError::NoImplementation(
                omega_types::Capability::LlmProvider,
            ))
        }
    }

    #[tokio::test]
    async fn llm_compaction_falls_back_on_provider_failure() {
        let inner = Arc::new(DefaultContextEngine::new(20));
        let provider = Arc::new(FailProvider);
        let engine = LlmCompactionEngine::new(inner, provider);

        let msgs = vec![
            Message::new(Role::System, "SYS"),
            Message::new(Role::User, "a".repeat(40)),
            Message::new(Role::User, "b".repeat(40)),
        ];
        let out = engine.assemble(msgs).await.unwrap();
        // Doit tomber sur truncation simple (pas de resume)
        assert!(out.iter().any(|m| m.content.starts_with("…[compacted")));
    }

    #[tokio::test]
    async fn llm_compaction_adds_summary_when_provider_works() {
        let inner = Arc::new(DefaultContextEngine::new(20));
        let provider = Arc::new(EchoProvider);
        let engine = LlmCompactionEngine::new(inner, provider);

        let msgs = vec![
            Message::new(Role::System, "SYS"),
            Message::new(Role::User, "a".repeat(40)),
            Message::new(Role::User, "b".repeat(40)),
        ];
        let out = engine.assemble(msgs).await.unwrap();
        // Doit contenir le resume LLM
        assert!(out
            .iter()
            .any(|m| m.content.starts_with("Summary: SUMMARY:")));
    }
}

/// Compacte une conversation sous budget en **préservant les faits épinglés**.
///
/// Les faits épinglés (via `PinnedFacts`) sont injectés après le préfixe système
/// et avant le marqueur de compaction. Ils ne sont jamais supprimés.
///
/// Algorithme :
/// 1. Sépare le préfixe système (rôles `System` initiaux)
/// 2. Calcule le coût des faits épinglés + système
/// 3. Réserve la place pour le marqueur de compaction
/// 4. Garde le suffixe maximal des messages récents non-épinglés
/// 5. Insère le marqueur si des messages ont été omis
pub fn truncate_compact_with_pinned(
    messages: &[Message],
    budget: usize,
    pinned: &PinnedFacts,
) -> Vec<Message> {
    let estimator = TokenEstimator::new();

    // 1. Préfixe système (intouchable)
    let system: Vec<Message> = messages
        .iter()
        .take_while(|m| m.role == Role::System)
        .cloned()
        .collect();

    // 2. Messages non-système
    let non_system = &messages[system.len()..];

    // 3. Faits épinglés -> messages système
    let pinned_msgs = pinned.to_system_messages();

    // 4. Combine système + épinglés
    let mut preserved_prefix = system;
    preserved_prefix.extend(pinned_msgs);
    let prefix_tokens = estimator.estimate_messages(&preserved_prefix);

    // 5. Réserve place pour le marqueur (pire cas : tout le non-système omis)
    let marker_reserve = estimator.estimate(&compaction_marker(non_system.len()));

    // 6. Si tout tient dans le budget (messages originaux + faits épinglés), on retourne
    //    le tout avec les faits épinglés injectés (pas de compaction nécessaire).
    let original_with_pinned = {
        let mut v = preserved_prefix.clone();
        v.extend(non_system.iter().cloned());
        v
    };
    if estimator.estimate_messages(&original_with_pinned) <= budget {
        return original_with_pinned;
    }

    // 7. Budget trop étroit pour tout garder : garde seulement le préfixe (pas de marqueur mensonger)
    if prefix_tokens + marker_reserve > budget {
        return preserved_prefix;
    }

    // 8. Suffixe maximal des messages récents non-épinglés
    let mut kept: Vec<Message> = Vec::new();
    let mut kept_tokens = prefix_tokens + marker_reserve;
    for msg in non_system.iter().rev() {
        let t = estimator.estimate(&msg.content);
        if kept_tokens + t <= budget {
            kept_tokens += t;
            kept.push(msg.clone());
        } else {
            break;
        }
    }
    kept.reverse();
    let omitted = non_system.len() - kept.len();

    // 9. Reconstitue : préfixe + marqueur (si omis) + suffixe gardé
    let mut result = preserved_prefix;
    if omitted > 0 {
        result.push(Message::new(Role::System, compaction_marker(omitted)));
    }
    result.extend(kept);
    result
}

#[cfg(test)]
mod pinned_compaction_tests {
    use super::*;
    use crate::pinned_facts::PinnedFact;
    use omega_types::{Message, Role};

    #[test]
    fn truncate_compact_with_pinned_preserves_pinned_facts() {
        let system = Message::new(Role::System, "SYS");
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let u3 = Message::new(Role::User, "c".repeat(40));
        let msgs = vec![system.clone(), u1, u2, u3];

        let pinned = PinnedFacts::new();
        pinned.upsert(PinnedFact::new(
            "pref1",
            "User prefers dark mode",
            "preference",
        ));

        let out = truncate_compact_with_pinned(&msgs, 25, &pinned);

        // Le fait épinglé doit être présent
        assert!(out
            .iter()
            .any(|m| m.content.contains("User prefers dark mode")));
        // Le système préservé
        assert_eq!(out[0], system);
        // Marqueur de compaction présent (3 messages non-système, 2 gardés max)
        assert!(out.iter().any(|m| m.content.starts_with("…[compacted")));
    }

    #[test]
    fn truncate_compact_with_pinned_multiple_facts_sorted_by_priority() {
        let system = Message::new(Role::System, "SYS");
        let u1 = Message::new(Role::User, "x".repeat(20));
        let msgs = vec![system.clone(), u1];

        let pinned = PinnedFacts::new();
        pinned.upsert(PinnedFact::new("low", "low priority", "preference").with_priority(0));
        pinned.upsert(PinnedFact::new("high", "high priority", "decision").with_priority(10));

        let out = truncate_compact_with_pinned(&msgs, 30, &pinned);

        // Les deux faits doivent être présents, triés par priorité
        let pinned_contents: Vec<_> = out
            .iter()
            .filter(|m| m.content.starts_with("[PinnedFact:"))
            .map(|m| m.content.clone())
            .collect();
        assert_eq!(pinned_contents.len(), 2);
        // high priority en premier
        assert!(pinned_contents[0].contains("high priority"));
        assert!(pinned_contents[1].contains("low priority"));
    }

    #[test]
    fn truncate_compact_with_pinned_budget_too_tight_keeps_system_and_pinned() {
        let system = Message::new(Role::System, "S".repeat(100)); // ~25 tokens
        let u1 = Message::new(Role::User, "u".repeat(100));
        let msgs = vec![system.clone(), u1];

        let pinned = PinnedFacts::new();
        pinned.upsert(PinnedFact::new("fact1", "critical fact", "constraint").with_priority(100));

        // Budget très serré : système + épinglé + marqueur > budget
        let out = truncate_compact_with_pinned(&msgs, 15, &pinned);

        // Système préservé (primauté du système)
        assert_eq!(out[0], system);
        // Fait épinglé préservé aussi
        assert!(out.iter().any(|m| m.content.contains("critical fact")));
        // Pas de marqueur mensonger
        assert!(out.iter().all(|m| !m.content.starts_with("…[compacted")));
    }

    #[test]
    fn truncate_compact_with_pinned_deterministic() {
        let system = Message::new(Role::System, "SYS");
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let msgs = vec![system, u1, u2];

        let pinned = PinnedFacts::new();
        pinned.upsert(PinnedFact::new("z", "z fact", "cat"));
        pinned.upsert(PinnedFact::new("a", "a fact", "cat"));

        let out1 = truncate_compact_with_pinned(&msgs, 25, &pinned);
        let out2 = truncate_compact_with_pinned(&msgs, 25, &pinned);
        assert_eq!(out1, out2);
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use omega_types::{Message, Role};

    #[test]
    fn under_budget_is_unchanged() {
        let msgs = vec![
            Message::new(Role::System, "SYS"),
            Message::new(Role::User, "u1"),
        ];
        let out = truncate_compact(&msgs, 100);
        assert_eq!(out, msgs);
    }

    #[test]
    fn over_budget_keeps_system_recent_and_marks_omitted() {
        let system = Message::new(Role::System, "s".repeat(40));
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let u3 = Message::new(Role::User, "c".repeat(40));
        let u4 = Message::new(Role::User, "d".repeat(40));
        let u5 = Message::new(Role::User, "e".repeat(40));
        let msgs = vec![system.clone(), u1, u2, u3, u4.clone(), u5.clone()];

        let out = truncate_compact(&msgs, 38);

        assert_eq!(out.len(), 4);
        assert_eq!(out[0], system);
        assert_eq!(
            out[1],
            Message::new(Role::System, "…[compacted 3 messages omis]…")
        );
        assert_eq!(out[2], u4);
        assert_eq!(out[3], u5);
    }

    #[test]
    fn compacted_result_respects_budget() {
        let msgs: Vec<Message> = (0..20)
            .map(|i| Message::new(Role::User, format!("message numero {i}")))
            .collect();
        let out = truncate_compact(&msgs, 24);
        let estimator = TokenEstimator::new();
        assert!(estimator.estimate_messages(&out) <= 24);
        assert!(out.iter().any(|m| m.content.starts_with("…[compacted")));
    }

    #[test]
    fn compaction_is_deterministic() {
        let msgs = vec![
            Message::new(Role::System, "s".repeat(40)),
            Message::new(Role::User, "a".repeat(40)),
            Message::new(Role::User, "b".repeat(40)),
            Message::new(Role::User, "c".repeat(40)),
            Message::new(Role::User, "d".repeat(40)),
        ];
        let a = truncate_compact(&msgs, 28);
        let b = truncate_compact(&msgs, 28);
        assert_eq!(a, b);
    }

    #[test]
    fn system_alone_over_budget_preserves_system_without_lying_marker() {
        let system = Message::new(Role::System, "s".repeat(40));
        let user = Message::new(Role::User, "u".repeat(8));
        let out = truncate_compact(&[system.clone(), user], 5);

        assert_eq!(out, vec![system]);
        assert!(out.iter().all(|m| !m.content.starts_with("…[compacted")));
    }
}
