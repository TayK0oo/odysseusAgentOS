//! `omega-registry` — le **registre de capacites** d'Omega.
//!
//! Coeur de « everything is a plugin » (ADR-0003) :
//! - deux **regimes** : `Slot` (exclusif) et `Ensemble` (concurrent) ;
//! - des **manifestes** declaratifs ;
//! - une resolution deterministe (IndexMap + priorite) ;
//! - une **quarantaine** : un plugin qui echoue n'arrete jamais le noyau, on replie sur
//!   l'implementation suivante (in fine la `legacy`).
//!
//! Phase 1 : stockage type-efface (`Arc<dyn Any + Send + Sync>`) + `resolve::<T>()`.
//! Une sous-registre type par capacite viendra en Phase 2 si necessaire.

use std::any::Any;
use std::sync::Arc;

use indexmap::IndexMap;
use omega_types::{Capability, OmegaError, OmegaResult, Regime};

/// Descripteur declaratif d'un plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub id: String,
    pub capability: Capability,
    pub version: String,
    /// Plus la priorite est haute, plus le plugin est choisi tot.
    pub priority: i32,
}

impl Manifest {
    pub fn new(id: impl Into<String>, capability: Capability, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capability,
            version: version.into(),
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

struct Entry {
    manifest: Manifest,
    instance: Arc<dyn Any + Send + Sync>,
    quarantined: bool,
}

/// Registre de capacites, immuable en usage, construit par [`RegistryBuilder`].
#[derive(Default)]
pub struct Registry {
    entries: IndexMap<Capability, Vec<Entry>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enregistre une implementation. En regime `Slot`, une seconde implementation
    /// non quarantainee est refusee (`SlotConflict`) : il faut retirer/remplacer la
    /// precedente explicitement.
    pub fn register(
        &mut self,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> OmegaResult<()> {
        let cap = manifest.capability;
        let list = self.entries.entry(cap).or_default();

        if cap.regime() == Regime::Slot {
            if let Some(owner) = list.iter().find(|e| !e.quarantined) {
                return Err(OmegaError::SlotConflict(cap, owner.manifest.id.clone()));
            }
        }

        list.push(Entry {
            manifest,
            instance,
            quarantined: false,
        });
        // Determinisme : tri par priorite decroissante, puis par id croissant.
        list.sort_by(|a, b| {
            b.manifest
                .priority
                .cmp(&a.manifest.priority)
                .then_with(|| a.manifest.id.cmp(&b.manifest.id))
        });
        Ok(())
    }

    /// Met un plugin en quarantaine (il n'est plus resolu ; on replie sur le suivant).
    pub fn quarantine(&mut self, id: &str) {
        for list in self.entries.values_mut() {
            for e in list.iter_mut() {
                if e.manifest.id == id {
                    e.quarantined = true;
                }
            }
        }
    }

    /// Ids des implementations actives (non quarantainees) d'une capacite, par ordre de priorite.
    pub fn active_ids(&self, cap: Capability) -> Vec<String> {
        self.entries
            .get(&cap)
            .map(|l| {
                l.iter()
                    .filter(|e| !e.quarantined)
                    .map(|e| e.manifest.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Resout la premiere implementation active du type demande.
    pub fn resolve<T: 'static + Send + Sync>(&self, cap: Capability) -> OmegaResult<Arc<T>> {
        let list = self
            .entries
            .get(&cap)
            .ok_or(OmegaError::NoImplementation(cap))?;
        for e in list.iter().filter(|e| !e.quarantined) {
            if let Ok(t) = e.instance.clone().downcast::<T>() {
                return Ok(t);
            }
        }
        Err(OmegaError::NoImplementation(cap))
    }
}

/// Constructeur de registre : on enregistre, puis on gele (`build()`).
pub struct RegistryBuilder {
    registry: Registry,
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }

    pub fn register(
        &mut self,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> OmegaResult<()> {
        self.registry.register(manifest, instance)
    }

    /// Gele le registre (immuable en usage).
    pub fn build(self) -> Registry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use omega_ports::{ContextEngine, Provider};
    use omega_types::{ChatRequest, ChatResponse, Message, Role};

    #[derive(Debug)]
    struct FakeProvider(&'static str);
    #[async_trait]
    impl Provider for FakeProvider {
        fn id(&self) -> &str {
            self.0
        }
        async fn complete(&self, _req: ChatRequest) -> OmegaResult<ChatResponse> {
            Ok(ChatResponse {
                message: Message::new(Role::Assistant, self.0),
                usage: Default::default(),
            })
        }
    }

    #[derive(Debug)]
    struct FakeEngine(&'static str);
    #[async_trait]
    impl ContextEngine for FakeEngine {
        fn name(&self) -> &str {
            self.0
        }
        fn owns_compaction(&self) -> bool {
            false
        }
        async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
            Ok(messages)
        }
    }

    fn as_any<T: 'static + Send + Sync>(v: T) -> Arc<dyn Any + Send + Sync> {
        Arc::new(v)
    }

    #[test]
    fn ensemble_allows_many() {
        // (b) regime ensemble : N implementations coexistent.
        let mut r = Registry::new();
        r.register(
            Manifest::new("openai", Capability::LlmProvider, "1"),
            as_any(FakeProvider("openai")),
        )
        .unwrap();
        r.register(
            Manifest::new("ollama", Capability::LlmProvider, "1"),
            as_any(FakeProvider("ollama")),
        )
        .unwrap();
        assert_eq!(
            r.active_ids(Capability::LlmProvider),
            vec!["ollama".to_string(), "openai".to_string()]
        );
    }

    #[test]
    fn slot_rejects_second_owner() {
        // (a) exclusivite slot : une seconde implementation non arbitree est refusee.
        let mut r = Registry::new();
        r.register(
            Manifest::new("default", Capability::ContextEngine, "1"),
            as_any(FakeEngine("default")),
        )
        .unwrap();
        let err = r
            .register(
                Manifest::new("other", Capability::ContextEngine, "1"),
                as_any(FakeEngine("other")),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            OmegaError::SlotConflict(Capability::ContextEngine, _)
        ));
        assert_eq!(
            r.active_ids(Capability::ContextEngine),
            vec!["default".to_string()]
        );
    }

    #[test]
    fn failure_is_isolated_and_falls_back() {
        // (c) isolation d'echec : quarantaine du 1er -> resolution sur le suivant.
        let mut r = Registry::new();
        r.register(
            Manifest::new("a", Capability::LlmProvider, "1").with_priority(10),
            as_any(FakeProvider("a")),
        )
        .unwrap();
        r.register(
            Manifest::new("b", Capability::LlmProvider, "1").with_priority(5),
            as_any(FakeProvider("b")),
        )
        .unwrap();

        // avant quarantaine : "a" (priorite 10) est resolu.
        let first = r.resolve::<FakeProvider>(Capability::LlmProvider).unwrap();
        assert_eq!(first.id(), "a");

        r.quarantine("a");
        assert_eq!(r.active_ids(Capability::LlmProvider), vec!["b".to_string()]);
        let second = r.resolve::<FakeProvider>(Capability::LlmProvider).unwrap();
        assert_eq!(second.id(), "b");
    }

    #[test]
    fn missing_capability_is_reported() {
        let r = Registry::new();
        let err = r
            .resolve::<FakeProvider>(Capability::LlmProvider)
            .unwrap_err();
        assert!(matches!(err, OmegaError::NoImplementation(_)));
    }

    #[test]
    fn builder_freezes() {
        let mut b = RegistryBuilder::new();
        b.register(
            Manifest::new("p", Capability::LlmProvider, "1"),
            as_any(FakeProvider("p")),
        )
        .unwrap();
        let r = b.build();
        assert_eq!(r.active_ids(Capability::LlmProvider).len(), 1);
    }
}
