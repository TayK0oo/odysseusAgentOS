//! Faits épinglés (pinned facts) : faits durables qui survivent à la compaction.
//!
//! Ce module fournit une structure `PinnedFacts` qui stocke des faits clés
//! (décisions, préférences utilisateur, informations de projet) qui doivent
//! être préservés même quand le contexte est compacté par `truncate_compact`
//! ou `LlmCompactionEngine`.

use std::collections::HashMap;
use std::sync::RwLock;

use omega_types::{Message, Role};

/// Fait épinglé avec métadonnées.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedFact {
    /// Identifiant unique du fait (pour mise à jour / suppression).
    pub id: String,
    /// Contenu du fait.
    pub content: String,
    /// Catégorie : "decision", "preference", "project", "constraint", etc.
    pub category: String,
    /// Priorité (plus haut = plus important, conservé en priorité).
    pub priority: i32,
    /// Timestamp de création (epoch millis).
    pub created_at: u64,
    /// Timestamp de dernière mise à jour (epoch millis).
    pub updated_at: u64,
}

impl PinnedFact {
    pub fn new(id: impl Into<String>, content: impl Into<String>, category: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Self {
            id: id.into(),
            content: content.into(),
            category: category.into(),
            priority: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Convertit en message système pour injection dans le contexte.
    pub fn to_system_message(&self) -> Message {
        Message::new(
            Role::System,
            format!("[PinnedFact:{}] {}", self.category, self.content),
        )
    }
}

/// Stockage des faits épinglés, thread-safe.
#[derive(Debug, Default)]
pub struct PinnedFacts {
    facts: RwLock<HashMap<String, PinnedFact>>,
}

impl PinnedFacts {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute ou met à jour un fait épinglé.
    pub fn upsert(&self, fact: PinnedFact) {
        let mut facts = self.facts.write().expect("pinned facts lock poisoned");
        let mut updated = fact;
        updated.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        facts.insert(updated.id.clone(), updated);
    }

    /// Récupère un fait par son ID.
    pub fn get(&self, id: &str) -> Option<PinnedFact> {
        self.facts.read().expect("pinned facts lock poisoned").get(id).cloned()
    }

    /// Supprime un fait par son ID.
    pub fn remove(&self, id: &str) -> Option<PinnedFact> {
        self.facts.write().expect("pinned facts lock poisoned").remove(id)
    }

    /// Liste tous les faits, triés par priorité (desc) puis date de création (asc).
    pub fn list(&self) -> Vec<PinnedFact> {
        let mut facts: Vec<PinnedFact> = self
            .facts
            .read()
            .expect("pinned facts lock poisoned")
            .values()
            .cloned()
            .collect();
        facts.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
        facts
    }

    /// Liste les faits d'une catégorie donnée.
    pub fn list_by_category(&self, category: &str) -> Vec<PinnedFact> {
        self.list()
            .into_iter()
            .filter(|f| f.category == category)
            .collect()
    }

    /// Nombre total de faits épinglés.
    pub fn len(&self) -> usize {
        self.facts.read().expect("pinned facts lock poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convertit tous les faits en messages système (pour injection dans le contexte).
    /// Triés par priorité décroissante.
    pub fn to_system_messages(&self) -> Vec<Message> {
        self.list().iter().map(|f| f.to_system_message()).collect()
    }

    /// Estime le coût en tokens de tous les faits épinglés.
    pub fn estimate_tokens(&self, estimator: &crate::token::TokenEstimator) -> usize {
        self.list()
            .iter()
            .map(|f| estimator.estimate(&f.content))
            .sum()
    }

    /// Nettoie tous les faits.
    pub fn clear(&self) {
        self.facts.write().expect("pinned facts lock poisoned").clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_and_get() {
        let pf = PinnedFacts::new();
        let fact = PinnedFact::new("fact1", "User prefers dark mode", "preference");
        pf.upsert(fact.clone());
        let got = pf.get("fact1").unwrap();
        assert_eq!(got.content, "User prefers dark mode");
        assert_eq!(got.category, "preference");
    }

    #[test]
    fn update_preserves_id_and_category() {
        let pf = PinnedFacts::new();
        pf.upsert(PinnedFact::new("f1", "old content", "decision"));
        pf.upsert(PinnedFact::new("f1", "new content", "decision").with_priority(10));
        let got = pf.get("f1").unwrap();
        assert_eq!(got.content, "new content");
        assert_eq!(got.priority, 10);
        assert_eq!(got.category, "decision");
    }

    #[test]
    fn remove_returns_fact() {
        let pf = PinnedFacts::new();
        pf.upsert(PinnedFact::new("f1", "content", "cat"));
        let removed = pf.remove("f1").unwrap();
        assert_eq!(removed.id, "f1");
        assert!(pf.get("f1").is_none());
    }

    #[test]
    fn list_sorted_by_priority_then_created() {
        let pf = PinnedFacts::new();
        pf.upsert(PinnedFact::new("low", "low prio", "cat").with_priority(0));
        pf.upsert(PinnedFact::new("high", "high prio", "cat").with_priority(10));
        pf.upsert(PinnedFact::new("med", "med prio", "cat").with_priority(5));
        let list = pf.list();
        assert_eq!(list[0].id, "high");
        assert_eq!(list[1].id, "med");
        assert_eq!(list[2].id, "low");
    }

    #[test]
    fn list_by_category_filters() {
        let pf = PinnedFacts::new();
        pf.upsert(PinnedFact::new("p1", "pref 1", "preference"));
        pf.upsert(PinnedFact::new("d1", "dec 1", "decision"));
        pf.upsert(PinnedFact::new("p2", "pref 2", "preference"));
        let prefs = pf.list_by_category("preference");
        assert_eq!(prefs.len(), 2);
        assert!(prefs.iter().all(|f| f.category == "preference"));
    }

    #[test]
    fn to_system_messages_format() {
        let pf = PinnedFacts::new();
        pf.upsert(PinnedFact::new("f1", "User likes Rust", "preference"));
        let msgs = pf.to_system_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].role, Role::System);
        assert!(msgs[0].content.starts_with("[PinnedFact:preference]"));
        assert!(msgs[0].content.contains("User likes Rust"));
    }
}