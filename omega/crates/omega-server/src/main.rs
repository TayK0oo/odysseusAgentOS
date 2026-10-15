//! Binaire de bord Omega : sert l'API HTTP de l'UI Cockpit.
//!
//! Configuration par l'environnement (`OMEGA_PORT`/`PORT`, `OLLAMA_URL`,
//! `OLLAMA_MODEL`, `OMEGA_EVENT_DIR`), defauts W7 sinon (port 8787).
//! Prerequis : `ollama serve` + modele tire (ex. `ollama pull qwen2.5-coder:7b`).

use std::net::SocketAddr;

use omega_server::{build_app_state_with, create_router, ServerConfig};

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
    let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("omega-server: ecoute sur {addr} impossible : {err}");
            std::process::exit(1);
        }
    };
    println!("omega-server: ecoute sur http://{addr}");
    if let Err(err) = axum::serve(listener, create_router(state)).await {
        eprintln!("omega-server: erreur de service : {err}");
        std::process::exit(1);
    }
}
