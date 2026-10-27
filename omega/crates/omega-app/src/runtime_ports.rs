// ponytail: FileMemoryStore/SimpleHookRegistry/LocalSubagentRunner et leurs
// Contributor ne sont construits nulle part hors de ce module (adaptateurs « Phase B »
// jamais branchés dans omega-server) — suppression ou branchement en P1, cf. ADR-0004.
#![allow(dead_code)]

//! Adaptateurs runtime des ports orphelins (NOYAU Phase B, top-level).
//!
//! Opt-in via `AppBuilder::with_runtime_ports` : un adaptateur par defaut
//! pour `MemoryStore` (fichier JSONL), `Hook` (registre memoire) et
//! `SubagentRunner` (spawn local tokio). Les contributeurs utilisateur
//! gardent la priorite (compose dans l'ordre d'ajout).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use omega_plugin_api::{Contributor, InstallContext};
use omega_ports::{Hook, MemoryStore, SubagentRunner};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaError, OmegaResult};

/// MemoryStore simple : stockage JSONL append-only, **relu depuis le disque a
/// chaque `recall`** (un outil peut ecrire dans le meme fichier — un cache en
/// memoire rendrait la lecture incoherente avec l'ecriture).
///
/// Format accepte en lecture (compatibilite outil/injection) :
/// - une chaine JSON (`"fait"`) — forme historique ;
/// - un objet (`{"content": "...", "provenance": "[stated]"}`) — forme ecrite
///   par l'outil `memory_store` (`omega-tools`).
pub struct FileMemoryStore {
    path: std::path::PathBuf,
}

impl FileMemoryStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> OmegaResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OmegaError::Io(format!("creation repertoire : {e}")))?;
        }
        Ok(Self { path })
    }

    /// Chemin du magasin (le meme fichier doit servir a l'outil et a
    /// l'injection : un seul magasin, ADR-0023).
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Faits presents sur disque, dans l'ordre d'ecriture.
    fn read_facts(path: &std::path::Path) -> OmegaResult<Vec<String>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(OmegaError::Io(format!("lecture fichier : {e}"))),
        };
        let mut facts = Vec::new();
        for line in content.lines() {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                // Ligne corrompue (ecriture interrompue) : ignoree, la memoire survit.
                continue;
            };
            let fact = v
                .as_str()
                .map(str::to_string)
                .or_else(|| v.get("content").and_then(|c| c.as_str()).map(str::to_string))
                .or_else(|| v.get("text").and_then(|c| c.as_str()).map(str::to_string));
            if let Some(f) = fact {
                facts.push(f);
            }
        }
        Ok(facts)
    }
}

#[async_trait]
impl MemoryStore for FileMemoryStore {
    fn name(&self) -> &str {
        "file-memory"
    }
    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let facts = Self::read_facts(&self.path)?;
        let results: Vec<String> = facts
            .into_iter()
            .filter(|f| f.contains(query))
            .take(limit)
            .collect();
        Ok(results)
    }
    async fn remember(&self, fact: &str) -> OmegaResult<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| OmegaError::Io(format!("ouverture fichier : {e}")))?;
        use std::io::Write;
        writeln!(file, "{}", serde_json::to_string(fact).unwrap())
            .map_err(|e| OmegaError::Io(format!("ecriture fichier : {e}")))?;
        file.sync_all()
            .map_err(|e| OmegaError::Io(format!("flush fichier : {e}")))?;
        Ok(())
    }
}

/// Hook simple : registre en memoire des hooks.
pub struct SimpleHookRegistry {
    hooks: Mutex<Vec<Box<dyn Hook>>>,
}

impl Default for SimpleHookRegistry {
    fn default() -> Self {
        Self {
            hooks: Mutex::new(Vec::new()),
        }
    }
}

impl SimpleHookRegistry {
    pub fn add(&self, hook: Box<dyn Hook>) {
        self.hooks.lock().unwrap().push(hook);
    }
}

#[async_trait]
impl Hook for SimpleHookRegistry {
    fn name(&self) -> &str {
        "hook-registry"
    }
}

/// SubagentRunner local : spawn via tokio.
pub struct LocalSubagentRunner {
    running: Mutex<Vec<tokio::task::JoinHandle<OmegaResult<String>>>>,
}

impl Default for LocalSubagentRunner {
    fn default() -> Self {
        Self {
            running: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl SubagentRunner for LocalSubagentRunner {
    fn name(&self) -> &str {
        "local-subagents"
    }
    async fn spawn(&self, spec: &str) -> OmegaResult<String> {
        let spec = spec.to_string();
        let handle = tokio::spawn(async move {
            // ponytail: execution locale simulee ; provider externe (Phase D
            // ExternalSubagentRunner) pour le reel.
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            Ok(format!("subagent:{}", spec))
        });
        self.running.lock().unwrap().push(handle);
        Ok("spawned".to_string())
    }
}

/// Contributeur "runtime ports" par defaut : un adaptateur par port orphelin.
pub struct RuntimePortsContributor {
    store: Arc<dyn MemoryStore>,
    hooks: Arc<SimpleHookRegistry>,
    runner: Arc<dyn SubagentRunner>,
}

impl RuntimePortsContributor {
    /// Adaptateurs par defaut ; `memory_path` personnalise le fichier JSONL.
    pub fn new(memory_path: std::path::PathBuf) -> OmegaResult<Self> {
        Ok(Self {
            store: Arc::new(FileMemoryStore::new(memory_path)?),
            hooks: Arc::new(SimpleHookRegistry::default()),
            runner: Arc::new(LocalSubagentRunner::default()),
        })
    }
}

impl Contributor for RuntimePortsContributor {
    fn name(&self) -> &str {
        "runtime-ports"
    }
    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        builder
            .register_dyn(
                Manifest::new("file", Capability::MemoryStore, "1"),
                self.store.clone(),
            )
            .map(|_| ())?;
        builder
            .register_dyn(
                Manifest::new("registry", Capability::Hook, "1"),
                self.hooks.clone(),
            )
            .map(|_| ())?;
        builder
            .register_dyn(
                Manifest::new("local", Capability::SubagentRunner, "1"),
                self.runner.clone(),
            )
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_ports::MemoryStore;

    #[tokio::test]
    async fn file_memory_store_roundtrip() {
        let dir = std::env::temp_dir().join(format!("omega-rt-{}", std::process::id()));
        let store = FileMemoryStore::new(dir.join("facts.jsonl")).unwrap();
        store.remember("fact-alpha").await.unwrap();
        let hits = store.recall("alpha", 5).await.unwrap();
        assert_eq!(hits, vec!["fact-alpha".to_string()]);
    }

    #[test]
    fn contributor_registers_three_ports() {
        let dir = std::env::temp_dir().join(format!("omega-rt-reg-{}", std::process::id()));
        let c = RuntimePortsContributor::new(dir.join("facts.jsonl")).unwrap();
        let mut b = RegistryBuilder::default();
        let ctx = InstallContext {
            config: omega_config::LayeredConfig::new().merged(),
        };
        c.contribute(&mut b, &ctx).unwrap();
        // Registre finalise sans erreur : preuve que les 3 capacites
        // (MemoryStore, Hook, SubagentRunner) s'ajoutent au builder.
        let _reg = b.build();
    }
}
