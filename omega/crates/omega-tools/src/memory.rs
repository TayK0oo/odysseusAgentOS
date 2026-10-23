//! `memory` — outils mémoire de la surface (T1 : écriture dans le workspace).
//!
//! Même forme que `FileMemoryStore` (`omega-app`) : JSONL append-only, une
//! ligne par fait. Ici le fait garde sa provenance ; le fichier vit dans la
//! jail de [`ToolCtx`], donc `resolve` applique confinement et deny-list.

use super::{ToolCtx, ToolOutput};

/// Fichier de mémoire, relatif à la racine du workspace.
pub const MEMORY_PATH: &str = ".omega/memory.jsonl";

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
    let abs = match ctx.resolve(MEMORY_PATH) {
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
            let _ = f.flush();
            ToolOutput::ok(serde_json::json!({
                "path": MEMORY_PATH,
                "bytes": line.len(),
            }))
        }
        Err(e) => ToolOutput::err(format!("écriture impossible : {}", e)),
    }
}

/// Rappelle les faits dont le contenu contient `query` (filtre littéral, comme
/// `FileMemoryStore::recall`).
pub fn recall(ctx: &ToolCtx, query: &str) -> ToolOutput {
    let abs = match ctx.resolve(MEMORY_PATH) {
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
            "content": content,
            "provenance": v.get("provenance").and_then(|p| p.as_str()).unwrap_or(""),
        }));
        if results.len() >= RECALL_LIMIT {
            break;
        }
    }
    ToolOutput::ok(serde_json::json!({
        "query": query,
        "results": results,
        "count": results.len(),
    }))
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
