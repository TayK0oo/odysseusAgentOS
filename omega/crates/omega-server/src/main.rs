//! Binaire de bord Omega : sert l'API HTTP de l'UI Cockpit.
//!
//! Configuration par l'environnement (`OMEGA_PORT`/`PORT`, `OLLAMA_URL`,
//! `OLLAMA_MODEL`, `OMEGA_EVENT_DIR`), defauts W7 sinon (port 8787).
//! Prerequis : `ollama serve` + modele tire (ex. `ollama pull qwen2.5-coder:7b`).
//!
//! Contexte du modele : le candidat local declare 32768 tokens, ce qui exige
//! `OLLAMA_NUM_CTX=32768` côté daemon (le defaut d'Ollama, 4096, tronquerait
//! tout contexte assemblé au-dela). Sans cette variable, reduisez `max_tokens`
//! et `REQUIRED_CONTEXT` en conséquence.

use std::net::SocketAddr;

use omega_server::{build_app_state_with, create_router_with_ui, ServerConfig};

#[tokio::main]
async fn main() {
    let cfg = ServerConfig::from_env();
    let state = match build_app_state_with(&cfg) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("omega-server: assemblage impossible : {err}");
            std::process::exit(1);
        }
    };
    let ui = cfg.ui_dir.clone();
    match &ui {
        Some(dir) if dir.is_dir() => println!("omega-server: UI servie depuis {}", dir.display()),
        Some(dir) => eprintln!(
            "omega-server: UI absente ({} introuvable) — API seule. \
             Buildez-la : cd omega/ui && npm run build",
            dir.display()
        ),
        None => println!("omega-server: service statique de l'UI desactive (OMEGA_UI_DIR vide)"),
    }
    let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("omega-server: ecoute sur {addr} impossible : {err}");
            std::process::exit(1);
        }
    };
    println!("omega-server: ecoute sur http://{addr}");
    if let Err(err) = axum::serve(listener, create_router_with_ui(state, ui)).await {
        eprintln!("omega-server: erreur de service : {err}");
        std::process::exit(1);
    }
}
