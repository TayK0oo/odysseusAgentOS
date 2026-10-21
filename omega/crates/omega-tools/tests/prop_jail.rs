//! Propriété §7-A2 (jail de chemins) : quel que soit le chemin écrit par le
//! modèle (Unicode, ponctuation, séparateurs, très long), une résolution
//! `Ok` reste strictement sous la racine autorisée — tout échappement est
//! un `Err` structuré, jamais de panic et jamais un chemin résolu hors jail.
//!
//! Complète `tests/adversarial.rs` (cas nommés) par un balayage aléatoire.

use omega_tools::ToolCtx;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Tout chemin résolu `Ok` reste sous la racine canonique.
    #[test]
    fn jail_ne_sors_jamais(raw in any::<String>()) {
        // Borné : proptest avec une taille réaliste (longueur ≤ 128 chars).
        let raw: String = raw.chars().take(128).collect();
        let root = std::env::temp_dir().join("prop-jail-omega");
        std::fs::create_dir_all(&root).expect("root temporaire");
        let ctx = ToolCtx::new(&root);
        match ctx.resolve(&raw) {
            Ok(p) => {
                let canon = std::fs::canonicalize(&root)
                    .unwrap_or_else(|_| normalize_fallback(&root));
                prop_assert!(p.starts_with(&canon), "échappement: {:?} -> {:?}", raw, p);
            }
            Err(_) => { /* Refus = comportement attendu pour les échappements */ }
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// Fallback canonique si le tempdir est déjà parti (concurrence de test).
fn normalize_fallback(p: &std::path::Path) -> std::path::PathBuf {
    let mut out = std::path::PathBuf::new();
    for c in p.components() {
        if c == std::path::Component::ParentDir {
            out.pop();
        } else {
            out.push(c.as_os_str());
        }
    }
    out
}
