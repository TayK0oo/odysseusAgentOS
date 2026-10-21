//! `omega-tools` — portage Rust de `src/agent_tools/filesystem_tools.py`.
//!
//! Outil set : `read_file`, `write_file`, `edit_file`, `ls`, `glob`, `grep`,
//! `get_workspace`. Chaque fonction est une implémentation Rust pure ; le
//! registre (`ToolSurface`) est construit dans [`registry`].
#![allow(clippy::new_without_default)]

pub mod diff;
pub mod glob;
pub mod grep;
pub mod registry;
pub mod surface;
pub mod workspace;

use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

/// Bornes de sécurité (port de `agent_tools/__init__.py`).
pub const MAX_AGENT_ROUNDS: usize = 50;
pub const SHELL_TIMEOUT_SECS: u64 = 60;
pub const PYTHON_TIMEOUT_SECS: u64 = 30;
pub const MAX_DIFF_LINES: usize = 400;
pub const MAX_READ_BYTES: usize = 2_000_000;
/// Borne de longueur d'un chemin exigé par la jail (PATH_MAX + marge).
pub const MAX_PATH_CHARS: usize = 4096;

/// Résultat d'un appel d'outil.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status")]
pub enum ToolOutput {
    Ok { content: serde_json::Value },
    Error { message: String },
}

impl ToolOutput {
    pub fn ok(v: serde_json::Value) -> Self {
        ToolOutput::Ok { content: v }
    }
    pub fn err(msg: impl Into<String>) -> Self {
        ToolOutput::Error {
            message: msg.into(),
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, ToolOutput::Ok { .. })
    }
}

/// Contexte d'exécution (racine du workspace + politique).
pub struct ToolCtx {
    pub root: PathBuf,
    /// Chemins qui ne doivent jamais être lus/écrits (guard-fou).
    pub deny_paths: Vec<String>,
}

impl ToolCtx {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            deny_paths: vec![
                ".env".into(),
                ".ssh".into(),
                ".aws/credentials".into(),
                ".docker/config.json".into(),
                ".gnupg".into(),
                ".netrc".into(),
                "credentials".into(),
                "id_rsa".into(),
                "id_dsa".into(),
                "id_ecdsa".into(),
                "id_ed25519".into(),
                "*.pem".into(),
                "*.key".into(),
                "*.p12".into(),
                "*.pfx".into(),
                "*.keystore".into(),
            ],
        }
    }

    /// Résout un chemin relatif et refuse la sortie du workspace + la deny-list.
    ///
    /// **Symlinks** : la normalisation lexicale seule est contournable (un lien
    /// dans le workspace vers `/tmp` passait). On canonise donc le parent
    /// existant du chemin cible, puis on ré-ancre le nom de fichier final.
    /// (`ponytail:` on garde une seule résolution — `canonicalize` du parent —
    /// plutôt qu'un moteur de résolution complet ; suffisant pour un garde-fou.)
    pub fn resolve(&self, rel: &str) -> Result<PathBuf, String> {
        // Garde de surface avant tout contact avec le FS : octets nuls (API C,
        // elles échouent plus tard avec un message inexploitable), Unicode non
        // canonique (NFC/NFD : deux écritures d'un "même" fichier → confusion
        // de cible), chemins au-delà de PATH_MAX.
        if rel.contains('\0') {
            return Err("octet nul dans le chemin".to_string());
        }
        if rel.chars().count() > MAX_PATH_CHARS {
            return Err(format!(
                "chemin trop long : {} caracteres (max {})",
                rel.chars().count(),
                MAX_PATH_CHARS
            ));
        }
        if rel.nfc().collect::<String>() != rel {
            return Err("unicode non canonique (décomposition NFC/NFD)".to_string());
        }
        let raw = Path::new(rel);
        let abs = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            self.root.join(raw)
        };
        let lex = normalize(&abs);

        // Remonte jusqu'au premier composant existant, canonise, ré-ancre le reste.
        let mut anchor = lex.clone();
        while !anchor.exists() {
            if !anchor.pop() {
                break;
            }
        }
        let canon_anchor =
            std::fs::canonicalize(&anchor).map_err(|e| format!("chemin introuvable : {}", e))?;
        // `join("")` ajouterait un slash final (-> "Not a directory") : on
        // garde le chemin canonique tel quel quand il ne reste rien à ré-ancrer.
        let canon = match lex.strip_prefix(&anchor) {
            Ok(rest) if !rest.as_os_str().is_empty() => canon_anchor.join(rest),
            _ => canon_anchor.clone(),
        };

        let root_canon =
            std::fs::canonicalize(&self.root).unwrap_or_else(|_| normalize(&self.root));
        if !canon.starts_with(&root_canon) {
            return Err(format!("chemin hors workspace : {}", rel));
        }
        let s = canon.to_string_lossy().to_string();
        for d in &self.deny_paths {
            let hit = if let Some(suffix) = d.strip_prefix('*') {
                s.ends_with(suffix)
            } else if d.ends_with('/') {
                s.contains(d.as_str())
            } else {
                s == *d
                    || s.ends_with(&format!("/{}", d))
                    || canon
                        .components()
                        .any(|seg| seg.as_os_str() == std::ffi::OsStr::new(d))
            };
            if hit {
                return Err(format!("chemin protégé : {}", rel));
            }
        }
        Ok(canon)
    }
}

/// Normalisation lexicale (résout `.` et `..` sans toucher au FS).
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Port de `ReadFileTool.execute` : lecture avec borne de taille.
pub fn read_file(ctx: &ToolCtx, rel: &str) -> ToolOutput {
    let abs = match ctx.resolve(rel) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    match std::fs::read(&abs) {
        Ok(bytes) => {
            if bytes.len() > MAX_READ_BYTES {
                return ToolOutput::err(format!(
                    "fichier trop volumineux : {} octets (max {})",
                    bytes.len(),
                    MAX_READ_BYTES
                ));
            }
            match String::from_utf8(bytes) {
                Ok(text) => ToolOutput::ok(serde_json::json!({
                    "path": rel,
                    "content": text,
                    "lines": text.lines().count(),
                })),
                // Fichier binaire (cas limite plan §6) : contenu UTF-8 valide
                // tronque au premier octet invalide, marque binary+truncated —
                // jamais une erreur ni un contenu decode elle-meme.
                Err(e) => {
                    let valid = e.utf8_error().valid_up_to();
                    let bytes = e.into_bytes();
                    let text = String::from_utf8_lossy(&bytes[..valid]).into_owned();
                    ToolOutput::ok(serde_json::json!({
                        "path": rel,
                        "content": text,
                        "binary": true,
                        "truncated": true,
                    }))
                }
            }
        }
        Err(e) => ToolOutput::err(format!("lecture impossible : {}", e)),
    }
}

/// Port de `WriteFileTool.execute` : écriture (création dossiers).
pub fn write_file(ctx: &ToolCtx, rel: &str, content: &str) -> ToolOutput {
    let abs = match ctx.resolve(rel) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    if let Some(parent) = abs.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return ToolOutput::err(format!("création dossier impossible : {}", e));
        }
    }
    match std::fs::write(&abs, content) {
        Ok(()) => ToolOutput::ok(serde_json::json!({
            "path": rel,
            "bytes": content.len(),
            "lines": content.lines().count(),
        })),
        Err(e) => ToolOutput::err(format!("écriture impossible : {}", e)),
    }
}

/// Port de `EditFileTool.execute` : substitution exacte (old_string → new_string),
/// avec refus si l'ancre est absente ou ambiguë.
pub fn edit_file(ctx: &ToolCtx, rel: &str, old_string: &str, new_string: &str) -> ToolOutput {
    let abs = match ctx.resolve(rel) {
        Ok(p) => p,
        Err(e) => return ToolOutput::err(e),
    };
    let original = match std::fs::read_to_string(&abs) {
        Ok(s) => s,
        Err(e) => return ToolOutput::err(format!("lecture impossible : {}", e)),
    };
    let occurrences = original.matches(old_string).count();
    if occurrences == 0 {
        return ToolOutput::err("old_string introuvable dans le fichier");
    }
    if occurrences > 1 {
        return ToolOutput::err(format!(
            "old_string ambiguë : {} occurrences — préciser un contexte plus long",
            occurrences
        ));
    }
    let updated = original.replacen(old_string, new_string, 1);
    let d = diff::unified_diff(&original, &updated, rel);
    if let Err(e) = std::fs::write(&abs, &updated) {
        return ToolOutput::err(format!("écriture impossible : {}", e));
    }
    let mut payload = serde_json::json!({ "path": rel });
    if let Some(dobj) = d {
        payload["diff"] = serde_json::to_value(dobj).unwrap_or(serde_json::Value::Null);
    }
    ToolOutput::ok(payload)
}

/// Port de `LsTool.execute` : listage d'un répertoire (tri, type, taille).
pub fn list_dir(ctx: &ToolCtx, rel: &str) -> ToolOutput {
    let abs = if rel.is_empty() || rel == "." {
        normalize(&ctx.root)
    } else {
        match ctx.resolve(rel) {
            Ok(p) => p,
            Err(e) => return ToolOutput::err(e),
        }
    };
    let entries = match std::fs::read_dir(&abs) {
        Ok(e) => e,
        Err(e) => return ToolOutput::err(format!("listage impossible : {}", e)),
    };
    let mut items: Vec<serde_json::Value> = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = e.path().is_dir();
        let size = e.metadata().map(|m| m.len()).unwrap_or(0);
        items.push(serde_json::json!({ "name": name, "type": if is_dir {"dir"} else {"file"}, "size": size }));
    }
    items.sort_by(|a, b| {
        let ta = a["type"].as_str().unwrap_or("");
        let tb = b["type"].as_str().unwrap_or("");
        // "dir" < "file" lexicographiquement : les répertoires remontent en tête
        ta.cmp(tb).then_with(|| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or(""))
        })
    });
    ToolOutput::ok(serde_json::json!({ "path": rel, "entries": items, "count": items.len() }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_resolve(c: &ToolCtx, rel: &str) -> bool {
        c.resolve(rel).is_ok()
    }

    fn ctx_resolve_err(c: &ToolCtx, rel: &str) -> bool {
        c.resolve(rel).is_err()
    }

    fn ctx(name: &str) -> ToolCtx {
        let p = std::env::temp_dir().join(format!("omega-tools-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        ToolCtx::new(&p)
    }

    #[test]
    fn write_then_read_roundtrip() {
        let c = ctx("rw");
        assert!(write_file(&c, "a/b.txt", "hello\nworld").is_ok());
        let r = read_file(&c, "a/b.txt");
        match r {
            ToolOutput::Ok { content } => {
                assert_eq!(content["content"], "hello\nworld");
                assert_eq!(content["lines"], 2);
            }
            ToolOutput::Error { message } => panic!("lecture échouée : {}", message),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn read_missing_file_errors() {
        let c = ctx("missing");
        assert!(!read_file(&c, "nope.txt").is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn edit_replaces_and_diffs() {
        let c = ctx("edit");
        assert!(write_file(&c, "f.txt", "alpha\nbeta\ngamma").is_ok());
        let r = edit_file(&c, "f.txt", "beta", "BETA");
        match r {
            ToolOutput::Ok { content } => {
                assert_eq!(content["diff"]["added"], 1);
                assert_eq!(content["diff"]["removed"], 1);
            }
            _ => panic!("édition échouée"),
        }
        assert_eq!(
            std::fs::read_to_string(c.root.join("f.txt")).unwrap(),
            "alpha\nBETA\ngamma"
        );
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn edit_rejects_ambiguous_anchor() {
        let c = ctx("ambig");
        assert!(write_file(&c, "f.txt", "x\nx").is_ok());
        assert!(!edit_file(&c, "f.txt", "x", "y").is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn escapes_workspace_are_denied() {
        let c = ctx("escape");
        assert!(!write_file(&c, "../evil.txt", "x").is_ok());
        assert!(!read_file(&c, "../../etc/passwd").is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn jail_refuse_octets_nuls() {
        let c = ctx("nul");
        assert!(ctx_resolve_err(&c, "a\0b"));
        assert!(ctx_resolve_err(&c, "sous\0/rep/f.txt"));
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn jail_refuse_unicode_non_canonique() {
        // NFC = "é" précomposé ; NFD = "e" + combining. La même lettre visuelle,
        // deux formes : seule l'une d'elles doit passer.
        let c = ctx("unicode");
        let nfc = "caf\u{00e9}.txt"; // precompose (U+00E9)
        let nfd = "cafe\u{0301}.txt"; // decomposed (e + U+0301)
        assert_eq!(nfc.chars().count() + 1, nfd.chars().count());
        // L'une est NFC, l'autre non : la forme non canonique est refusee.
        assert!(ctx_resolve_err(&c, nfd) || ctx_resolve_err(&c, nfc));
        // Et les deux formes ne sont JAMAIS resolues en un seul fichier commun :
        // si les deux passaient, un modele pourrait viser un fichier hors
        // attente par confusion NFC/NFD.
        let forms = [nfd, nfc].iter().filter(|p| ctx_resolve(&c, p)).count();
        assert!(forms <= 1, "au moins une forme non canonique est passee");
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn jail_refuse_chemins_tres_longs() {
        let c = ctx("long");
        let long = format!("{}.txt", "a".repeat(4096));
        assert!(ctx_resolve_err(&c, &long));
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn deny_list_protects_secrets() {
        let c = ctx("secrets");
        std::fs::write(c.root.join(".env"), "SECRET=1").unwrap();
        assert!(!read_file(&c, ".env").is_ok());
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn binary_read_is_truncated_and_marked() {
        // (Cas limite plan §6) fichier binaire : pas d'erreur, pas un contenu
        // octet-alors-texte fantaisiste — un contenu UTF-8 tronqué, marque.
        let c = ctx("bin");
        let mut bytes = b"TXT".to_vec();
        bytes.push(0xFF); // invalide UTF-8 au milieu ; valides avant.
        std::fs::write(c.root.join("bin.dat"), &bytes).unwrap();
        match read_file(&c, "bin.dat") {
            ToolOutput::Ok { content } => {
                assert_eq!(content["binary"], true);
                assert_eq!(content["truncated"], true);
                assert_eq!(content["content"], "TXT");
            }
            _ => panic!("un fichier binaire ne doit pas renvoyer une erreur (plan §6)"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }

    #[test]
    fn list_dir_sorts_dirs_first() {
        let c = ctx("ls");
        std::fs::create_dir_all(c.root.join("zdir")).unwrap();
        std::fs::write(c.root.join("afile.txt"), "x").unwrap();
        match list_dir(&c, "") {
            ToolOutput::Ok { content } => {
                let e = content["entries"].as_array().unwrap();
                assert_eq!(e[0]["type"], "dir");
                assert_eq!(e[1]["type"], "file");
            }
            _ => panic!("listage échoué"),
        }
        let _ = std::fs::remove_dir_all(&c.root);
    }
}
