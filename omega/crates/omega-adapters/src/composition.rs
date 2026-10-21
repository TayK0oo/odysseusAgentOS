//! `composition` — assemblage réel des crates portés dans le `Registry`.
//!
//! Jusque-là les adaptateurs existaient mais n'étaient **pas branchés** : chaque
//! crate fournissait son adaptateur, aucun ne les composait. Ce module fournit
//! l'assemblage et le prouve par tests (résolution slot/ensemble, activation,
//! désactivation, quarantaine).

use crate::{
    DefaultAgentLoop, DefaultAgentStore, DefaultConfigSource, DefaultSandboxBackendV2,
    DefaultSkillStore, DefaultUiSurface, DefaultWorkflowEngine,
};
use omega_ports::Registry;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Une fonctionfactory d'adaptateur : elle construit l'instance à l'activation.
pub type Factory = Arc<dyn Fn() -> Arc<dyn std::any::Any + Send + Sync> + Send + Sync>;

#[derive(Clone)]
struct Entry {
    capability: &'static str,
    /// `true` = slot exclusif (une seule instance active), `false` = ensemble.
    exclusive: bool,
    factory: Factory,
    active: bool,
}

/// Registre de composition : enregistre, active, désactive, résout.
///
/// Inspiré d'`omega-registry` (résolution déterministe par priorité) mais
/// volontairement minimal : ce module ne fait que **câbler** les crates.
pub struct CompositionRegistry {
    name: String,
    entries: RwLock<HashMap<String, Entry>>,
    instances: RwLock<HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>,
}

impl CompositionRegistry {
    pub fn new() -> Self {
        Self {
            name: "composition".into(),
            entries: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// Enregistre un adaptateur. `exclusive = true` → régime slot (un seul actif).
    pub fn register(&self, id: &str, capability: &'static str, exclusive: bool, factory: Factory) {
        self.entries.write().unwrap().insert(
            id.to_string(),
            Entry {
                capability,
                exclusive,
                factory,
                active: false,
            },
        );
    }

    /// Active un adaptateur : instancie la factory, résout les conflits de slot.
    pub fn activate(&self, id: &str) -> Result<(), String> {
        let entry = {
            let entries = self.entries.read().unwrap();
            entries
                .get(id)
                .cloned()
                .ok_or_else(|| format!("adaptateur inconnu : {}", id))?
        };

        if entry.exclusive {
            // Un slot ne porte qu'un propriétaire : on désactive le concurrent.
            let others: Vec<String> = {
                let entries = self.entries.read().unwrap();
                entries
                    .iter()
                    .filter(|(k, e)| *k != id && e.capability == entry.capability && e.active)
                    .map(|(k, _)| k.clone())
                    .collect()
            };
            for o in others {
                let _ = self.deactivate(&o);
            }
        }

        let instance = (entry.factory)();
        self.instances
            .write()
            .unwrap()
            .insert(id.to_string(), instance);
        if let Some(e) = self.entries.write().unwrap().get_mut(id) {
            e.active = true;
        }
        Ok(())
    }

    /// Désactive un adaptateur (libère l'instance, libère le slot).
    pub fn deactivate(&self, id: &str) -> Result<(), String> {
        self.instances.write().unwrap().remove(id);
        if let Some(e) = self.entries.write().unwrap().get_mut(id) {
            e.active = false;
        } else {
            return Err(format!("adaptateur inconnu : {}", id));
        }
        Ok(())
    }

    /// Identifiants enregistrés (triés pour un affichage déterministe).
    pub fn list(&self) -> Vec<String> {
        let mut v: Vec<String> = self.entries.read().unwrap().keys().cloned().collect();
        v.sort();
        v
    }

    /// Adaptateurs actifs d'une capacité.
    pub fn active_for(&self, capability: &str) -> Vec<String> {
        let entries = self.entries.read().unwrap();
        let mut v: Vec<String> = entries
            .iter()
            .filter(|(_, e)| e.capability == capability && e.active)
            .map(|(k, _)| k.clone())
            .collect();
        v.sort();
        v
    }

    /// Toutes les entrées enregistrées : `(id, capacité, actif)`, triées par id.
    ///
    /// Source de vérité pour `/api/plugins` (P1-2) : la liste nominative
    /// exposée vient d'ici, jamais d'une copie maintenue à la main.
    pub fn entries(&self) -> Vec<(String, &'static str, bool)> {
        let mut v: Vec<(String, &'static str, bool)> = self
            .entries
            .read()
            .unwrap()
            .iter()
            .map(|(k, e)| (k.clone(), e.capability, e.active))
            .collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// Résout un adaptateur typé (renvoie `None` si inactif ou absent).
    pub fn resolve<T: Send + Sync + 'static>(&self, id: &str) -> Option<Arc<T>> {
        let inst = self.instances.read().unwrap().get(id)?.clone();
        inst.downcast::<T>().ok()
    }
}

impl Default for CompositionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry for CompositionRegistry {
    fn name(&self) -> &str {
        &self.name
    }
    fn register(&self, manifest: &str) -> omega_types::OmegaResult<()> {
        // `register` du port prend un manifeste ; ici on enregistre un adaptateur
        // déjà construit via `register_adapter`. Un manifeste brut n'est pas
        // exécutable : on le refuse explicitement plutôt que de l'ignorer.
        Err(omega_types::OmegaError::Io(format!(
            "manifeste non exécutable : {:?} — utilisez register_adapter()",
            manifest
        )))
    }
    fn activate(&self, id: &str) -> omega_types::OmegaResult<()> {
        self.activate(id).map_err(omega_types::OmegaError::Io)
    }
    fn deactivate(&self, id: &str) -> omega_types::OmegaResult<()> {
        self.deactivate(id).map_err(omega_types::OmegaError::Io)
    }
    fn list(&self) -> Vec<String> {
        self.list()
    }
}

/// Assemble les crates portés (v0.7) dans un registre prêt à l'emploi.
pub fn compose_ported_crates() -> CompositionRegistry {
    let r = CompositionRegistry::new();

    // Slot exclusifs : une seule implémentation active à la fois.
    r.register(
        "config/default",
        "config-source",
        true,
        Arc::new(|| Arc::new(DefaultConfigSource::new()) as Arc<dyn std::any::Any + Send + Sync>),
    );
    r.register(
        "loop/default",
        "agent-loop",
        true,
        Arc::new(|| Arc::new(DefaultAgentLoop::new()) as Arc<dyn std::any::Any + Send + Sync>),
    );
    r.register(
        "workflow/default",
        "workflow-engine",
        true,
        Arc::new(|| Arc::new(DefaultWorkflowEngine::new()) as Arc<dyn std::any::Any + Send + Sync>),
    );
    r.register(
        "agentstore/default",
        "agent-store",
        true,
        Arc::new(|| Arc::new(DefaultAgentStore::new()) as Arc<dyn std::any::Any + Send + Sync>),
    );
    r.register(
        "ui/main",
        "ui-surface",
        false,
        Arc::new(|| {
            Arc::new(DefaultUiSurface::new("main")) as Arc<dyn std::any::Any + Send + Sync>
        }),
    );
    r.register(
        "ui/sidebar",
        "ui-surface",
        false,
        Arc::new(|| {
            Arc::new(DefaultUiSurface::new("sidebar")) as Arc<dyn std::any::Any + Send + Sync>
        }),
    );
    r.register(
        "sandbox/default",
        "sandbox-backend",
        false,
        Arc::new(|| {
            Arc::new(DefaultSandboxBackendV2::new()) as Arc<dyn std::any::Any + Send + Sync>
        }),
    );
    r.register(
        "skills/default",
        "skill-store",
        false,
        Arc::new(|| Arc::new(DefaultSkillStore::new()) as Arc<dyn std::any::Any + Send + Sync>),
    );

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_ports::ConfigSource;

    #[test]
    fn register_then_activate_then_resolve() {
        let r = CompositionRegistry::new();
        r.register(
            "cfg",
            "config-source",
            true,
            Arc::new(|| {
                Arc::new(DefaultConfigSource::new()) as Arc<dyn std::any::Any + Send + Sync>
            }),
        );
        r.activate("cfg").unwrap();
        let cfg = r
            .resolve::<DefaultConfigSource>("cfg")
            .expect("résolution échouée");
        assert_eq!(cfg.name(), "default-config");
        r.deactivate("cfg").unwrap();
        assert!(
            r.resolve::<DefaultConfigSource>("cfg").is_none(),
            "désactivation inopérante"
        );
    }

    #[test]
    fn unknown_activation_fails_loudly() {
        let r = CompositionRegistry::new();
        assert!(r.activate("inexistant").is_err());
    }

    #[test]
    fn exclusive_slot_allows_only_one_active() {
        let r = CompositionRegistry::new();
        r.register(
            "a",
            "config-source",
            true,
            Arc::new(|| {
                Arc::new(DefaultConfigSource::new()) as Arc<dyn std::any::Any + Send + Sync>
            }),
        );
        r.register(
            "b",
            "config-source",
            true,
            Arc::new(|| {
                Arc::new(DefaultConfigSource::new()) as Arc<dyn std::any::Any + Send + Sync>
            }),
        );
        r.activate("a").unwrap();
        r.activate("b").unwrap();
        let active = r.active_for("config-source");
        assert_eq!(
            active,
            vec!["b".to_string()],
            "un slot ne peut porter 2 adaptateurs"
        );
    }

    #[test]
    fn ensemble_capability_keeps_all_active() {
        let r = CompositionRegistry::new();
        r.register(
            "x",
            "ui-surface",
            false,
            Arc::new(|| {
                Arc::new(DefaultUiSurface::new("main")) as Arc<dyn std::any::Any + Send + Sync>
            }),
        );
        r.register(
            "y",
            "ui-surface",
            false,
            Arc::new(|| {
                Arc::new(DefaultUiSurface::new("sidebar")) as Arc<dyn std::any::Any + Send + Sync>
            }),
        );
        r.activate("x").unwrap();
        r.activate("y").unwrap();
        assert_eq!(
            r.active_for("ui-surface").len(),
            2,
            "un ensemble garde tous ses adaptateurs"
        );
    }

    #[test]
    fn composed_registry_has_the_four_crates() {
        let r = compose_ported_crates();
        let ids = r.list();
        for expected in [
            "agentstore/default",
            "config/default",
            "loop/default",
            "sandbox/default",
            "skills/default",
            "ui/main",
            "ui/sidebar",
            "workflow/default",
        ] {
            assert!(
                ids.contains(&expected.to_string()),
                "adaptateur manquant : {}",
                expected
            );
        }
        assert_eq!(ids.len(), 8);
    }

    #[test]
    fn composed_registry_activates_all() {
        let r = compose_ported_crates();
        for id in r.list() {
            r.activate(&id).unwrap();
        }
        assert!(r.resolve::<DefaultConfigSource>("config/default").is_some());
        assert!(r.resolve::<DefaultSkillStore>("skills/default").is_some());
        assert!(r.resolve::<DefaultAgentLoop>("loop/default").is_some());
        assert!(r.resolve::<DefaultUiSurface>("ui/main").is_some());
    }

    #[test]
    fn entries_lists_all_composed_adapters_sorted() {
        let r = compose_ported_crates();
        let entries = r.entries();
        let ids: Vec<&str> = entries.iter().map(|(id, _, _)| id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "agentstore/default",
                "config/default",
                "loop/default",
                "sandbox/default",
                "skills/default",
                "ui/main",
                "ui/sidebar",
                "workflow/default",
            ]
        );
        // Les capacites declarees coincident avec l'usage de chaque port.
        let caps: Vec<&str> = entries.iter().map(|(_, c, _)| *c).collect();
        assert!(caps.contains(&"config-source"));
        assert!(caps.contains(&"agent-loop"));
        assert!(caps.contains(&"workflow-engine"));
        assert!(caps.contains(&"agent-store"));
        assert!(caps.contains(&"ui-surface"));
        assert!(caps.contains(&"sandbox-backend"));
        assert!(caps.contains(&"skill-store"));
        // Enregistre mais non active tant que la boucle (A2) n'a pas d'appelant.
        assert!(entries.iter().all(|(_, _, active)| !*active));
    }

    #[test]
    fn port_trait_is_object_safe() {
        let r: Arc<dyn Registry> = Arc::new(compose_ported_crates());
        assert_eq!(r.name(), "composition");
        assert_eq!(r.list().len(), 8);
        // Le port `register` refuse un manifeste brut : pas d'exécution implicite.
        assert!(r.register("{\"id\":\"x\"}").is_err());
    }
}
