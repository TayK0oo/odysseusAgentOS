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

/// MemoryStore simple : stockage en memoire + fichier JSONL pour persistance.
pub struct FileMemoryStore {
    path: std::path::PathBuf,
    facts: Mutex<Vec<String>>,
}

impl FileMemoryStore {
    pub fn new(path: impl Into<std::path::PathBuf>) -> OmegaResult<Self> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OmegaError::Io(format!("creation repertoire : {e}")))?;
        }
        let mut facts = Vec::new();
        if path.is_file() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| OmegaError::Io(format!("lecture fichier : {e}")))?;
            for line in content.lines() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(s) = v.as_str() {
                        facts.push(s.to_string());
                    }
                }
            }
        }
        Ok(Self {
            path,
            facts: Mutex::new(facts),
        })
    }
}

#[async_trait]
impl MemoryStore for FileMemoryStore {
    fn name(&self) -> &str {
        "file-memory"
    }
    async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
        let facts = self.facts.lock().unwrap();
        let results: Vec<String> = facts
            .iter()
            .filter(|f| f.contains(query))
            .take(limit)
            .cloned()
            .collect();
        Ok(results)
    }
    async fn remember(&self, fact: &str) -> OmegaResult<()> {
        self.facts.lock().unwrap().push(fact.to_string());
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| OmegaError::Io(format!("ouverture fichier : {e}")))?;
        use std::io::Write;
        writeln!(file, "{}", serde_json::to_string(fact).unwrap())
            .map_err(|e| OmegaError::Io(format!("ecriture fichier : {e}")))?;
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
