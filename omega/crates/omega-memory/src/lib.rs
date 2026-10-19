//! `omega-memory` — portage Rust de `services/memory/` (odysseusAgentOS).
//!
//! Deux adaptateurs de port :
//! - [`JsonlMemoryStore`] : `MemoryStore` (stockage JSONL + recherche Jaccard)
//! - [`FileSkillStore`] : `SkillStore` (SKILL.md avec frontmatter YAML)
#![allow(clippy::new_without_default)]

pub mod skill;

use async_trait::async_trait;
use omega_ports::MemoryStore;
use omega_types::OmegaResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Un souvenir persisté.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    pub id: String,
    pub text: String,
    pub timestamp: i64,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl MemoryRecord {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            session_id: None,
            metadata: serde_json::Map::new(),
        }
    }
}

/// Un souvenir avec score de pertinence.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredMemory {
    pub record: MemoryRecord,
    pub score: f64,
}

/// Tokenisation simple (mots > 1 char, ponctuation retirée) — port de
/// `services/memory/skills.py::_tokenize`.
pub fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| ".,!?\";:()[]".contains(c)))
        .filter(|w| w.len() > 1)
        .map(|w| w.to_string())
        .collect()
}

/// Similarité Jaccard — port de `skills.py::_jaccard`.
pub fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 { 0.0 } else { inter as f64 / union as f64 }
}

/// Store mémoire JSONL — adaptateur du port `MemoryStore`.
pub struct JsonlMemoryStore {
    path: PathBuf,
}

impl JsonlMemoryStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join("memories.jsonl"),
        }
    }

    fn load_all(&self) -> Vec<MemoryRecord> {
        let Ok(content) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str::<MemoryRecord>(l).ok())
            .collect()
    }

    fn save_all(&self, records: &[MemoryRecord]) -> OmegaResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let mut out = String::new();
        for r in records {
            out.push_str(&serde_json::to_string(r).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?);
            out.push('\n');
        }
        // Écriture atomique : write to temp then rename (pattern `core/atomic_io.py`).
        let tmp = self.path.with_extension("jsonl.tmp");
        std::fs::write(&tmp, out).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        std::fs::rename(&tmp, &self.path).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        Ok(())
    }

    /// Toutes les mémoires (équivalent `MemoryService.get_all`).
    pub fn all(&self, limit: usize) -> Vec<MemoryRecord> {
        self.load_all().into_iter().take(limit).collect()
    }

    /// Recherche par pertinence Jaccard (équivalent `MemoryService.recall`).
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        let q = tokenize(query);
        let mut hits: Vec<ScoredMemory> = self
            .load_all()
            .into_iter()
            .map(|r| {
                let score = jaccard(&q, &tokenize(&r.text));
                ScoredMemory { record: r, score }
            })
            .filter(|s| s.score > 0.0)
            .collect();
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(top_k);
        hits
    }

    /// Supprime par id (équivalent `MemoryService.delete`).
    pub fn delete(&self, id: &str) -> bool {
        let all = self.load_all();
        let remaining: Vec<_> = all.iter().filter(|m| m.id != id).cloned().collect();
        if remaining.len() == all.len() {
            return false;
        }
        self.save_all(&remaining).is_ok()
    }
}

#[async_trait]
impl MemoryStore for JsonlMemoryStore {
    fn name(&self) -> &str {
        "jsonl-memory-store"
    }

    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        Ok(self
            .search(query, limit)
            .into_iter()
            .map(|s| s.record.text)
            .collect())
    }

    async fn remember(&self, fact: &str) -> OmegaResult<()> {
        let mut all = self.load_all();
        let id = format!("m{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0));
        all.push(MemoryRecord::new(id, fact));
        self.save_all(&all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("omega-memory-test-{}", name));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn tokenize_filters_short_words() {
        let t = tokenize("Le chat dort sur le canapé");
        assert!(t.contains("chat"));
        assert!(t.contains("dort"));
        assert!(t.contains("le")); // len("le") == 2 > 1 -> conservé
        let t2 = tokenize("a b chat");
        assert!(!t2.contains("a")); // 1 char -> filtré
    }

    #[test]
    fn jaccard_empty_is_zero() {
        let a: HashSet<String> = HashSet::new();
        let b = tokenize("hello world");
        assert_eq!(jaccard(&a, &b), 0.0);
    }

    #[test]
    fn jaccard_identical_is_one() {
        let a = tokenize("prospection commerciale IA");
        assert!((jaccard(&a, &a) - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn remember_then_recall() {
        let dir = tmpdir("remember");
        let store = JsonlMemoryStore::new(&dir);
        store.remember("User prefers dark mode").await.unwrap();
        store.remember("Le chat dort sur le canapé").await.unwrap();

        let hits = store.recall("dark mode", 5).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("dark mode"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn search_ranks_by_score() {
        let dir = tmpdir("search");
        let store = JsonlMemoryStore::new(&dir);
        store.remember("prospection commerciale avec IA").await.unwrap();
        store.remember("recette de cuisine italienne").await.unwrap();

        let hits = store.search("prospection IA", 2);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].score > 0.0);
        assert!(hits[0].record.text.contains("prospection"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_removes_record() {
        let dir = tmpdir("delete");
        let store = JsonlMemoryStore::new(&dir);
        let rec = MemoryRecord::new("id1", "hello world test");
        store.save_all(&[rec.clone()]).unwrap();
        assert!(store.delete("id1"));
        assert!(!store.delete("id1")); // déjà supprimé
        assert!(store.all(10).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
