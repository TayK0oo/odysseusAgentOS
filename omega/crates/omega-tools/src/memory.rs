//! `memory` — outils mémoire de la surface (T1 : écriture dans le workspace).
//!
//! Même forme que `FileMemoryStore` (`omega-app`) : JSONL append-only, une
//! ligne par fait. Ici le fait garde sa provenance ; le fichier vit dans la
//! jail de [`ToolCtx`], donc `resolve` applique confinement et deny-list.

use super::{ToolCtx, ToolOutput};

/// Fichier de mémoire, relatif à la racine du workspace.
pub const MEMORY_PATH: &str = ".omega/memory.jsonl";

/// Résout le chemin du magasin mémoire : env OMEGA_MEMORY_STORE si défini
/// (store unifié avec l'injection C4), sinon .omega/memory.jsonl dans le
/// workspace. Un seul magasin pour l'outil et l'injection.
pub fn memory_path(ctx: &ToolCtx) -> Result<std::path::PathBuf, String> {
    let rel = std::env::var("OMEGA_MEMORY_STORE")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| MEMORY_PATH.to_string());
    ctx.resolve(&rel)
}

/// Nombre maximal de faits renvoyés par un `recall` (borne de sortie).
pub const RECALL_LIMIT: usize = 20;

/// Enregistre un fait avec sa provenance (append JSONL, jamais d'écrasement).
pub fn store(ctx: &ToolCtx, content: &str, provenance: &str) -> ToolOutput {
    if content.trim().is_empty() {
        return ToolOutput::err("contenu vide : rien à mémoriser");
    }
    if provenance.trim().is_empty() {
        return ToolOutput::err("provenance vide : fait non traçable");
    }
    let content = omega_types::redact_secrets(content);
    let provenance = omega_types::redact_secrets(provenance);
    let abs = match memory_path(ctx) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    if let Some(parent) = abs.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ToolOutput::err(format!("création dossier impossible : {}", e));
        }
    }
    let record = serde_json::json!({ "content": content, "provenance": provenance });
    // `serde_json` échappe les retours à la ligne : une ligne = un fait.
    let line = format!("{}\n", record);
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&abs)
    {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(line.as_bytes()) {
                return ToolOutput::err(format!("écriture impossible : {}", e));
            }
            if let Err(e) = f.sync_all() {
                return ToolOutput::err(format!("écriture impossible : {}", e));
            }
            ToolOutput::ok(serde_json::json!({
                "path": MEMORY_PATH,
                "bytes": line.len(),
                "provenance": provenance,
            }))
        }
        Err(e) => ToolOutput::err(format!("écriture impossible : {}", e)),
    }
}

/// Rappelle les faits dont le contenu contient `query` (filtre littéral, comme
/// `FileMemoryStore::recall`).
pub fn recall(ctx: &ToolCtx, query: &str) -> ToolOutput {
    if query.trim().is_empty() {
        return ToolOutput::ok(serde_json::json!({
            "query": omega_types::redact_secrets(query),
            "results": [],
            "count": 0,
        }));
    }
    let abs = match memory_path(ctx) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    // Mémoire vide (fichier jamais écrit) : zéro résultat, pas une erreur.
    let text = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return ToolOutput::err(format!("lecture impossible : {}", e)),
    };
    // ponytail: scan linéaire + borne de sortie ; un index (FTS) quand la
    // mémoire dépassera la lecture complète.
    let mut results = Vec::new();
    for line in text.lines() {
        // Ligne corrompue (écriture interrompue) : ignorée, la mémoire survit.
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(content) = v.get("content").and_then(|c| c.as_str()) else {
            continue;
        };
        if !content.contains(query) {
            continue;
        }
        results.push(serde_json::json!({
            "content": omega_types::redact_secrets(content),
            "provenance": omega_types::redact_secrets(v.get("provenance").and_then(|p| p.as_str()).unwrap_or("")),
        }));
        if results.len() >= RECALL_LIMIT {
            break;
        }
    }
    ToolOutput::ok(serde_json::json!({
        "query": omega_types::redact_secrets(query),
        "results": results,
        "count": results.len(),
    }))
}

/// Supprime les faits dont le contenu contient `query` (filtre littéral).
pub fn forget(ctx: &ToolCtx, query: &str) -> ToolOutput {
    if query.trim().is_empty() {
        return ToolOutput::err("requête vide : rien à oublier");
    }
    let abs = match memory_path(ctx) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    let text = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return ToolOutput::err(format!("lecture impossible : {}", e)),
    };
    let mut kept: Vec<String> = Vec::new();
    let mut removed = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => {
                let content = v.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if content.contains(query) {
                    removed += 1;
                } else {
                    // Masque les secrets dans les faits conserves (reecriture).
                    kept.push(omega_types::redact_secrets(line));
                }
            }
            Err(_) => kept.push(omega_types::redact_secrets(line)),
        }
    }
    let mut out = String::new();
    for l in &kept {
        out.push_str(l);
        out.push('\n');
    }
    if let Err(e) = atomic_write(&abs, &out) {
        return ToolOutput::err(e);
    }
    ToolOutput::ok(serde_json::json!({
        "query": omega_types::redact_secrets(query),
        "removed": removed,
        "remaining": kept.len(),
    }))
}

/// Alias explicite (nom d'outil) pour `forget`.
pub fn memory_forget(ctx: &ToolCtx, query: &str) -> ToolOutput {
    forget(ctx, query)
}

fn unique_tmp_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let n = CTR.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos ^ (n.wrapping_mul(0x9e3779b97f4a7c15)) ^ (std::process::id() as u64)
}

fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), String> {
    let tmp_name = format!(
        "{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        unique_tmp_id()
    );
    let tmp = path.with_file_name(tmp_name);
    {
        use std::io::Write;
        let mut f =
            std::fs::File::create(&tmp).map_err(|e| format!("écriture impossible : {}", e))?;
        f.write_all(content.as_bytes())
            .map_err(|e| format!("écriture impossible : {}", e))?;
        f.sync_all()
            .map_err(|e| format!("écriture impossible : {}", e))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("écriture impossible : {}", e)
    })?;
    if let Some(parent) = path.parent() {
        if let Ok(d) = std::fs::File::open(parent) {
            let _ = d.sync_all();
        }
    }
    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(name: &str) -> ToolCtx {
        let p = std::env::temp_dir().join(format!("omega-mem-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        ToolCtx::new(&p)
    }

    #[test]
    fn store_then_recall_filters_by_content() {
        let c = ctx("roundtrip");
        assert!(store(&c, "le canari est vert", "test").is_ok());
        assert!(store(&c, "le chat dort", "test").is_ok());
        match recall(&c, "canari") {
            ToolOutput::Ok { content } => {
                let hits = content["results"].as_array().unwrap();
                assert_eq!(hits.len(), 1);
                assert_eq!(hits[0]["content"], "le canari est vert");
                assert_eq!(content["count"], 1);
            }
            _ => panic!("recall échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn recall_is_bounded() {
        let c = ctx("bound");
        for i in 0..(RECALL_LIMIT + 5) {
            assert!(store(&c, &format!("fait-{}", i), "test").is_ok());
        }
        match recall(&c, "fait-") {
            ToolOutput::Ok { content } => {
                assert_eq!(content["results"].as_array().unwrap().len(), RECALL_LIMIT);
            }
            _ => panic!("recall échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn store_refuses_empty_content_or_provenance() {
        let c = ctx("empty");
        assert!(!store(&c, "  ", "test").is_ok());
        assert!(!store(&c, "fait", "").is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn recall_on_missing_file_is_empty_and_skips_corrupt_lines() {
        let c = ctx("corrupt");
        match recall(&c, "x") {
            ToolOutput::Ok { content } => assert_eq!(content["count"], 0),
            _ => panic!("fichier absent = mémoire vide"),
        }
        std::fs::create_dir_all(c.root.join(".omega")).unwrap();
        std::fs::write(
            c.root.join(MEMORY_PATH),
            "pas du json\n{\"content\":\"fait lisible\",\"provenance\":\"t\"}\n",
        )
        .unwrap();
        match recall(&c, "fait") {
            ToolOutput::Ok { content } => {
                assert_eq!(content["count"], 1);
                assert_eq!(content["results"][0]["provenance"], "t");
            }
            _ => panic!("recall échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }
}
