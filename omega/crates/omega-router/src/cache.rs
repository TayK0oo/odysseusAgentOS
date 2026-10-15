//! Cache deterministe des decisions de routage.
//!
//! La cle de cache inclut une version du registre (catalogue + sante) pour
//! invalider automatiquement les decisions obsoletes lors d'un changement de
//! catalogue (REVUE-W4 M2). L'invalidation manuelle reste disponible.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use omega_types::{Capabilities, RouterDecision, TaskType};

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
    ///
    /// `registry_version` represente la version du catalogue / etat de sante.
    /// Deux versions differentes produisent des cles differentes, assurant
    /// l'invalidation sur changement de registre.
    pub fn key(
        task: TaskType,
        required_capabilities: &Capabilities,
        required_context: u32,
        text: &str,
        registry_version: u64,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        task.hash(&mut hasher);
        required_capabilities.hash(&mut hasher);
        required_context.hash(&mut hasher);
        text.hash(&mut hasher);
        registry_version.hash(&mut hasher);
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
        let key_a = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello", 1);
        let key_b = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "world", 1);

        assert!(cache.get(key_a).is_none());
        let decision = RouterDecision::new("m", 0.9, "test");
        cache.insert(key_a, decision.clone());
        assert_eq!(cache.get(key_a).unwrap().model_id, "m");
        assert!(cache.get(key_b).is_none());
    }

    #[test]
    fn cache_invalidation() {
        let mut cache = DecisionCache::new();
        let key = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hi", 1);
        cache.insert(key, RouterDecision::new("m", 0.9, "test"));
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
            7,
        );
        let key2 = DecisionCache::key(
            TaskType::Implementation,
            &Capabilities::default(),
            4096,
            "plan",
            7,
        );
        assert_eq!(key1, key2);
    }

    #[test]
    fn registry_version_changes_key() {
        // Meme requete mais version de registre differente => cle differente.
        let key_v1 = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello", 1);
        let key_v2 = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello", 2);
        assert_ne!(key_v1, key_v2);
    }

    #[test]
    fn cache_miss_after_registry_change() {
        let mut cache = DecisionCache::new();
        let key_v1 = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello", 1);
        cache.insert(key_v1, RouterDecision::new("m", 0.9, "test"));

        let key_v2 = DecisionCache::key(TaskType::Chat, &Capabilities::default(), 0, "hello", 2);
        assert!(
            cache.get(key_v2).is_none(),
            "catalogue change should produce a cache miss"
        );
    }
}
