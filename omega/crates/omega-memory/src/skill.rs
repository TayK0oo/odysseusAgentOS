//! `skill` — portage Rust de `services/memory/skill_format.py` + `skills.py`.
//!
//! Format : `data/skills/<category>/<name>/SKILL.md` avec frontmatter YAML et
//! corps structuré (When to Use / Procedure / Pitfalls / Verification).
//! Sidecar `_usage.json` pour les compteurs `uses` / `last_used`.

use async_trait::async_trait;
use omega_ports::SkillStore;
use omega_types::OmegaResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Slugifie un nom (port de `skill_format.py::slugify`).
pub fn slugify(text: &str) -> String {
    let s: String = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let s = s.trim_matches('-').to_string();
    // collapse des tirets multiples
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_dash { out.push(c); }
            prev_dash = true;
        } else {
            out.push(c);
            prev_dash = false;
        }
    }
    if out.is_empty() { "skill".into() } else { out }
}

/// Une compétence (SKILL.md).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub requires_toolsets: Vec<String>,
    #[serde(default)]
    pub fallback_for_toolsets: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub teacher_model: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub when_to_use: String,
    #[serde(default)]
    pub procedure: Vec<String>,
    #[serde(default)]
    pub pitfalls: Vec<String>,
    #[serde(default)]
    pub verification: Vec<String>,
    #[serde(default)]
    pub body_extra: String,
    /// Sidecar (pas dans SKILL.md)
    #[serde(default)]
    pub uses: u32,
    #[serde(default)]
    pub last_used: Option<i64>,
    /// Chemin sur disque
    #[serde(default, skip_serializing)]
    pub path: Option<String>,
}

fn default_version() -> String { "1.0.0".into() }
fn default_category() -> String { "general".into() }
fn default_status() -> String { "draft".into() }
fn default_confidence() -> f64 { 0.8 }
fn default_source() -> String { "learned".into() }

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            version: default_version(),
            category: default_category(),
            tags: Vec::new(),
            platforms: Vec::new(),
            requires_toolsets: Vec::new(),
            fallback_for_toolsets: Vec::new(),
            status: default_status(),
            confidence: default_confidence(),
            source: default_source(),
            teacher_model: None,
            owner: None,
            created: String::new(),
            when_to_use: String::new(),
            procedure: Vec::new(),
            pitfalls: Vec::new(),
            verification: Vec::new(),
            body_extra: String::new(),
            uses: 0,
            last_used: None,
            path: None,
        }
    }
}

/// Parse un frontmatter YAML minimal (`---` délimité, `clé: valeur`, listes `[a, b]`).
pub fn parse_frontmatter(text: &str) -> (BTreeMap<String, String>, String) {
    let mut fm = BTreeMap::new();
    let mut body = String::new();
    let mut in_fm = false;
    let mut seen_first = false;

    for line in text.lines() {
        if line.trim() == "---" {
            if !seen_first {
                in_fm = true;
                seen_first = true;
            } else {
                in_fm = false;
            }
            continue;
        }
        if in_fm {
            if let Some((k, v)) = line.split_once(':') {
                fm.insert(k.trim().to_string(), v.trim().to_string());
            }
        } else if seen_first {
            body.push_str(line);
            body.push('\n');
        }
    }
    (fm, body)
}

/// Émet un frontmatter YAML minimal.
pub fn emit_frontmatter(fm: &BTreeMap<String, String>) -> String {
    let mut out = String::from("---\n");
    for (k, v) in fm {
        out.push_str(&format!("{}: {}\n", k, v));
    }
    out.push_str("---\n");
    out
}

/// Split le corps en sections connues (port de `parse_body`).
pub fn parse_body(body: &str) -> (String, Vec<String>, Vec<String>, Vec<String>, String) {
    let mut when_to_use = String::new();
    let mut procedure = Vec::new();
    let mut pitfalls = Vec::new();
    let mut verification = Vec::new();
    let mut extra = String::new();

    let mut current: Option<String> = None;
    let mut buf: Vec<String> = Vec::new();
    let mut extras: Vec<String> = Vec::new();

    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("##") {
            let heading = rest.trim().to_lowercase();
            // flush section précédente
            flush(&current, &buf, &mut when_to_use, &mut procedure, &mut pitfalls, &mut verification);
            buf.clear();
            current = Some(match heading.as_str() {
                "when to use" => "when_to_use".into(),
                "procedure" => "procedure".into(),
                "pitfalls" => "pitfalls".into(),
                "verification" => "verification".into(),
                _ => "__unknown__".into(),
            });
            continue;
        }
        match current.as_deref() {
            None | Some("__unknown__") => extras.push(line.to_string()),
            _ => buf.push(line.to_string()),
        }
    }
    flush(&current, &buf, &mut when_to_use, &mut procedure, &mut pitfalls, &mut verification);

    let ex = extras.join("\n").trim().to_string();
    if !ex.is_empty() { extra = ex; }

    (when_to_use, procedure, pitfalls, verification, extra)
}

fn flush(
    current: &Option<String>,
    buf: &[String],
    when_to_use: &mut String,
    procedure: &mut Vec<String>,
    pitfalls: &mut Vec<String>,
    verification: &mut Vec<String>,
) {
    let text = buf.join("\n").trim().to_string();
    match current.as_deref() {
        Some("when_to_use") => *when_to_use = text,
        Some("procedure") => *procedure = parse_list_lines(&text),
        Some("pitfalls") => *pitfalls = parse_list_lines(&text),
        Some("verification") => *verification = parse_list_lines(&text),
        _ => {}
    }
}

/// Extrait les lignes de liste (puce / numérotée) — port de `_parse_list_lines`.
pub fn parse_list_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() { continue; }
        let stripped = t
            .trim_start_matches(|c| "-*•".contains(c))
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .trim_start_matches(|c| ". )".contains(c))
            .trim();
        if !stripped.is_empty() {
            out.push(stripped.to_string());
        }
    }
    out
}

impl Skill {
    /// Construit une Skill depuis le markdown complet.
    pub fn from_markdown(text: &str) -> Self {
        let (fm, body) = parse_frontmatter(text);
        let (when_to_use, procedure, pitfalls, verification, extra) = parse_body(&body);
        Self {
            name: fm.get("name").cloned().unwrap_or_default(),
            description: fm.get("description").cloned().unwrap_or_default(),
            version: fm.get("version").cloned().unwrap_or_else(default_version),
            category: fm.get("category").cloned().unwrap_or_else(default_category),
            tags: parse_list_inline(fm.get("tags")),
            platforms: parse_list_inline(fm.get("platforms")),
            requires_toolsets: parse_list_inline(fm.get("requires_toolsets")),
            fallback_for_toolsets: parse_list_inline(fm.get("fallback_for_toolsets")),
            status: fm.get("status").cloned().unwrap_or_else(default_status),
            confidence: fm
                .get("confidence")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.8),
            source: fm.get("source").cloned().unwrap_or_else(default_source),
            teacher_model: fm.get("teacher_model").cloned(),
            owner: fm.get("owner").cloned(),
            created: fm.get("created").cloned().unwrap_or_default(),
            when_to_use,
            procedure,
            pitfalls,
            verification,
            body_extra: extra,
            uses: 0,
            last_used: None,
            path: None,
        }
    }

    /// Sérialise en markdown SKILL.md.
    pub fn to_markdown(&self) -> String {
        let mut fm: BTreeMap<String, String> = BTreeMap::new();
        fm.insert("name".into(), self.name.clone());
        fm.insert("description".into(), self.description.clone());
        fm.insert("version".into(), self.version.clone());
        fm.insert("category".into(), self.category.clone());
        if !self.tags.is_empty() {
            fm.insert("tags".into(), format!("[{}]", self.tags.join(", ")));
        }
        fm.insert("status".into(), self.status.clone());
        fm.insert("confidence".into(), format!("{:.3}", self.confidence));
        fm.insert("source".into(), self.source.clone());
        if let Some(o) = &self.owner {
            fm.insert("owner".into(), o.clone());
        }

        let mut out = emit_frontmatter(&fm);
        if !self.when_to_use.is_empty() {
            out.push_str(&format!("\n## When to Use\n{}\n", self.when_to_use));
        }
        if !self.procedure.is_empty() {
            out.push_str("\n## Procedure\n");
            for (i, p) in self.procedure.iter().enumerate() {
                out.push_str(&format!("{}. {}\n", i + 1, p));
            }
        }
        if !self.pitfalls.is_empty() {
            out.push_str("\n## Pitfalls\n");
            for p in &self.pitfalls {
                out.push_str(&format!("- {}\n", p));
            }
        }
        if !self.verification.is_empty() {
            out.push_str("\n## Verification\n");
            for v in &self.verification {
                out.push_str(&format!("- {}\n", v));
            }
        }
        if !self.body_extra.is_empty() {
            out.push_str(&format!("\n{}\n", self.body_extra));
        }
        out
    }
}

fn parse_list_inline(v: Option<&String>) -> Vec<String> {
    match v {
        None => Vec::new(),
        Some(s) => s
            .trim()
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect(),
    }
}

/// Store de compétences sur disque — adaptateur du port `SkillStore`.
pub struct FileSkillStore {
    root: PathBuf,
    usage: std::sync::RwLock<BTreeMap<String, (u32, Option<i64>)>>,
}

impl FileSkillStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let root = data_dir.as_ref().join("skills");
        std::fs::create_dir_all(&root).ok();
        Self {
            root,
            usage: std::sync::RwLock::new(BTreeMap::new()),
        }
    }

    /// Charge le sidecar `_usage.json`.
    pub fn load_usage(&self) {
        let p = self.root.join("_usage.json");
        if let Ok(content) = std::fs::read_to_string(p) {
            if let Ok(map) = serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&content) {
                let mut w = self.usage.write().unwrap();
                for (k, v) in map {
                    let uses = v.get("uses").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                    let last = v.get("last_used").and_then(|x| x.as_i64());
                    w.insert(k, (uses, last));
                }
            }
        }
    }

    /// Sauvegarde le sidecar.
    pub fn save_usage(&self) {
        let w = self.usage.read().unwrap();
        let mut map = serde_json::Map::new();
        for (k, (uses, last)) in w.iter() {
            map.insert(
                k.clone(),
                serde_json::json!({ "uses": uses, "last_used": last }),
            );
        }
        let p = self.root.join("_usage.json");
        std::fs::write(p, serde_json::to_string_pretty(&map).unwrap_or_default()).ok();
    }

    /// Incrémente le compteur d'usage.
    pub fn record_use(&self, skill: &str) {
        let mut w = self.usage.write().unwrap();
        let e = w.entry(skill.to_string()).or_insert((0, None));
        e.0 += 1;
        e.1 = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        );
    }

    /// Tous les skills sur disque.
    pub fn all(&self) -> Vec<Skill> {
        let mut out = Vec::new();
        let Ok(cats) = std::fs::read_dir(&self.root) else {
            return out;
        };
        for cat in cats.flatten() {
            if !cat.path().is_dir() { continue; }
            let Ok(skills) = std::fs::read_dir(cat.path()) else { continue; };
            for sk in skills.flatten() {
                let md = sk.path().join("SKILL.md");
                if md.exists() {
                    if let Ok(text) = std::fs::read_to_string(&md) {
                        let mut s = Skill::from_markdown(&text);
                        s.path = Some(md.to_string_lossy().to_string());
                        let u = self.usage.read().unwrap();
                        if let Some((uses, last)) = u.get(&s.name) {
                            s.uses = *uses;
                            s.last_used = *last;
                        }
                        out.push(s);
                    }
                }
            }
        }
        out
    }

    /// Écrit un skill sur disque (`<category>/<name>/SKILL.md`).
    pub fn save(&self, skill: &Skill) -> OmegaResult<()> {
        let dir = self.root.join(&skill.category).join(&skill.name);
        std::fs::create_dir_all(&dir)
            .map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        let path = dir.join("SKILL.md");
        std::fs::write(&path, skill.to_markdown())
            .map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl SkillStore for FileSkillStore {
    fn name(&self) -> &str {
        "file-skill-store"
    }

    async fn discover(&self) -> OmegaResult<Vec<String>> {
        Ok(self.all().into_iter().map(|s| s.name).collect())
    }

    async fn load(&self, id: &str) -> OmegaResult<String> {
        self.all()
            .into_iter()
            .find(|s| s.name == id)
            .map(|s| s.to_markdown())
            .ok_or_else(|| omega_types::OmegaError::Io(format!("skill not found: {}", id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes() {
        assert_eq!(slugify("Prospection Commerciale"), "prospection-commerciale");
        assert_eq!(slugify("  Hello   World!! "), "hello-world");
        assert_eq!(slugify(""), "skill");
        assert_eq!(slugify("a/b\\c"), "a-b-c");
    }

    #[test]
    fn frontmatter_roundtrip() {
        let md = "---\nname: test\ndescription: Une skill\nversion: 2.0.0\nconfidence: 0.9\n---\n\n## When to Use\nQuand il faut\n\n## Procedure\n1. Premier pas\n2. Deuxième\n\n## Pitfalls\n- Piège 1\n";
        let (fm, body) = parse_frontmatter(md);
        assert_eq!(fm.get("name").unwrap(), "test");
        assert_eq!(fm.get("version").unwrap(), "2.0.0");
        assert!(body.contains("When to Use"));
    }

    #[test]
    fn skill_from_markdown_parses_sections() {
        let md = "---\nname: prospection\ndescription: Trouver des clients\nversion: 1.0.0\n---\n\n## When to Use\nQuand prospecter\n\n## Procedure\n1. Chercher\n2. Contacter\n\n## Pitfalls\n- Ne pas spammer\n\n## Verification\n- Vérifier le taux\n";
        let s = Skill::from_markdown(md);
        assert_eq!(s.name, "prospection");
        assert_eq!(s.when_to_use, "Quand prospecter");
        assert_eq!(s.procedure, vec!["Chercher", "Contacter"]);
        assert_eq!(s.pitfalls, vec!["Ne pas spammer"]);
        assert_eq!(s.verification, vec!["Vérifier le taux"]);
    }

    #[test]
    fn skill_to_markdown_roundtrip() {
        let mut s = Skill {
            name: "test".into(),
            description: "desc".into(),
            when_to_use: "Quand".into(),
            procedure: vec!["a".into(), "b".into()],
            pitfalls: vec!["p".into()],
            ..Default::default()
        };
        s.tags = vec!["x".into()];
        let md = s.to_markdown();
        let back = Skill::from_markdown(&md);
        assert_eq!(back.name, "test");
        assert_eq!(back.procedure, vec!["a", "b"]);
        assert_eq!(back.pitfalls, vec!["p"]);
        assert_eq!(back.when_to_use, "Quand");
    }

    #[tokio::test]
    async fn file_skill_store_saves_and_discovers() {
        let dir = std::env::temp_dir().join("omega-skill-store-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let store = FileSkillStore::new(&dir);
        let s = Skill {
            name: "ma-skill".into(),
            category: "sales".into(),
            description: "test".into(),
            when_to_use: "toujours".into(),
            ..Default::default()
        };
        store.save(&s).unwrap();

        let found = store.discover().await.unwrap();
        assert_eq!(found, vec!["ma-skill".to_string()]);

        let loaded = store.load("ma-skill").await.unwrap();
        assert!(loaded.contains("ma-skill"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn usage_counter_increments() {
        let dir = std::env::temp_dir().join("omega-skill-usage-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let store = FileSkillStore::new(&dir);
        store.record_use("x");
        store.record_use("x");
        let w = store.usage.read().unwrap();
        assert_eq!(w.get("x").unwrap().0, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
