//! Tests W3 — registry (schémas + dispatch).

use omega_tools::registry::{dispatch, schemas};
use omega_tools::ToolCtx;

fn ctx(name: &str) -> ToolCtx {
    let p = std::env::temp_dir().join(format!("omega-w3reg-{}", name));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    ToolCtx::new(&p)
}

#[test]
fn schema_contains_required_props() {
    let all = schemas();
    // Au moins un schéma contient chacune des propriétés.
    let mut has_content = false;
    let mut has_new_string = false;
    let mut has_old_string = false;
    let mut has_path = false;
    let mut has_pattern = false;
    for s in &all {
        if let Some(props) = s["function"]["parameters"]["properties"].as_object() {
            if props.contains_key("content") {
                has_content = true;
            }
            if props.contains_key("new_string") {
                has_new_string = true;
            }
            if props.contains_key("old_string") {
                has_old_string = true;
            }
            if props.contains_key("path") {
                has_path = true;
            }
            if props.contains_key("pattern") {
                has_pattern = true;
            }
        }
    }
    assert!(has_content);
    assert!(has_new_string);
    assert!(has_old_string);
    assert!(has_path);
    assert!(has_pattern);
}

#[test]
fn dispatch_rejects_missing_inputs() {
    let c = ctx("missing");
    // Outils exigeant au moins un argument : dispatch avec {} doit échouer.
    for t in [
        "read_file",
        "write_file",
        "append_file",
        "edit_file",
        "memory_store",
        "memory_forget",
    ] {
        let r = dispatch(&c, t, &serde_json::json!({}));
        assert!(!r.is_ok(), "dispatch accepte input manquant pour {}", t);
    }
    let _ = std::fs::remove_dir_all(&c.root);
}
