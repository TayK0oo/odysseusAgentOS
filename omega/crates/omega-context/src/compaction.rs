//! Compaction deterministe (sans LLM) : tronque les plus anciens messages et insere
//! un marqueur de synthese structuree.

use omega_types::{Message, Role};

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
/// superieur a `systeme + marqueur` ; sinon le systeme (intouchable) est preserve tel quel.
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

#[cfg(test)]
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
        // 10 tokens chacun ; total 60, budget 38 -> systeme + marqueur + 2 recents.
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
}
