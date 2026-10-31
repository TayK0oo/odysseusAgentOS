//! Tests v25 — `omega-registry`.

use omega_registry::{Manifest, Registry, RegistryBuilder};
use omega_types::Capability;
use std::sync::Arc;

#[test]
fn register_and_resolve() {
    let mut b = RegistryBuilder::new();
    let reg = b
        .register_dyn(
            Manifest::new("test", Capability::Tool, "1"),
            Arc::new(42i32),
        )
        .unwrap();
    let r = b.build();
    assert_eq!(
        r.resolve_dyn::<i32>(Capability::Tool).unwrap().as_ref(),
        &42
    );
    let _ = reg;
}

#[test]
fn unregister_removes() {
    let mut b = RegistryBuilder::new();
    let reg = b
        .register_dyn(
            Manifest::new("test", Capability::Tool, "1"),
            Arc::new(42i32),
        )
        .unwrap();
    b.unregister(Capability::Tool, &reg.id);
    let r = b.build();
    assert!(r.resolve_dyn::<i32>(Capability::Tool).is_err());
}

#[test]
fn quarantine_marks() {
    let mut b = RegistryBuilder::new();
    let reg = b
        .register_dyn(
            Manifest::new("test", Capability::Tool, "1"),
            Arc::new(42i32),
        )
        .unwrap();
    let mut r = b.build();
    r.quarantine(Capability::Tool, &reg.id);
    assert!(!r.active_ids(Capability::Tool).contains(&reg.id));
}

#[test]
fn active_owner_returns_id() {
    let mut b = RegistryBuilder::new();
    let reg = b
        .register_dyn(
            Manifest::new("test", Capability::Tool, "1"),
            Arc::new(42i32),
        )
        .unwrap();
    let r = b.build();
    assert!(r.active_ids(Capability::Tool).contains(&reg.id));
}

#[test]
fn build_empty_ok() {
    let b = RegistryBuilder::new();
    let r = b.build();
    assert!(r.active_ids(Capability::Tool).is_empty());
}

#[test]
fn registration_dispose() {
    let mut b = RegistryBuilder::new();
    let reg = b
        .register_dyn(
            Manifest::new("test", Capability::Tool, "1"),
            Arc::new(42i32),
        )
        .unwrap();
    let mut r = b.build();
    reg.dispose(&mut r);
    assert!(r.resolve_dyn::<i32>(Capability::Tool).is_err());
}
