//! Tests v26 — `omega-plugin-api`.

use omega_plugin_api::{
    compose, install, Contributor, InstallContext, PluginFactory, PluginLoader,
};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaError, OmegaResult};
use std::sync::Arc;

struct FakeFactory;

impl PluginFactory for FakeFactory {
    type Instance = i32;
    fn manifest(&self) -> Manifest {
        Manifest::new("fake", Capability::Tool, "1")
    }
    fn instantiate(&self, _ctx: &InstallContext) -> OmegaResult<Arc<i32>> {
        Ok(Arc::new(42))
    }
}

#[test]
fn manifests_from_json() {
    let json = r#"[{"id":"a","capability":"tool","version":"1","priority":0}]"#;
    let m = PluginLoader::manifests_from_json(json).unwrap();
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].id, "a");
}

#[test]
fn manifests_from_json_rejects_bad() {
    assert!(PluginLoader::manifests_from_json("not json").is_err());
}

#[test]
fn compose_runs_contributors() {
    struct Noop;
    impl Contributor for Noop {
        fn name(&self) -> &str {
            "noop"
        }
        fn contribute(&self, _b: &mut RegistryBuilder, _c: &InstallContext) -> OmegaResult<()> {
            Ok(())
        }
    }
    let reg = compose(&[&Noop], &InstallContext::new()).unwrap();
    assert!(reg.active_ids(Capability::Tool).is_empty());
}

#[test]
fn install_factory_registers() {
    let mut b = RegistryBuilder::new();
    install(&FakeFactory, &mut b, &InstallContext::new()).unwrap();
    let r = b.build();
    assert_eq!(r.active_ids(Capability::Tool), vec!["fake".to_string()]);
}
