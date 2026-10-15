//! Puits de télémétrie : `StdoutSink` (live) et `FileSink` (JSONL append-only).
//!
//! Les deux implémentent le port `omega_ports::TelemetrySink` (régime *ensemble*).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use omega_ports::TelemetrySink;
use omega_types::TelemetryEvent;

/// Émet chaque événement sur stdout (une ligne JSON). Pour le live / debug.
#[derive(Debug, Default)]
pub struct StdoutSink;

impl StdoutSink {
    pub fn new() -> Self {
        Self
    }
}

impl TelemetrySink for StdoutSink {
    fn name(&self) -> &str {
        "stdout"
    }

    fn emit(&self, event: TelemetryEvent) {
        if let Ok(line) = serde_json::to_string(&event) {
            println!("{line}");
        }
    }
}

/// Émet chaque événement dans un fichier JSONL (une ligne par événement).
///
/// Le fichier est ouvert en append à chaque `emit` (coût assumé Phase 1) sous un
/// `Mutex` : les émissions concurrentes ne s'entrelacent jamais.
#[derive(Debug)]
pub struct FileSink {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileSink {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Relit tous les événements émis (une ligne JSON par événement).
    pub fn read_all(&self) -> Vec<TelemetryEvent> {
        let content = std::fs::read_to_string(&self.path).unwrap_or_default();
        content
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }
}

impl TelemetrySink for FileSink {
    fn name(&self) -> &str {
        "file"
    }

    fn emit(&self, event: TelemetryEvent) {
        let _guard = self.lock.lock().expect("sink empoisonne");
        let mut line = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        line.push('\n');
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = f.write_all(line.as_bytes());
            let _ = f.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn ev(name: &str) -> TelemetryEvent {
        TelemetryEvent {
            name: name.to_string(),
            fields: BTreeMap::new().into_iter().collect(),
        }
    }

    #[test]
    fn stdout_sink_does_not_panic() {
        let s = StdoutSink::new();
        assert_eq!(s.name(), "stdout");
        s.emit(ev("omega.run.step"));
    }

    #[test]
    fn file_sink_round_trips() {
        let dir = std::env::temp_dir().join(format!("omega-otel-sink-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let sink = FileSink::new(dir.join("otel.jsonl"));
        sink.emit(ev("omega.router.choose"));
        sink.emit(ev("omega.tool.call"));
        let all = sink.read_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "omega.router.choose");
        assert_eq!(all[1].name, "omega.tool.call");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_sink_is_object_safe() {
        let _s: std::sync::Arc<dyn TelemetrySink> = std::sync::Arc::new(StdoutSink::new());
    }
}
