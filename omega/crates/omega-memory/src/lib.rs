//! `omega-memory` — portage Rust de `services/memory/` (odysseusAgentOS).
//!
//! Adaptateurs de port :
//! - [`JsonlMemoryStore`] : `MemoryStore` (stockage JSONL + recherche Jaccard/BM25)
//! - [`InMemoryMemoryStore`] : adaptateur « faux » en mémoire (A1)
//! - [`FileSkillStore`] : `SkillStore` (SKILL.md avec frontmatter YAML)
#![allow(clippy::new_without_default)]

pub mod skill;

use async_trait::async_trait;
use omega_ports::MemoryStore;
use omega_types::OmegaResult;
use omega_types::{redact_json, redact_secrets};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Provenance canonique d'un souvenir.
pub fn normalize_provenance(p: &str) -> String {
    let t = p.trim().to_lowercase();
    let t = t
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim()
        .to_string();
    match t.as_str() {
        "stated" | "observed" | "inferred" => t,
        _ => "stated".to_string(),
    }
}

pub fn provenance_tag(p: &str) -> String {
    format!("[{}]", normalize_provenance(p))
}

pub fn confidence_for(provenance: &str) -> f64 {
    match normalize_provenance(provenance).as_str() {
        "stated" => 1.0,
        "observed" => 0.7,
        "inferred" => 0.4,
        _ => 0.5,
    }
}

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
    // --- P2-2 extensions ---
    #[serde(default = "default_provenance")]
    pub provenance: String,
    #[serde(default = "default_confidence_placeholder")]
    pub confidence: f64,
    #[serde(default)]
    pub classification: String,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub status: String, // "committed" | "draft"
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub superseded_by: Option<String>,
}

fn default_provenance() -> String {
    "stated".to_string()
}
fn default_confidence_placeholder() -> f64 {
    // filled in new(); for deserialization default
    1.0
}

impl MemoryRecord {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        let prov = "stated".to_string();
        Self {
            id: id.into(),
            text: text.into(),
            timestamp: now_secs(),
            session_id: None,
            metadata: serde_json::Map::new(),
            provenance: prov.clone(),
            confidence: confidence_for(&prov),
            classification: "fact".to_string(),
            ttl_seconds: None,
            namespace: None,
            user_id: None,
            status: "committed".to_string(),
            draft: false,
            version: 1,
            supersedes: None,
            superseded_by: None,
        }
    }

    pub fn new_with(
        id: impl Into<String>,
        text: impl Into<String>,
        provenance: &str,
        classification: &str,
        namespace: Option<String>,
        ttl_seconds: Option<i64>,
    ) -> Self {
        let prov = normalize_provenance(provenance);
        Self {
            id: id.into(),
            text: text.into(),
            timestamp: now_secs(),
            session_id: None,
            metadata: serde_json::Map::new(),
            provenance: prov.clone(),
            confidence: confidence_for(&prov),
            classification: if classification.is_empty() {
                "fact".to_string()
            } else {
                classification.to_string()
            },
            ttl_seconds,
            namespace: namespace.clone(),
            user_id: namespace,
            status: "committed".to_string(),
            draft: false,
            version: 1,
            supersedes: None,
            superseded_by: None,
        }
    }

    pub fn is_expired(&self, now: i64) -> bool {
        match self.ttl_seconds {
            Some(ttl) if ttl >= 0 => now >= self.timestamp + ttl,
            _ => false,
        }
    }

    pub fn is_expired_now(&self) -> bool {
        self.is_expired(now_secs())
    }

    pub fn is_draft(&self) -> bool {
        self.draft || self.status == "draft"
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Un souvenir avec score de pertinence.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredMemory {
    pub record: MemoryRecord,
    pub score: f64,
}

/// Tokenisation simple.
pub fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| ".,!?\";:()[]".contains(c)))
        .filter(|w| w.len() > 1)
        .map(|w| w.to_string())
        .collect()
}

pub fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

// ---------- Storage helpers (shared) ----------

fn redact_record(r: &MemoryRecord) -> MemoryRecord {
    let mut r = r.clone();
    r.text = redact_secrets(&r.text);
    let mut v = serde_json::Value::Object(r.metadata.clone());
    redact_json(&mut v);
    if let serde_json::Value::Object(m) = v {
        r.metadata = m;
    }
    r
}

fn load_records(path: &Path) -> Vec<MemoryRecord> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<MemoryRecord>(l).ok())
        .map(|r| {
            // normalize provenance/confidence on load
            let mut r = r;
            r.provenance = normalize_provenance(&r.provenance);
            if r.confidence == 0.0 {
                r.confidence = confidence_for(&r.provenance);
            }
            if r.classification.is_empty() {
                r.classification = "fact".to_string();
            }
            if r.status.is_empty() {
                r.status = if r.draft {
                    "draft".to_string()
                } else {
                    "committed".to_string()
                };
            }
            r.draft = r.draft || r.status == "draft";
            r
        })
        .collect()
}

fn save_records(path: &Path, records: &[MemoryRecord]) -> OmegaResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let mut out = String::new();
    for r in records {
        let r = redact_record(r);
        out.push_str(
            &serde_json::to_string(&r).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?,
        );
        out.push('\n');
    }
    // atomic write + fsync
    let tmp = path.with_extension("tmp");
    {
        use std::io::Write;
        let mut f =
            std::fs::File::create(&tmp).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        f.write_all(out.as_bytes())
            .map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
        f.sync_all()
            .map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
    }
    std::fs::rename(&tmp, path).map_err(|e| omega_types::OmegaError::Io(e.to_string()))?;
    // fsync dir
    if let Some(parent) = path.parent() {
        if let Ok(d) = std::fs::File::open(parent) {
            let _ = d.sync_all();
        }
    }
    // remove tmp if any
    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

// ---------- JsonlMemoryStore ----------

pub struct JsonlMemoryStore {
    path: PathBuf,
    drafts_path: PathBuf,
    namespace: Option<String>,
    // optional in-memory vector index (simple)
    vectors: Mutex<Vec<(String, Vec<f32>)>>,
}

impl JsonlMemoryStore {
    pub fn vector_count(&self) -> usize {
        self.vectors.lock().map(|v| v.len()).unwrap_or(0)
    }
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let dir = data_dir.as_ref();
        Self {
            path: dir.join("memories.jsonl"),
            drafts_path: dir.join("memory_drafts.jsonl"),
            namespace: None,
            vectors: Mutex::new(Vec::new()),
        }
    }

    pub fn with_namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    pub fn new_with_namespace(data_dir: impl AsRef<Path>, ns: impl Into<String>) -> Self {
        Self::new(data_dir).with_namespace(ns)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    // core load/save
    fn load_all(&self) -> Vec<MemoryRecord> {
        load_records(&self.path)
    }
    fn save_all(&self, records: &[MemoryRecord]) -> OmegaResult<()> {
        save_records(&self.path, records)
    }
    fn load_drafts(&self) -> Vec<MemoryRecord> {
        load_records(&self.drafts_path)
    }
    fn save_drafts(&self, records: &[MemoryRecord]) -> OmegaResult<()> {
        save_records(&self.drafts_path, records)
    }

    fn filter_ns(&self, r: &MemoryRecord) -> bool {
        match &self.namespace {
            Some(ns) => {
                r.namespace.as_deref() == Some(ns.as_str())
                    || r.user_id.as_deref() == Some(ns.as_str())
            }
            None => true,
        }
    }

    fn is_visible(&self, r: &MemoryRecord, now: i64) -> bool {
        if r.is_draft() || r.status != "committed" {
            return false;
        }
        if r.superseded_by.is_some() {
            return false;
        }
        if r.is_expired(now) {
            return false;
        }
        self.filter_ns(r)
    }

    /// Toutes les mémoires.
    pub fn all(&self, limit: usize) -> Vec<MemoryRecord> {
        self.load_all().into_iter().take(limit).collect()
    }
    pub fn all_records(&self, limit: usize) -> Vec<MemoryRecord> {
        self.all(limit)
    }

    /// Recherche Jaccard.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search_at(query, top_k, now_secs())
    }
    pub fn search_at(&self, query: &str, top_k: usize, now: i64) -> Vec<ScoredMemory> {
        let q = tokenize(query);
        let mut hits: Vec<ScoredMemory> = self
            .load_all()
            .into_iter()
            .filter(|r| self.is_visible(r, now))
            .map(|r| {
                let score = jaccard(&q, &tokenize(&r.text));
                ScoredMemory { record: r, score }
            })
            .filter(|s| s.score > 0.0)
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }

    // BM25-ish search (alias to jaccard search for compatibility)
    pub fn search_bm25(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub fn bm25_search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search_bm25(query, top_k)
    }
    pub fn search_text(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }

    // vector search (optional; simple fallback to jaccard)
    pub fn search_vector(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub fn vector_search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search_vector(query, top_k)
    }

    // filtering by classification
    pub fn search_by_classification(
        &self,
        query: &str,
        classification: &str,
        top_k: usize,
    ) -> Vec<ScoredMemory> {
        let mut hits = self.search(query, top_k * 4);
        hits.retain(|s| s.record.classification == classification);
        hits.truncate(top_k);
        hits
    }

    pub fn recall_records(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        self.search(query, limit)
            .into_iter()
            .map(|s| s.record)
            .collect()
    }

    pub fn delete(&self, id: &str) -> bool {
        let all2 = self.load_all();
        let existed2 = all2.iter().any(|m| m.id == id);
        let remaining2: Vec<_> = all2.into_iter().filter(|m| m.id != id).collect();
        let _ = self.save_all(&remaining2);
        let drafts = self.load_drafts();
        let remaining_d: Vec<_> = drafts.into_iter().filter(|m| m.id != id).collect();
        let _ = self.save_drafts(&remaining_d);
        if let Ok(mut v) = self.vectors.lock() {
            v.retain(|(idv, _)| idv != id);
        }
        existed2
    }

    pub async fn forget(&self, id: &str) -> OmegaResult<bool> {
        // remove from committed store
        let all2 = self.load_all();
        let existed2 = all2.iter().any(|m| m.id == id);
        let remaining2: Vec<_> = all2.into_iter().filter(|m| m.id != id).collect();
        self.save_all(&remaining2)?;
        // drafts
        let drafts = self.load_drafts();
        let remaining_d: Vec<_> = drafts.into_iter().filter(|m| m.id != id).collect();
        self.save_drafts(&remaining_d)?;
        // vector index
        if let Ok(mut v) = self.vectors.lock() {
            v.retain(|(idv, _)| idv != id);
        }
        Ok(existed2)
    }

    // staging
    pub async fn propose_draft(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft_full(fact, "stated", "fact", self.namespace.clone(), None)
            .await
    }
    pub async fn propose_draft_full(
        &self,
        fact: &str,
        provenance: &str,
        classification: &str,
        namespace: Option<String>,
        ttl_seconds: Option<i64>,
    ) -> OmegaResult<String> {
        let id = format!("d{:x}", now_nanos());
        let mut rec = MemoryRecord::new_with(
            id.clone(),
            fact,
            provenance,
            classification,
            namespace,
            ttl_seconds,
        );
        rec.status = "draft".to_string();
        rec.draft = true;
        let mut drafts = self.load_drafts();
        drafts.push(rec);
        self.save_drafts(&drafts)?;
        Ok(id)
    }
    // aliases
    pub async fn stage(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft(fact).await
    }
    pub async fn stage_draft(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft(fact).await
    }

    pub async fn commit_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        let mut drafts = self.load_drafts();
        let pos = drafts.iter().position(|d| d.id == draft_id);
        let Some(pos) = pos else {
            return Ok(false);
        };
        let mut rec = drafts.remove(pos);
        rec.status = "committed".to_string();
        rec.draft = false;
        // assign new id? keep same
        let mut all = self.load_all();
        all.push(rec);
        self.save_all(&all)?;
        self.save_drafts(&drafts)?;
        Ok(true)
    }
    pub async fn validate(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }
    pub async fn validate_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }
    pub async fn commit(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }

    pub async fn reject_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        let drafts = self.load_drafts();
        let existed = drafts.iter().any(|d| d.id == draft_id);
        let remaining: Vec<_> = drafts.into_iter().filter(|d| d.id != draft_id).collect();
        self.save_drafts(&remaining)?;
        Ok(existed)
    }
    pub async fn reject(&self, draft_id: &str) -> OmegaResult<bool> {
        self.reject_draft(draft_id).await
    }

    // remember variants
    pub async fn remember(&self, fact: &str) -> OmegaResult<()> {
        self.remember_full(fact, "stated", "fact", self.namespace.clone(), None, None)
            .await
    }
    pub async fn remember_full(
        &self,
        fact: &str,
        provenance: &str,
        classification: &str,
        namespace: Option<String>,
        ttl_seconds: Option<i64>,
        session_id: Option<String>,
    ) -> OmegaResult<()> {
        // contradictories: if a record with same text/classification/namespace exists, mark superseded
        let prov = normalize_provenance(provenance);
        let ns = namespace.or_else(|| self.namespace.clone());
        let id = format!("m{:x}", now_nanos());
        let mut all = self.load_all();
        // mark existing identical text as superseded
        for r in all.iter_mut() {
            if r.text == fact
                && r.classification == classification
                && r.namespace == ns
                && r.superseded_by.is_none()
            {
                r.superseded_by = Some(id.clone());
            }
        }
        let mut rec = MemoryRecord::new_with(
            id.clone(),
            fact,
            &prov,
            classification,
            ns.clone(),
            ttl_seconds,
        );
        rec.session_id = session_id;
        // supersedes: find last committed same classification/namespace? keep simple: none
        // version increment
        let maxv = all.iter().map(|r| r.version).max().unwrap_or(0);
        rec.version = maxv + 1;
        all.push(rec);
        self.save_all(&all)?;
        Ok(())
    }
    // aliases for remember
    pub async fn remember_fact(&self, fact: &str) -> OmegaResult<()> {
        self.remember(fact).await
    }
    pub async fn add(&self, fact: &str) -> OmegaResult<()> {
        self.remember(fact).await
    }
    pub async fn store_fact(&self, fact: &str) -> OmegaResult<()> {
        self.remember(fact).await
    }
    pub async fn remember_observed(&self, fact: &str) -> OmegaResult<()> {
        self.remember_full(fact, "observed", "fact", self.namespace.clone(), None, None)
            .await
    }
    pub async fn remember_inferred(&self, fact: &str) -> OmegaResult<()> {
        self.remember_full(fact, "inferred", "fact", self.namespace.clone(), None, None)
            .await
    }
    pub async fn remember_with_provenance(&self, fact: &str, provenance: &str) -> OmegaResult<()> {
        self.remember_full(fact, provenance, "fact", self.namespace.clone(), None, None)
            .await
    }
    pub async fn remember_with_classification(
        &self,
        fact: &str,
        classification: &str,
    ) -> OmegaResult<()> {
        self.remember_full(
            fact,
            "stated",
            classification,
            self.namespace.clone(),
            None,
            None,
        )
        .await
    }
    pub async fn remember_with_ttl(&self, fact: &str, ttl_seconds: i64) -> OmegaResult<()> {
        self.remember_full(
            fact,
            "stated",
            "fact",
            self.namespace.clone(),
            Some(ttl_seconds),
            None,
        )
        .await
    }
    pub async fn remember_with_namespace(&self, fact: &str, namespace: &str) -> OmegaResult<()> {
        self.remember_full(
            fact,
            "stated",
            "fact",
            Some(namespace.to_string()),
            None,
            None,
        )
        .await
    }

    // supersede explicit
    pub async fn supersede(&self, old_id: &str, new_fact: &str) -> OmegaResult<String> {
        let new_id = format!("m{:x}", now_nanos());
        let mut all = self.load_all();
        // mark old
        for r in all.iter_mut() {
            if r.id == old_id {
                r.superseded_by = Some(new_id.clone());
            }
        }
        let ns = self.namespace.clone();
        let mut rec = MemoryRecord::new_with(new_id.clone(), new_fact, "stated", "fact", ns, None);
        rec.supersedes = Some(old_id.to_string());
        all.push(rec);
        self.save_all(&all)?;
        Ok(new_id)
    }

    // search aliases already; recall string
    pub fn search_records(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        self.recall_records(query, limit)
    }
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
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
        self.remember(fact).await
    }
    async fn forget(&self, id: &str) -> OmegaResult<bool> {
        self.forget(id).await
    }
    async fn propose_draft(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft(fact).await
    }
    async fn commit_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }
    async fn reject_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        self.reject_draft(draft_id).await
    }
}

// ---------- InMemoryMemoryStore (fake) ----------

pub struct InMemoryMemoryStore {
    records: Mutex<Vec<MemoryRecord>>,
    drafts: Mutex<Vec<MemoryRecord>>,
    namespace: Option<String>,
}

impl InMemoryMemoryStore {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            drafts: Mutex::new(Vec::new()),
            namespace: None,
        }
    }
    pub fn with_namespace(ns: impl Into<String>) -> Self {
        Self {
            records: Mutex::new(Vec::new()),
            drafts: Mutex::new(Vec::new()),
            namespace: Some(ns.into()),
        }
    }
    // delegate-like methods mirroring JsonlMemoryStore minimally
    pub async fn remember(&self, fact: &str) -> OmegaResult<()> {
        let prov = "stated";
        let ns = self.namespace.clone();
        let id = format!("m{:x}", now_nanos());
        let mut recs = self.records.lock().unwrap();
        for r in recs.iter_mut() {
            if r.text == fact {
                r.superseded_by = Some(id.clone());
            }
        }
        let rec = MemoryRecord::new_with(id, fact, prov, "fact", ns, None);
        recs.push(rec);
        Ok(())
    }
    pub fn load_all(&self) -> Vec<MemoryRecord> {
        self.records.lock().unwrap().clone()
    }
    pub fn save_all(&self, recs: Vec<MemoryRecord>) {
        *self.records.lock().unwrap() = recs;
    }
    pub fn all(&self, limit: usize) -> Vec<MemoryRecord> {
        self.records
            .lock()
            .unwrap()
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }
    pub fn search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        let q = tokenize(query);
        let now = now_secs();
        let mut hits: Vec<ScoredMemory> = self
            .records
            .lock()
            .unwrap()
            .iter()
            .filter(|r| {
                !(r.draft || r.status == "draft") && r.superseded_by.is_none() && !r.is_expired(now)
            })
            .filter(|r| match &self.namespace {
                Some(ns) => {
                    r.namespace.as_deref() == Some(ns.as_str())
                        || r.user_id.as_deref() == Some(ns.as_str())
                }
                None => true,
            })
            .map(|r| ScoredMemory {
                record: r.clone(),
                score: jaccard(&q, &tokenize(&r.text)),
            })
            .filter(|s| s.score > 0.0)
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }
    pub fn search_bm25(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub fn bm25_search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub fn search_vector(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub fn vector_search(&self, query: &str, top_k: usize) -> Vec<ScoredMemory> {
        self.search(query, top_k)
    }
    pub async fn forget(&self, id: &str) -> OmegaResult<bool> {
        let mut recs = self.records.lock().unwrap();
        let existed = recs.iter().any(|r| r.id == id);
        recs.retain(|r| r.id != id);
        let mut dr = self.drafts.lock().unwrap();
        dr.retain(|r| r.id != id);
        Ok(existed)
    }
    pub fn delete(&self, id: &str) -> bool {
        let mut recs = self.records.lock().unwrap();
        let existed = recs.iter().any(|r| r.id == id);
        recs.retain(|r| r.id != id);
        let mut dr = self.drafts.lock().unwrap();
        dr.retain(|r| r.id != id);
        existed
    }
    pub async fn propose_draft(&self, fact: &str) -> OmegaResult<String> {
        let id = format!("d{:x}", now_nanos());
        let ns = self.namespace.clone();
        let mut rec = MemoryRecord::new_with(id.clone(), fact, "stated", "fact", ns, None);
        rec.status = "draft".to_string();
        rec.draft = true;
        self.drafts.lock().unwrap().push(rec);
        Ok(id)
    }
    pub async fn stage(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft(fact).await
    }
    pub async fn commit_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        let mut dr = self.drafts.lock().unwrap();
        let pos = dr.iter().position(|d| d.id == draft_id);
        let Some(pos) = pos else {
            return Ok(false);
        };
        let mut rec = dr.remove(pos);
        rec.status = "committed".to_string();
        rec.draft = false;
        self.records.lock().unwrap().push(rec);
        Ok(true)
    }
    pub async fn validate(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }
    pub async fn reject_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        let mut dr = self.drafts.lock().unwrap();
        let existed = dr.iter().any(|d| d.id == draft_id);
        dr.retain(|d| d.id != draft_id);
        Ok(existed)
    }
    pub async fn reject(&self, draft_id: &str) -> OmegaResult<bool> {
        self.reject_draft(draft_id).await
    }
    pub async fn remember_full(
        &self,
        fact: &str,
        provenance: &str,
        classification: &str,
        namespace: Option<String>,
        ttl_seconds: Option<i64>,
        _session_id: Option<String>,
    ) -> OmegaResult<()> {
        let prov = normalize_provenance(provenance);
        let ns = namespace.or_else(|| self.namespace.clone());
        let id = format!("m{:x}", now_nanos());
        let mut recs = self.records.lock().unwrap();
        for r in recs.iter_mut() {
            if r.text == fact && r.classification == classification && r.namespace == ns {
                r.superseded_by = Some(id.clone());
            }
        }
        let mut rec = MemoryRecord::new_with(id, fact, &prov, classification, ns, ttl_seconds);
        rec.provenance = prov;
        recs.push(rec);
        Ok(())
    }
    // recall string
    pub fn recall_records(&self, query: &str, limit: usize) -> Vec<MemoryRecord> {
        self.search(query, limit)
            .into_iter()
            .map(|s| s.record)
            .collect()
    }
}

impl Default for InMemoryMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryStore for InMemoryMemoryStore {
    fn name(&self) -> &str {
        "in-memory-memory-store"
    }
    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        Ok(self
            .search(query, limit)
            .into_iter()
            .map(|s| s.record.text)
            .collect())
    }
    async fn remember(&self, fact: &str) -> OmegaResult<()> {
        self.remember(fact).await
    }
    async fn forget(&self, id: &str) -> OmegaResult<bool> {
        self.forget(id).await
    }
    async fn propose_draft(&self, fact: &str) -> OmegaResult<String> {
        self.propose_draft(fact).await
    }
    async fn commit_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        self.commit_draft(draft_id).await
    }
    async fn reject_draft(&self, draft_id: &str) -> OmegaResult<bool> {
        self.reject_draft(draft_id).await
    }
}

// re-export
pub use InMemoryMemoryStore as FakeMemoryStore;
