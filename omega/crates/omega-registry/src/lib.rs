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
//! Chaque entree memorise le `TypeId` attendu pour detecter les erreurs de cablage
//! (`TypeMismatch`) des l'enregistrement (`register_typed`) ou a la resolution.

use std::any::{Any, TypeId};
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

/// Vue d'inspection d'une entree de registre (isolation d'un plugin).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryInfo {
    pub id: String,
    pub version: String,
    pub priority: i32,
    pub quarantined: bool,
    pub kind: EntryKind,
}

/// Voie d'enregistrement d'une entree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Type concret (`register_typed`), resolu par `resolve::<T>`.
    Typed,
    /// Trait object (`register_dyn`), resolu par `resolve_dyn::<T>`.
    TraitObject,
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

#[derive(Clone)]
struct Entry {
    manifest: Manifest,
    instance: Arc<dyn Any + Send + Sync>,
    /// TypeId attendu pour la resolution (TypeId::of::<T>() pour register_typed).
    expected_type_id: TypeId,
    quarantined: bool,
}

/// Entree « trait object » : memorise une `Arc<dyn Trait>` (port non resolvable en
/// `Sized`). Permet a `App` de resoudre `Arc<dyn Provider>` / `Arc<dyn ContextEngine>`
/// / `Arc<dyn EventStore>` sans connaitre le type concret.
struct TraitEntry {
    manifest: Manifest,
    object: Box<dyn Any + Send + Sync>,
    quarantined: bool,
}

/// Registre de capacites, immuable en usage, construit par [`RegistryBuilder`].
#[derive(Default)]
pub struct Registry {
    entries: IndexMap<Capability, Vec<Entry>>,
    trait_entries: IndexMap<Capability, Vec<TraitEntry>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Proprietaire actif (non quarantaine) d'une capacite en regime `Slot`, en
    /// inspectant **les deux** voies d'enregistrement (`entries` et `trait_entries`).
    ///
    /// Garantit l'exclusivite de slot quel que soit le mode d'enregistrement : un
    /// `register_typed` puis un `register_dyn` sur la meme capacite Slot (ou l'inverse)
    /// refuse le second avec `SlotConflict`.
    fn active_owner(&self, cap: Capability) -> Option<String> {
        if let Some(list) = self.entries.get(&cap) {
            if let Some(e) = list.iter().find(|e| !e.quarantined) {
                return Some(e.manifest.id.clone());
            }
        }
        if let Some(list) = self.trait_entries.get(&cap) {
            if let Some(e) = list.iter().find(|e| !e.quarantined) {
                return Some(e.manifest.id.clone());
            }
        }
        None
    }

    fn register_inner(
        &mut self,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
        expected_type_id: TypeId,
    ) -> OmegaResult<()> {
        let cap = manifest.capability;

        if self
            .entries
            .get(&cap)
            .is_some_and(|l| l.iter().any(|e| e.manifest.id == manifest.id))
        {
            return Err(OmegaError::DuplicateId(cap, manifest.id));
        }

        if cap.regime() == Regime::Slot {
            if let Some(owner) = self.active_owner(cap) {
                return Err(OmegaError::SlotConflict(cap, owner));
            }
        }

        let list = self.entries.entry(cap).or_default();
        list.push(Entry {
            manifest,
            instance,
            expected_type_id,
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

    /// Enregistre une implementation type-effacee.
    ///
    /// Le `TypeId` attendu est celui du type concret encapsule. En regime `Slot`, une
    /// seconde implementation non quarantainee est refusee (`SlotConflict`) : il faut
    /// retirer/remplacer la precedente explicitement via [`Self::unregister`] /
    /// [`Self::replace`].
    pub fn register(
        &mut self,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> OmegaResult<()> {
        let expected = Any::type_id(&*instance);
        self.register_inner(manifest, instance, expected)
    }

    /// Enregistre une implementation en fixant le type attendu a `TypeId::of::<T>()`.
    ///
    /// C'est la forme recommandee : elle lie explicitement la capacite au port concret
    /// et permet a la resolution de renvoyer `TypeMismatch` si le cablage est incorrect.
    pub fn register_typed<T: 'static + Send + Sync>(
        &mut self,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        let expected = TypeId::of::<T>();
        let any_instance: Arc<dyn Any + Send + Sync> = instance;
        self.register_inner(manifest, any_instance, expected)
    }

    /// Enregistre un **trait object** (`Arc<dyn Trait>`) pour une capacite.
    ///
    /// Pour les capacites que l'on resout en port (`llm-provider`, `context-engine`,
    /// `event-store`...) : `App` manipule des `Arc<dyn Provider>` / `Arc<dyn ContextEngine>`
    /// / `Arc<dyn EventStore>` sans connaitre le type concret. Meme garantie
    /// d'exclusivite de slot / quarantaine que [`Self::register_typed`].
    pub fn register_dyn<T: ?Sized + Send + Sync + 'static>(
        &mut self,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        let cap = manifest.capability;

        if self
            .trait_entries
            .get(&cap)
            .is_some_and(|l| l.iter().any(|e| e.manifest.id == manifest.id))
        {
            return Err(OmegaError::DuplicateId(cap, manifest.id));
        }
        if cap.regime() == Regime::Slot {
            if let Some(owner) = self.active_owner(cap) {
                return Err(OmegaError::SlotConflict(cap, owner));
            }
        }

        let list = self.trait_entries.entry(cap).or_default();
        list.push(TraitEntry {
            manifest,
            object: Box::new(instance),
            quarantined: false,
        });
        list.sort_by(|a, b| {
            b.manifest
                .priority
                .cmp(&a.manifest.priority)
                .then_with(|| a.manifest.id.cmp(&b.manifest.id))
        });
        Ok(())
    }

    /// Resout le premier trait object actif du type demande (`Arc<dyn Provider>`...).
    pub fn resolve_dyn<T: ?Sized + Send + Sync + 'static>(
        &self,
        cap: Capability,
    ) -> OmegaResult<Arc<T>> {
        let list = self
            .trait_entries
            .get(&cap)
            .ok_or(OmegaError::NoImplementation(cap))?;

        let active: Vec<&TraitEntry> = list.iter().filter(|e| !e.quarantined).collect();
        if active.is_empty() {
            return Err(OmegaError::NoImplementation(cap));
        }

        for e in active {
            if let Some(a) = e.object.downcast_ref::<Arc<T>>() {
                return Ok(a.clone());
            }
        }
        // Des entrees actives existent mais aucune ne correspond au type demande :
        // c'est une erreur de cablage (TypeMismatch), pas une capacite absente.
        Err(OmegaError::TypeMismatch(cap))
    }

    /// Retire toutes les entrees `(cap, id)` — qu'elles soient actives ou quarantinees.
    pub fn unregister(&mut self, cap: Capability, id: &str) {
        if let Some(list) = self.entries.get_mut(&cap) {
            list.retain(|e| e.manifest.id != id);
        }
    }

    fn replace_inner(
        &mut self,
        cap: Capability,
        id: &str,
        register: impl FnOnce(&mut Self) -> OmegaResult<()>,
    ) -> OmegaResult<()> {
        let snapshot = self.entries.clone();
        self.unregister(cap, id);
        if let Err(e) = register(self) {
            self.entries = snapshot;
            return Err(e);
        }
        Ok(())
    }

    /// Remplace l'entree `(cap, id)` par une nouvelle implementation type-effacee.
    ///
    /// Le remplacement est **atomique** : si l'enregistrement de la nouvelle
    /// implementation echoue, l'ancienne entree est restauree.
    pub fn replace(
        &mut self,
        cap: Capability,
        id: &str,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> OmegaResult<()> {
        self.replace_inner(cap, id, |this| this.register(manifest, instance))
    }

    /// Remplace l'entree `(cap, id)` par une nouvelle implementation typee.
    ///
    /// Le remplacement est **atomique** : si l'enregistrement de la nouvelle
    /// implementation echoue, l'ancienne entree est restauree.
    pub fn replace_typed<T: 'static + Send + Sync>(
        &mut self,
        cap: Capability,
        id: &str,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        self.replace_inner(cap, id, |this| this.register_typed(manifest, instance))
    }

    /// Met un plugin en quarantaine : il n'est plus resolu ; on replie sur le suivant.
    ///
    /// La quarantaine est **scopée par capacite** : `quarantine(cap, id)` n'affecte pas
    /// un meme `id` sous une autre capacite.
    pub fn quarantine(&mut self, cap: Capability, id: &str) {
        if let Some(list) = self.entries.get_mut(&cap) {
            for e in list.iter_mut() {
                if e.manifest.id == id {
                    e.quarantined = true;
                }
            }
        }
        if let Some(list) = self.trait_entries.get_mut(&cap) {
            for e in list.iter_mut() {
                if e.manifest.id == id {
                    e.quarantined = true;
                }
            }
        }
    }

    /// Vue d'inspection d'une entree (sans exposer l'instance).
    ///
    /// Isole un plugin : id, version, priorite, quarantaine, voie d'enregistrement.
    pub fn describe(&self) -> Vec<(Capability, Vec<EntryInfo>)> {
        let mut out: Vec<(Capability, Vec<EntryInfo>)> = Vec::new();
        let mut push =
            |cap: Capability, info: EntryInfo| match out.iter_mut().find(|(c, _)| *c == cap) {
                Some((_, v)) => v.push(info),
                None => out.push((cap, vec![info])),
            };
        for (cap, list) in &self.entries {
            for e in list {
                push(
                    *cap,
                    EntryInfo {
                        id: e.manifest.id.clone(),
                        version: e.manifest.version.clone(),
                        priority: e.manifest.priority,
                        quarantined: e.quarantined,
                        kind: EntryKind::Typed,
                    },
                );
            }
        }
        for (cap, list) in &self.trait_entries {
            for e in list {
                push(
                    *cap,
                    EntryInfo {
                        id: e.manifest.id.clone(),
                        version: e.manifest.version.clone(),
                        priority: e.manifest.priority,
                        quarantined: e.quarantined,
                        kind: EntryKind::TraitObject,
                    },
                );
            }
        }
        out
    }

    /// Ids des implementations actives (non quarantainees) d'une capacite, par ordre de priorite.
    pub fn active_ids(&self, cap: Capability) -> Vec<String> {
        let mut ids: Vec<(i32, String)> = Vec::new();
        if let Some(list) = self.entries.get(&cap) {
            for e in list.iter().filter(|e| !e.quarantined) {
                ids.push((e.manifest.priority, e.manifest.id.clone()));
            }
        }
        if let Some(list) = self.trait_entries.get(&cap) {
            for e in list.iter().filter(|e| !e.quarantined) {
                ids.push((e.manifest.priority, e.manifest.id.clone()));
            }
        }
        // Fusion des deux voies puis tri **global** : priorite decroissante, id croissant.
        ids.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        ids.into_iter().map(|(_, id)| id).collect()
    }

    /// Resout la premiere implementation active du type demande.
    ///
    /// - `NoImplementation` si la capacite n'existe pas ou si toutes ses entrees sont
    ///   quarantinees.
    /// - `TypeMismatch` si des entrees actives existent mais aucune ne correspond au
    ///   type demande.
    pub fn resolve<T: 'static + Send + Sync>(&self, cap: Capability) -> OmegaResult<Arc<T>> {
        let list = self
            .entries
            .get(&cap)
            .ok_or(OmegaError::NoImplementation(cap))?;

        let active: Vec<&Entry> = list.iter().filter(|e| !e.quarantined).collect();
        if active.is_empty() {
            return Err(OmegaError::NoImplementation(cap));
        }

        let expected = TypeId::of::<T>();
        for e in active {
            if e.expected_type_id == expected {
                if let Ok(t) = e.instance.clone().downcast::<T>() {
                    return Ok(t);
                }
            }
        }
        Err(OmegaError::TypeMismatch(cap))
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

    pub fn register_typed<T: 'static + Send + Sync>(
        &mut self,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        self.registry.register_typed(manifest, instance)
    }

    pub fn register_dyn<T: ?Sized + Send + Sync + 'static>(
        &mut self,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        self.registry.register_dyn(manifest, instance)
    }

    pub fn unregister(&mut self, cap: Capability, id: &str) {
        self.registry.unregister(cap, id);
    }

    pub fn replace(
        &mut self,
        cap: Capability,
        id: &str,
        manifest: Manifest,
        instance: Arc<dyn Any + Send + Sync>,
    ) -> OmegaResult<()> {
        self.registry.replace(cap, id, manifest, instance)
    }

    pub fn replace_typed<T: 'static + Send + Sync>(
        &mut self,
        cap: Capability,
        id: &str,
        manifest: Manifest,
        instance: Arc<T>,
    ) -> OmegaResult<()> {
        self.registry.replace_typed(cap, id, manifest, instance)
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

    #[test]
    fn ensemble_allows_many() {
        // (b) regime ensemble : N implementations coexistent.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("openai", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("openai")),
        )
        .unwrap();
        r.register_typed(
            Manifest::new("ollama", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("ollama")),
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
        r.register_typed(
            Manifest::new("default", Capability::ContextEngine, "1"),
            Arc::new(FakeEngine("default")),
        )
        .unwrap();
        let err = r
            .register_typed(
                Manifest::new("other", Capability::ContextEngine, "1"),
                Arc::new(FakeEngine("other")),
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
        r.register_typed(
            Manifest::new("a", Capability::LlmProvider, "1").with_priority(10),
            Arc::new(FakeProvider("a")),
        )
        .unwrap();
        r.register_typed(
            Manifest::new("b", Capability::LlmProvider, "1").with_priority(5),
            Arc::new(FakeProvider("b")),
        )
        .unwrap();

        // avant quarantaine : "a" (priorite 10) est resolu.
        let first = r.resolve::<FakeProvider>(Capability::LlmProvider).unwrap();
        assert_eq!(first.id(), "a");

        r.quarantine(Capability::LlmProvider, "a");
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
    fn describe_isolates_entries_and_quarantine() {
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("a", Capability::LlmProvider, "2").with_priority(5),
            Arc::new(FakeProvider("a")),
        )
        .unwrap();
        r.register_typed(
            Manifest::new("b", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("b")),
        )
        .unwrap();
        r.register_dyn(
            Manifest::new("dyn", Capability::ContextEngine, "1"),
            Arc::new(FakeEngine("dyn")),
        )
        .unwrap();
        r.quarantine(Capability::LlmProvider, "b");

        let d = r.describe();
        let llm = d
            .iter()
            .find(|(c, _)| *c == Capability::LlmProvider)
            .unwrap();
        assert_eq!(llm.1.len(), 2);
        let b = llm.1.iter().find(|e| e.id == "b").unwrap();
        assert!(b.quarantined);
        assert_eq!(b.kind, EntryKind::Typed);
        let dyn_entry = d
            .iter()
            .find(|(c, _)| *c == Capability::ContextEngine)
            .map(|(_, v)| v.iter().find(|e| e.id == "dyn").unwrap())
            .unwrap();
        assert_eq!(dyn_entry.kind, EntryKind::TraitObject);
        assert!(!dyn_entry.quarantined);
    }

    #[test]
    fn builder_freezes() {
        let mut b = RegistryBuilder::new();
        b.register_typed(
            Manifest::new("p", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("p")),
        )
        .unwrap();
        let r = b.build();
        assert_eq!(r.active_ids(Capability::LlmProvider).len(), 1);
    }

    #[test]
    fn quarantine_is_scoped_by_capability() {
        // (§4.3) quarantaine d'un id sur une capacite ne neutralise pas l'autre.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("x", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("x")),
        )
        .unwrap();
        r.register_typed(
            Manifest::new("x", Capability::Tool, "1"),
            Arc::new(FakeProvider("x-tool")),
        )
        .unwrap();

        r.quarantine(Capability::LlmProvider, "x");

        assert!(r.active_ids(Capability::LlmProvider).is_empty());
        assert_eq!(r.active_ids(Capability::Tool), vec!["x"]);

        // La resolution sous Tool fonctionne toujours.
        let tool = r.resolve::<FakeProvider>(Capability::Tool).unwrap();
        assert_eq!(tool.id(), "x-tool");
    }

    #[test]
    fn type_mismatch_is_reported() {
        // (§4.4) enregistrer un Provider sous ContextEngine -> TypeMismatch.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("bad", Capability::ContextEngine, "1"),
            Arc::new(FakeProvider("bad")),
        )
        .unwrap();

        let err = r
            .resolve::<FakeEngine>(Capability::ContextEngine)
            .unwrap_err();
        assert!(matches!(
            err,
            OmegaError::TypeMismatch(Capability::ContextEngine)
        ));
    }

    #[test]
    fn slot_replace_after_quarantine() {
        // (§4.5) quarantaine du proprietaire d'un Slot puis re-enregistrement d'un successeur.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("v1", Capability::ContextEngine, "1"),
            Arc::new(FakeEngine("v1")),
        )
        .unwrap();

        r.quarantine(Capability::ContextEngine, "v1");
        assert!(r.active_ids(Capability::ContextEngine).is_empty());

        r.register_typed(
            Manifest::new("v2", Capability::ContextEngine, "2"),
            Arc::new(FakeEngine("v2")),
        )
        .unwrap();

        let resolved = r.resolve::<FakeEngine>(Capability::ContextEngine).unwrap();
        assert_eq!(resolved.name(), "v2");
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("dup", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("dup")),
        )
        .unwrap();
        let err = r
            .register_typed(
                Manifest::new("dup", Capability::LlmProvider, "2"),
                Arc::new(FakeProvider("dup2")),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            OmegaError::DuplicateId(Capability::LlmProvider, _)
        ));
    }

    #[test]
    fn slot_exclusivity_spans_typed_and_dyn() {
        // (B2) register_typed puis register_dyn sur une capacite Slot -> SlotConflict.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("typed-store", Capability::EventStore, "1"),
            Arc::new(FakeProvider("typed-store")),
        )
        .unwrap();
        let err = r
            .register_dyn(
                Manifest::new("dyn-store", Capability::EventStore, "1"),
                Arc::new(FakeProvider("dyn-store")),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            OmegaError::SlotConflict(Capability::EventStore, _)
        ));
        // Le proprietaire d'origine reste le seul actif.
        assert_eq!(
            r.active_ids(Capability::EventStore),
            vec!["typed-store".to_string()]
        );
    }

    #[test]
    fn resolve_dyn_reports_type_mismatch_not_no_implementation() {
        // (M1) trait object actif present mais de mauvais type -> TypeMismatch.
        let mut r = Registry::new();
        r.register_dyn(
            Manifest::new("bad", Capability::ContextEngine, "1"),
            Arc::new(FakeProvider("bad")),
        )
        .unwrap();

        let err = match r.resolve_dyn::<dyn ContextEngine>(Capability::ContextEngine) {
            Ok(_) => panic!("resolve_dyn devait echouer : type incompatible"),
            Err(e) => e,
        };
        assert!(matches!(
            err,
            OmegaError::TypeMismatch(Capability::ContextEngine)
        ));
    }

    #[test]
    fn active_ids_merges_and_sorts_across_ways() {
        // (M2) la priorite est globale entre voies typed et dyn.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("typed-low", Capability::LlmProvider, "1").with_priority(0),
            Arc::new(FakeProvider("typed-low")),
        )
        .unwrap();
        r.register_dyn(
            Manifest::new("dyn-high", Capability::LlmProvider, "1").with_priority(100),
            Arc::new(FakeProvider("dyn-high")),
        )
        .unwrap();

        assert_eq!(
            r.active_ids(Capability::LlmProvider),
            vec!["dyn-high".to_string(), "typed-low".to_string()]
        );
    }

    #[test]
    fn unregister_removes_entry() {
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("gone", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("gone")),
        )
        .unwrap();
        r.unregister(Capability::LlmProvider, "gone");
        assert!(r.active_ids(Capability::LlmProvider).is_empty());
        let err = r
            .resolve::<FakeProvider>(Capability::LlmProvider)
            .unwrap_err();
        assert!(matches!(err, OmegaError::NoImplementation(_)));
    }

    #[test]
    fn replace_is_atomic_on_failure() {
        // (N2) si le re-enregistrement echoue, l'ancienne implementation est restauree.
        let mut r = Registry::new();
        r.register_typed(
            Manifest::new("a", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("a")),
        )
        .unwrap();
        r.register_typed(
            Manifest::new("b", Capability::LlmProvider, "1"),
            Arc::new(FakeProvider("b")),
        )
        .unwrap();

        // Remplacer "a" par un manifeste utilisant l'id "b" -> DuplicateId.
        let err = r
            .replace_typed(
                Capability::LlmProvider,
                "a",
                Manifest::new("b", Capability::LlmProvider, "2"),
                Arc::new(FakeProvider("a2")),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            OmegaError::DuplicateId(Capability::LlmProvider, _)
        ));

        // "a" doit toujours etre present et resoluble.
        assert_eq!(r.active_ids(Capability::LlmProvider), vec!["a", "b"]);
        let resolved = r.resolve::<FakeProvider>(Capability::LlmProvider).unwrap();
        assert_eq!(resolved.id(), "a");
    }
}
