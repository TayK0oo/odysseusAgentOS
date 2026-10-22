//! Implémentations du port `EventStore` : mémoire (tests/embarqué) et fichier JSONL
//! append-only (`FileEventStore`) pour l'exécution durable.
//!
//! Contrat (voir `omega_ports::EventStore`) : `append` attribue un `seq` monotone par
//! run ; `read_run` restitue le journal dans l'ordre d'append. `FileEventStore` tolère
//! une dernière ligne tronquée (crash en cours d'écriture) en l'ignorant à la lecture.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use omega_ports::EventStore;
use omega_types::{DurableEvent, OmegaError, OmegaResult};

/// Store en mémoire : un journal par run, dans l'ordre d'append.
///
/// Utile pour les tests et l'embarqué (run éphémère sans fichier). N'est pas durable
/// au-delà de la durée de vie du process.
#[derive(Default)]
pub struct InMemoryEventStore {
    runs: Mutex<HashMap<String, Vec<DurableEvent>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventStore for InMemoryEventStore {
    fn name(&self) -> &str {
        "in-memory"
    }

    fn append(&self, event: DurableEvent) -> OmegaResult<u64> {
        // Même contrat que `FileEventStore` (cohérence des stores) : un run_id
        // invalide est refusé, même sans risque de traversal ici.
        if !is_valid_run_id(&event.run_id) {
            return Err(OmegaError::Config(format!(
                "identifiant de run invalide : {:?}",
                event.run_id
            )));
        }
        let mut runs = self.runs.lock().expect("journal empoisonné");
        let journal = runs.entry(event.run_id.clone()).or_default();
        let seq = journal.len() as u64;
        journal.push(event.with_seq(seq));
        Ok(seq)
    }

    fn read_run(&self, run_id: &str) -> OmegaResult<Vec<DurableEvent>> {
        if !is_valid_run_id(run_id) {
            return Err(OmegaError::Config(format!(
                "identifiant de run invalide : {run_id:?}"
            )));
        }
        Ok(self
            .runs
            .lock()
            .expect("journal empoisonné")
            .get(run_id)
            .cloned()
            .unwrap_or_default())
    }
}

/// Store durable : un fichier JSONL append-only par run (`<dir>/<run_id>.jsonl`).
///
/// Chaque événement est une ligne JSON. `seq` est attribué au nombre de lignes complètes
/// déjà présentes (monotone par run). La lecture ignore une dernière ligne malformée,
/// ce qui rend la reprise robuste à un crash en cours d'écriture.
pub struct FileEventStore {
    dir: PathBuf,
    /// Serre-file : `append` et `read_run` relisent/recalculent l'etat du fichier
    /// (read-modify-write). Sans verrou, deux appends concurrents sur le meme `run_id`
    /// attribueraient le meme `seq`. Serialise l'acces au fichier par store.
    lock: Mutex<()>,
}

impl FileEventStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            lock: Mutex::new(()),
        }
    }

    /// Répertoire racine des journaux (un fichier par run).
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn path_for(&self, run_id: &str) -> PathBuf {
        self.dir.join(format!("{run_id}.jsonl"))
    }
}

/// Identifiant de run sûr pour un nom de fichier : `[A-Za-z0-9_-]+`, 1–128 caractères.
///
/// `path_for` interpole le `run_id` dans un chemin : sans validation, un `run_id`
/// fourni par l'extérieur (`GET /api/trace/../../secret` décodé) sort du répertoire
/// des journaux (path traversal). `FileEventStore` refuse les identifiants invalides
/// (erreur typée, jamais de repli silencieux) ; le serveur les rejette en 400.
pub fn is_valid_run_id(run_id: &str) -> bool {
    !run_id.is_empty()
        && run_id.len() <= 128
        && run_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

impl EventStore for FileEventStore {
    fn name(&self) -> &str {
        "file"
    }

    fn append(&self, event: DurableEvent) -> OmegaResult<u64> {
        let _guard = self.lock.lock().expect("journal empoisonné");

        if !is_valid_run_id(&event.run_id) {
            return Err(OmegaError::Config(format!(
                "identifiant de run invalide : {:?}",
                event.run_id
            )));
        }

        fs::create_dir_all(&self.dir)
            .map_err(|e| OmegaError::Io(format!("création {} : {e}", self.dir.display())))?;

        let path = self.path_for(&event.run_id);
        let existing = if path.exists() {
            Some(
                fs::read(&path)
                    .map_err(|e| OmegaError::Io(format!("lecture {} : {e}", path.display())))?,
            )
        } else {
            None
        };

        let seq = existing
            .as_ref()
            .map(|b| b.iter().filter(|&&x| x == b'\n').count() as u64)
            .unwrap_or(0);

        let mut to_write: Vec<u8> = Vec::new();
        if let Some(bytes) = &existing {
            if !bytes.is_empty() && *bytes.last().unwrap() != b'\n' {
                // Termine une ligne tronquée (crash précédent) pour ne pas la fusionner
                // avec le prochain événement. Elle sera ignorée à la lecture (parse KO).
                to_write.push(b'\n');
            }
        }
        // P2-6 : masquage des secrets au point d'écriture unique (trace).
        let mut event = event;
        if let Some(op) = &event.operation_id {
            event.operation_id = Some(omega_core::redact_secrets(op));
        }
        omega_core::redact_json(&mut event.payload);
        let mut line = serde_json::to_vec(&event.with_seq(seq))
            .map_err(|e| OmegaError::Io(format!("sérialisation événement : {e}")))?;
        line.push(b'\n');
        to_write.extend_from_slice(&line);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| OmegaError::Io(format!("ouverture {} : {e}", path.display())))?;
        file.write_all(&to_write)
            .and_then(|_| file.flush())
            .map_err(|e| OmegaError::Io(format!("append {} : {e}", path.display())))?;

        Ok(seq)
    }

    fn read_run(&self, run_id: &str) -> OmegaResult<Vec<DurableEvent>> {
        let _guard = self.lock.lock().expect("journal empoisonné");

        if !is_valid_run_id(run_id) {
            return Err(OmegaError::Config(format!(
                "identifiant de run invalide : {run_id:?}"
            )));
        }

        let path = self.path_for(run_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| OmegaError::Io(format!("lecture {} : {e}", path.display())))?;
        let mut events = Vec::new();
        for line in content.lines() {
            // Une ligne malformée (écriture interrompue) est ignorée.
            if let Ok(mut ev) = serde_json::from_str::<DurableEvent>(line) {
                // P2-6 : masquage à la lecture (export/repr).
                if let Some(op) = &ev.operation_id {
                    ev.operation_id = Some(omega_core::redact_secrets(op));
                }
                omega_core::redact_json(&mut ev.payload);
                events.push(ev);
            }
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    static TMP_SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_dir(tag: &str) -> PathBuf {
        let n = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("omega-app-store-{tag}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ev(run_id: &str, name: &str) -> DurableEvent {
        DurableEvent::new(run_id, name, serde_json::json!({"op": name}))
            .with_operation_id(Some(name.to_string()))
    }

    #[test]
    fn in_memory_assigns_monotonic_seq_and_isolation() {
        let store = InMemoryEventStore::new();
        assert_eq!(store.append(ev("r1", "a")).unwrap(), 0);
        assert_eq!(store.append(ev("r1", "b")).unwrap(), 1);
        assert_eq!(store.append(ev("r2", "a")).unwrap(), 0);

        let r1 = store.read_run("r1").unwrap();
        assert_eq!(r1.len(), 2);
        assert_eq!(r1[0].seq, 0);
        assert_eq!(r1[1].seq, 1);
        assert!(store.read_run("absent").unwrap().is_empty());
    }

    #[test]
    fn file_store_round_trips_and_persists_across_instances() {
        let dir = temp_dir("roundtrip");
        let path = dir.clone();
        {
            let store = FileEventStore::new(&dir);
            assert_eq!(store.append(ev("run-1", "a")).unwrap(), 0);
            assert_eq!(store.append(ev("run-1", "b")).unwrap(), 1);
        }
        // Nouvelle instance sur le meme repertoire : les evenements persistent.
        let store2 = FileEventStore::new(&path);
        let journal = store2.read_run("run-1").unwrap();
        assert_eq!(journal.len(), 2);
        assert_eq!(journal[0].name, "a");
        assert_eq!(journal[0].operation_id.as_deref(), Some("a"));
        assert_eq!(journal[1].name, "b");
        assert_eq!(journal[1].seq, 1);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn run_id_validation_rejects_traversal_and_junk() {
        // (traversal) `..`, `/`, encodages décodés (`../`), vide, trop long.
        for bad in [
            "",
            "..",
            "../secret",
            "..\\secret",
            "a/b",
            "a b",
            "run;rm",
            &"x".repeat(129),
        ] {
            assert!(!is_valid_run_id(bad), "{bad:?}");
        }
        for ok in ["run-1", "live_1", "ABC123", "pipe-1", "route-trace-1"] {
            assert!(is_valid_run_id(ok), "{ok:?}");
        }
    }

    #[test]
    fn file_store_rejects_invalid_run_id_without_touching_disk() {
        use omega_ports::EventStore;
        let dir = temp_dir("badid");
        let store = FileEventStore::new(&dir);
        let err = store.append(ev("../evil", "a")).unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
        let err = store.read_run("../../etc").unwrap_err();
        assert!(matches!(err, OmegaError::Config(_)));
        // Aucun fichier créé hors du répertoire (ni dedans).
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_tolerates_truncated_last_line() {
        let dir = temp_dir("truncated");
        let store = FileEventStore::new(&dir);
        store.append(ev("run-1", "a")).unwrap();
        // Simule une ecriture interrompue : une ligne partielle sans newline final.
        let path = dir.join("run-1.jsonl");
        let mut f = OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{\"run_id\":\"run-1\",\"seq\":1,\"name\":\"b\"")
            .unwrap();

        // L'append suivant doit terminer la ligne tronquee et ecrire un evenement valide.
        let seq = store.append(ev("run-1", "c")).unwrap();
        assert_eq!(seq, 1); // une seule ligne complete precedente ("a")

        let journal = store.read_run("run-1").unwrap();
        // "a" (seq 0) + "c" (seq 1) ; "b" (tronque) est ignore.
        assert_eq!(journal.len(), 2);
        assert_eq!(journal[0].name, "a");
        assert_eq!(journal[1].name, "c");
        assert_eq!(journal[1].seq, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_seq_is_unique_under_concurrency() {
        // (B3) N threads × M appends sur le meme run_id -> seq uniques et monotones.
        use std::thread;

        let dir = temp_dir("concurrent");
        let store = Arc::new(FileEventStore::new(&dir));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let store = store.clone();
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    store
                        .append(ev("run-c", &format!("op-{i}")))
                        .expect("append concurrent");
                }
            }));
        }
        for handle in handles {
            handle.join().expect("thread d'append");
        }

        let journal = store.read_run("run-c").unwrap();
        assert_eq!(journal.len(), 400);

        let seqs: Vec<u64> = journal.iter().map(|e| e.seq).collect();
        let mut unique = seqs.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 400, "seq doivent etre uniques");

        for w in seqs.windows(2) {
            assert!(w[0] < w[1], "seq doit etre strictement croissant");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn event_store_is_object_safe_via_arc() {
        let _store: Arc<dyn EventStore> = Arc::new(InMemoryEventStore::new());
        let _file: Arc<dyn EventStore> = Arc::new(FileEventStore::new(temp_dir("dyn")));
    }
}
