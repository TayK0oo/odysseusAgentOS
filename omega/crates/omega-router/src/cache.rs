//! Cache deterministe des decisions de routage.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use omega_providers::Capabilities;

use crate::selector::RouterDecision;
use crate::task::TaskType;

/// Cache de decisions.
#[derive(Debug, Default)]
pub struct DecisionCache {
    entries: HashMap<u64, RouterDecision>,
}

impl DecisionCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cle deterministe pour une requete de routage.
    pub fn key(
        task: TaskType,
        required_capabilities: &Capabilities,
        required_context: u32,
        text: &str,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        task.hash(&mut hasher);
        required_capabilities.hash(&mut hasher);
        required_context.hash(&mut hasher);
        text.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get(&self, key: u64) -> Option<&RouterDecision> {
        self.entries.get(&key)
    }

    pub fn insert(&mut self, key: u64, decision: RouterDecision) -> Option<RouterDecision> {
        self.entries.insert(key, decision)
    }

    pub fn remove(&mut self, key: u64) -> Option<RouterDecision> {
        self.entries.remove(&key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_and_miss() {
        let mut cache = DecisionCache::new();
        let key_a = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello");
        let key_b = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "world");

        assert!(cache.get(key_a).is_none());
        let decision = RouterDecision {
            model_id: "m".to_string(),
            score: 0.9,
            reason: "test".to_string(),
        };
        cache.insert(key_a, decision.clone());
        assert_eq!(cache.get(key_a).unwrap().model_id, "m");
        assert!(cache.get(key_b).is_none());
    }

    #[test]
    fn cache_invalidation() {
        let mut cache = DecisionCache::new();
        let key = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hi");
        cache.insert(
            key,
            RouterDecision {
                model_id: "m".to_string(),
                score: 0.9,
                reason: "test".to_string(),
            },
        );
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn determinism_same_key() {
        let key1 = DecisionCache::key(
            TaskType::Implementation,
            &Capabilities::default(),
            4096,
            "plan",
        );
        let key2 = DecisionCache::key(
            TaskType::Implementation,
            &Capabilities::default(),
            4096,
            "plan",
        );
        assert_eq!(key1, key2);
    }
}
