//! Tests adversariaux — `omega-python-bridge`.
//!
//! Ce crate **exécute du code Python** (subprocess). C'est le vecteur de sécurité
//! le plus sensible du projet : chaque test ci-dessous attaque un mode d'échappement.
//! Un test qui échoue = menace réelle.

use omega_python_bridge::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, PythonAgentBridge, PythonBridge, PythonToolBridge};
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Crée un script Python temporaire.
fn script(name: &str, body: &str) -> PathBuf {
    // Répertoire NOMINATIF : les tests tournent en parallèle, un répertoire
    // partagé faisait s'effacer les scripts les uns des autres.
    let dir = std::env::temp_dir().join(format!("omega-bridge-adv-{}", name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{}.py", name));
    std::fs::write(&p, body).unwrap();
    p
}

fn runtime() -> (&'static str, bool) {
    for c in ["python3", "python"] {
        if Command::new(c).arg("--version").output().is_ok() {
            return (c, true);
        }
    }
    ("python3", false)
}

// ═══════════════════════════════════════════════════════════════════════════
// T1 — Fuite de processus : le timeout tue-t-il l'enfant ?
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_timeout_kills_child_process() {
    let (py, ok) = runtime();
    if !ok { return; }
    let pidfile = std::env::temp_dir().join("omega-bridge-timeout.pid");
    let _ = std::fs::remove_file(&pidfile);

    // Le script écrit son PID puis dort 30 s. La question n'est PAS "a-t-il
    // tourné" (il a forcément tourné) mais "survit-il au timeout ?" — on vérifie
    // donc que le PID n'est plus vivant après le retour du bridge.
    let s = script("sleeper", &format!(
        "import sys, os, time\n\
         sys.stdin.readline()\n\
         open({:?}, 'w').write(str(os.getpid()))\n\
         sys.stdout.flush()\n\
         time.sleep(30)\n",
        pidfile.to_string_lossy()
    ));

    let bridge = PythonBridge::new(py, &s.to_string_lossy(), 800);
    let t0 = Instant::now();
    let res = bridge.call("noop", json!({})).await;
    let elapsed = t0.elapsed();

    assert!(res.is_err(), "un script qui dort doit produire un timeout");
    assert!(
        elapsed < Duration::from_secs(5),
        "le timeout (800 ms) n'a pas court-circuité : bridge a attendu {:?}",
        elapsed
    );

    let pid: i32 = std::fs::read_to_string(&pidfile)
        .expect("le script n'a pas écrit son PID")
        .trim()
        .parse()
        .expect("PID illisible");

    // Laisse le temps au kill() de se propager.
    tokio::time::sleep(Duration::from_millis(600)).await;

    // `/proc/<pid>` reste présent pour un processus ZOMBIE (terminé mais non
    // récolté) : lire l'entrée n'a donc pas valeur de preuve. Il faut lire
    // l'état réel dans /proc/<pid>/status.
    let state = std::fs::read_to_string(format!("/proc/{}/status", pid))
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("State:"))
                .map(|l| l.split_whitespace().nth(1).unwrap_or("").to_string())
        });
    let truly_alive = match state.as_deref() {
        None => false,              // gone du tout -> mort
        Some("Z") => false,         // zombie -> terminated
        Some(_) => true,            // R / S / D / T -> vivant
    };
    assert!(
        !truly_alive,
        "THREAT OPEN: le processus Python (pid {}) a survécu au timeout (état {:?})",
        pid,
        state
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T2 — Débordement de sortie : stdout non borné
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_unbounded_stdout_does_not_exhaust_memory() {
    let (py, ok) = runtime();
    if !ok { return; }
    // 200 Mo sur une seule "ligne" (sans \n) -> read_line accumule tout en RAM.
    let s = script("flood", "import sys\nsys.stdin.readline()\nsys.stdout.write('A' * (200*1024*1024))\n");
    let bridge = PythonBridge::new(py, &s.to_string_lossy(), 3_000);
    let _ = bridge.call("noop", json!({})).await; // ne doit pas paniquer ni OOM
}

// ═══════════════════════════════════════════════════════════════════════════
// T3 — Pollution du protocole : sortie non-JSON
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_non_json_stdout_fails_cleanly() {
    let (py, ok) = runtime();
    if !ok { return; }
    // Python écrit une trace/warning avant le JSON -> la 1re ligne n'est pas du JSON.
    let s = script(
        "noisy",
        "import sys, json\n\
         sys.stdin.readline()\n\
         sys.stderr.write('warning: deprecation\\n')\n\
         print('WARNING: module already imported')\n\
         sys.stdout.flush()\n\
         print(json.dumps({'jsonrpc':'2.0','result':'ok','error':None,'id':1}))\n",
    );
    let bridge = PythonBridge::new(py, &s.to_string_lossy(), 5_000);
    match bridge.call("noop", json!({})).await {
        Ok(v) => assert_eq!(v, "ok", "la vraie réponse doit être lue"),
        Err(e) => panic!(
            "THREAT OPEN: le bridge n'a pas récupéré après une ligne non-JSON en tête : {}",
            e
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// T4 — Stdout muet : le script ne répond jamais et ne ferme pas stdin
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_silent_process_times_out_not_hangs() {
    let (py, ok) = runtime();
    if !ok { return; }
    let s = script("silent", "import time\ntime.sleep(60)\n");
    let bridge = PythonBridge::new(py, &s.to_string_lossy(), 700);
    let t0 = Instant::now();
    let r = bridge.call("noop", json!({})).await;
    assert!(r.is_err());
    assert!(t0.elapsed() < Duration::from_secs(4), "le bridge a bloqué");
}

// ═══════════════════════════════════════════════════════════════════════════
// T5 — Requête giganticque : bornage des params
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_huge_params_are_rejected_or_bounded() {
    let (py, ok) = runtime();
    if !ok { return; }
    let s = script("echo", "import sys, json\nsys.stdin.readline()\nprint(json.dumps({'jsonrpc':'2.0','result':'ok','error':None,'id':1}))\n");
    let bridge = PythonBridge::new(py, &s.to_string_lossy(), 5_000);
    let huge = "A".repeat(50 * 1024 * 1024); // 50 Mo
    let _ = bridge.call("noop", json!({ "blob": huge })).await;
}

// ═══════════════════════════════════════════════════════════════════════════
// T6 — Chemin d'interpréteur hostile (argument injection)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_interpreter_path_is_not_shell_interpreted() {
    // Un chemin contenant des métacaractères shell ne doit PAS être passé au shell.
    // Vérifie que le binaire est bien exec'd (pas de fork /bin/sh).
    let evil = "/nonexistent/python; touch /tmp/omega-bridge-pwned";
    let bridge = PythonBridge::new(evil, "/tmp/x.py", 500);
    let r = bridge.call("noop", json!({})).await;
    assert!(r.is_err(), "un binaire inexistant doit échouer");
    assert!(
        !Path::new("/tmp/omega-bridge-pwned").exists(),
        "THREAT OPEN: le chemin a été interprété par un shell (injection de commande)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// T7 — Réponse JSON-RPC malformée / IDs divergents
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn atk_malformed_response_is_rejected() {
    for body in [
        "",
        "{",
        "null",
        "[]",
        "{\"jsonrpc\":\"1.0\"}",
        "{\"result\":\"sans id\"}",
    ] {
        let r: Result<JsonRpcResponse, _> = serde_json::from_str(body);
        assert!(r.is_err(), "une réponse malformée doit être refusée : {:?}", body);
    }
}

#[test]
fn atk_error_response_is_surfaced_not_swallowed() {
    let body = r#"{"jsonrpc":"2.0","result":null,"error":{"code":-32601,"message":"Method not found"},"id":1}"#;
    let r: JsonRpcResponse = serde_json::from_str(body).unwrap();
    assert!(r.result.is_none());
    let e: &JsonRpcError = r.error.as_ref().unwrap();
    assert_eq!(e.code, -32601);
}

#[test]
fn atk_request_id_is_incrementable() {
    // Deux requêtes doivent être distinguables (le bridge les numérote).
    let a = JsonRpcRequest::new("m", json!({}), 1);
    let b = JsonRpcRequest::new("m", json!({}), 2);
    assert_ne!(a.id, b.id);
    assert_eq!(a.jsonrpc, "2.0");
}

// ═══════════════════════════════════════════════════════════════════════════
// T8 — Les ponts specialised doivent-ils valider la méthode ?
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn atk_specialised_bridges_use_fixed_methods() {
    let (py, ok) = runtime();
    if !ok { return; }
    let s = script(
        "spy",
        "import sys, json\n\
         req = json.loads(sys.stdin.readline())\n\
         open('/tmp/omega-bridge-method.log','a').write(req['method'] + '\\n')\n\
         print(json.dumps({'jsonrpc':'2.0','result':'ok','error':None,'id':1}))\n",
    );
    let _ = std::fs::remove_file("/tmp/omega-bridge-method.log");

    PythonAgentBridge::new(py, &s.to_string_lossy()).run("goal").await.ok();
    PythonToolBridge::new(py, &s.to_string_lossy())
        .execute("read_file", json!({}))
        .await
        .ok();

    let log = std::fs::read_to_string("/tmp/omega-bridge-method.log").unwrap_or_default();
    let methods: Vec<&str> = log.lines().collect();
    assert!(
        methods.iter().any(|m| m.contains("agent_loop")),
        "PythonAgentBridge doit appeler agent_loop.run, vu {:?}",
        methods
    );
    assert!(
        methods.iter().any(|m| m.contains("tool.execute")),
        "PythonToolBridge doit appeler tool.execute, vu {:?}",
        methods
    );
}