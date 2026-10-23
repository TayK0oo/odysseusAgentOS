//! Faits épinglés (pinned facts) — faits durables qui survivent à la compaction.
//!
//! Un fait épinglé est un message marqué comme `pinned = true` qui ne doit jamais être
// supprimé lors de la compaction (truncation ou LLM). Cela permet de préserver des
// informations critiques : décisions, clés API, identifiants de session, préférences utilisateur,
// etc.

use omega_types::{Message, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// Marqueur pour un message épinglé : préfixe ajouté au contenu pour identification.
/// Ce n'est pas un rôle — c'est un attribut du message (via metadata ou convention).
pub const PINNED_PREFIX: &str = "[PINNED] ";

/// Vérifie si un message est épinglé (contient le marqueur).
pub fn is_pinned(msg: &Message) -> bool {
    msg.content.starts_with(PINNED_PREFIX)
}

/// Extrait le contenu réel d'un message épinglé (sans le préfixe).
pub fn unpin_content(msg: &Message) -> &str {
    msg.content.strip_prefix(PINNED_PREFIX).unwrap_or(&msg.content)
}

/// Crée un message système épinglé.
pub fn pinned_system(content: impl Into<String>) -> Message {
    Message::new(Role::System, format!("{}{}", PINNED_PREFIX, content.into()))
}

/// Crée un message utilisateur épinglé.
pub fn pinned_user(content: impl Into<String>) -> Message {
    Message::new(Role::User, format!("{}{}", PINNED_PREFIX, content.into()))
}

/// Crée un message assistant épinglé.
pub fn pinned_assistant(content: impl Into<String>) -> Message {
    Message::new(Role::Assistant, format!("{}{}", PINNED_PREFIX, content.into()))
}

/// Ensemble de faits épinglés géré par le moteur de contexte.
///
/// Les faits épinglés sont conservés **toujours** lors de la compaction, au-dessus du
/// budget normal. Ils sont insérés après le préfixe système et avant les messages récents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinnedFacts {
    facts: HashSet<String>,
}

impl PinnedFacts {
    /// Crée un ensemble vide de faits épinglés.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un fait épinglé (idempotent : doublons ignorés).
    pub fn pin(&mut self, fact: impl Into<String>) {
        self.facts.insert(fact.into());
    }

    /// Retire un fait épinglé.
    pub fn unpin(&mut self, fact: &str) -> bool {
        self.facts.remove(fact)
    }

    /// Vérifie si un fait est épinglé.
    pub fn contains(&self, fact: &str) -> bool {
        self.facts.contains(fact)
    }

    /// Retourne tous les faits épinglés (ordre déterministe : tri alphabétique).
    pub fn all(&self) -> Vec<String> {
        let mut v: Vec<String> = self.facts.iter().cloned().collect();
        v.sort();
        v
    }

    /// Nombre de faits épinglés.
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Vrai si aucun fait épinglé.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Convertit en messages système épinglés pour injection dans le contexte.
    pub fn to_messages(&self) -> Vec<Message> {
        self.all()
            .into_iter()
            .map(pinned_system)
            .collect()
    }
}

/// Extension de `truncate_compact` qui respecte les faits épinglés.
///
/// Les messages épinglés sont traités comme le préfixe système : ils sont **toujours**
/// préservés, insérés après le système et avant le marqueur de compaction.
pub fn truncate_compact_with_pinned(
    messages: &[Message],
    budget: usize,
    pinned: &PinnedFacts,
) -> Vec<Message> {
    use crate::token::TokenEstimator;

    let estimator = TokenEstimator::new();
    if estimator.estimate_messages(messages) <= budget {
        return messages.to_vec();
    }

    // Séparer : système + épinglés (depuis messages) + reste
    let system: Vec<Message> = messages
        .iter()
        .take_while(|m| m.role == Role::System)
        .cloned()
        .collect();

    let non_system = &messages[system.len()..];

    // Extraire les messages épinglés du non-système (ils peuvent être n'importe où)
    let (pinned_msgs, other_msgs): (Vec<Message>, Vec<Message>) = non_system
        .iter()
        .cloned()
        .partition(|m| is_pinned(m));

    // Aussi ajouter les faits épinglés gérés explicitement
    let explicit_pinned = pinned.to_messages();

    // Combiner : système + épinglés explicites + épinglés du contexte
    let mut preserved_prefix = system;
    preserved_prefix.extend(explicit_pinned);
    preserved_prefix.extend(pinned_msgs);

    let prefix_tokens = estimator.estimate_messages(&preserved_prefix);

    // Réserver place pour le marqueur
    let marker_reserve = estimator.estimate(&crate::compaction::compaction_marker(other_msgs.len()));

    // Si préfixe + marqueur > budget : on garde seulement le préfixe (pas de marqueur mensonger)
    if prefix_tokens + marker_reserve > budget {
        return preserved_prefix;
    }

    // Suffixe maximal des messages récents non-épinglés
    let mut kept: Vec<Message> = Vec::new();
    let mut kept_tokens = prefix_tokens + marker_reserve;
    for msg in other_msgs.iter().rev() {
        let t = estimator.estimate(&msg.content);
        if kept_tokens + t <= budget {
            kept_tokens += t;
            kept.push(msg.clone());
        } else {
            break;
        }
    }
    kept.reverse();
    let omitted = other_msgs.len() - kept.len();

    let mut result = preserved_prefix;
    if omitted > 0 {
        result.push(Message::new(
            Role::System,
            crate::compaction::compaction_marker(omitted),
        ));
    }
    result.extend(kept);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::{Message, Role};

    #[test]
    fn pinned_facts_new_is_empty() {
        let p = PinnedFacts::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn pinned_facts_pin_and_contains() {
        let mut p = PinnedFacts::new();
        p.pin("user_pref: dark_mode");
        assert!(p.contains("user_pref: dark_mode"));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn pinned_facts_idempotent() {
        let mut p = PinnedFacts::new();
        p.pin("fact1");
        p.pin("fact1");
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn pinned_facts_unpin() {
        let mut p = PinnedFacts::new();
        p.pin("fact1");
        assert!(p.unpin("fact1"));
        assert!(!p.contains("fact1"));
        assert!(!p.unpin("fact1")); // déjà retiré
    }

    #[test]
    fn pinned_facts_to_messages() {
        let mut p = PinnedFacts::new();
        p.pin("pref: dark");
        p.pin("session: abc123");
        let msgs = p.to_messages();
        assert_eq!(msgs.len(), 2);
        assert!(msgs[0].content.starts_with(PINNED_PREFIX));
        assert!(msgs[1].content.starts_with(PINNED_PREFIX));
    }

    #[test]
    fn is_pinned_detects_marker() {
        let msg = pinned_system("important fact");
        assert!(is_pinned(&msg));
        let normal = Message::new(Role::System, "normal");
        assert!(!is_pinned(&normal));
    }

    #[test]
    fn unpin_content_strips_prefix() {
        let msg = pinned_user("secret key");
        assert_eq!(unpin_content(&msg), "secret key");
    }

    #[test]
    fn truncate_compact_with_pinned_preserves_pinned() {
        let system = Message::new(Role::System, "SYS");
        let pinned_msg = pinned_user("PINNED FACT");
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let u3 = Message::new(Role::User, "c".repeat(40));
        let msgs = vec![system.clone(), pinned_msg.clone(), u1, u2, u3];

        let pinned = PinnedFacts::new();
        let out = truncate_compact_with_pinned(&msgs, 30, &pinned);

        // Le message épinglé doit être préservé
        assert!(out.iter().any(|m| is_pinned(m)));
        // Le système aussi
        assert_eq!(out[0], system);
    }

    #[test]
    fn truncate_compact_with_pinned_explicit_facts() {
        let system = Message::new(Role::System, "SYS");
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let msgs = vec![system.clone(), u1, u2];

        let pinned = PinnedFacts::new();
        pinned.pin("explicit: important");
        let out = truncate_compact_with_pinned(&msgs, 25, &pinned);

        // Le fait explicite doit être inséré après le système
        assert!(out.iter().any(|m| m.content.contains("explicit: important")));
        assert_eq!(out[0], system);
    }

    #[test]
    fn truncate_compact_with_pinned_deterministic() {
        let system = Message::new(Role::System, "SYS");
        let u1 = Message::new(Role::User, "a".repeat(40));
        let u2 = Message::new(Role::User, "b".repeat(40));
        let msgs = vec![system, u1, u2];

        let pinned = PinnedFacts::new();
        pinned.pin("z_fact");
        pinned.pin("a_fact");

        let a = truncate_compact_with_pinned(&msgs, 25, &pinned);
        let b = truncate_compact_with_pinned(&msgs, 25, &pinned);
        assert_eq!(a, b);
    }
}