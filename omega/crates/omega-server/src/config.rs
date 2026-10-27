//! Configuration du serveur (binaire de bord).
//!
//! Le port d'ecoute est configurable (defaut **8787**, cf. proxy Vite dans
//! `omega/ui/vite.config.ts`) : variable d'environnement `OMEGA_PORT` puis
//! `PORT`, sinon defaut. Meme logique pour Ollama (`OLLAMA_URL`,
//! `OLLAMA_MODEL`) et le repertoire des journaux (`OMEGA_EVENT_DIR`).

use std::path::PathBuf;

/// Port d'ecoute par defaut (convention W7, reprise par le proxy Vite).
pub const DEFAULT_PORT: u16 = 8787;

/// URL de base d'Ollama par defaut (self-hosted, cout nul).
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Modele local par defaut (doit etre tire via `ollama pull`).
pub const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5-coder:7b";

/// Budget de contexte par defaut (tokens) transmis au `ContextContributor`.
pub const DEFAULT_MAX_TOKENS: usize = 8000;

/// Contexte d'inference exige par le selecteur (tokens).
pub const REQUIRED_CONTEXT: u32 = 4000;

/// Budget d'executions d'outils par run (P1-1) : borne stricte, un tour de
/// boucle consomme une unite.
pub const DEFAULT_TOOL_BUDGET: usize = 8;

/// Surface d'outils utilisee par la boucle d'outils (P1-1).
///
/// La surface **MCP** (`mcp:rea`) reste branchee au registre comme capacite
/// `tool-surface` (cf. `state.rs`). Elle lance `npx rea-agents --mcp` a chaque
/// appel : impossible en CI, et sans rapport avec le cas d'usage local
/// (lister/lire un fichier du workspace). D'ou une **seconde** surface, locale
/// et explicite, choisie par configuration — pas par le registre.
///
/// Ce n'est pas une exception a « everything is a plugin » : la surface reste
/// une implementation du port `ToolSurface` (`omega-ports`), resolue par le
/// meme chemin que les autres capacites. Seule la *selection* est explicite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSurfaceKind {
    /// Surface filesystem locale, ancree sur une racine (jail `omega-tools`).
    LocalFs {
        /// Racine du workspace : tout chemin resolu doit en rester sous elle.
        root: PathBuf,
    },
    /// Surface MCP (serveur externe) — comportement historique.
    Mcp,
}

impl ToolSurfaceKind {
    /// Racine du workspace, si la surface est locale.
    pub fn root(&self) -> Option<&std::path::Path> {
        match self {
            ToolSurfaceKind::LocalFs { root } => Some(root),
            ToolSurfaceKind::Mcp => None,
        }
    }
}

/// Configuration du serveur.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port d'ecoute HTTP.
    pub port: u16,
    /// URL de base d'Ollama (sans slash final).
    pub ollama_base_url: String,
    /// Modele Ollama utilise pour l'inference et le candidat de selection.
    pub ollama_model: String,
    /// Repertoire des journaux (`FileEventStore`, un fichier par run).
    pub event_dir: PathBuf,
    /// Budget de contexte (tokens) du moteur d'assemblage.
    pub max_tokens: usize,
    /// Repertoire de l'UI buildee (`omega/ui/dist`) servi sur `/`.
    ///
    /// `None` → API seule. Defaut : chemin de compilation `../../ui/dist`
    /// (relatif a la crate), surchargeable par `OMEGA_UI_DIR`.
    pub ui_dir: Option<PathBuf>,
    /// Surface d'outils de la boucle (P1-1). `None` → pas de boucle d'outils
    /// (comportement d'avant B1 : un seul appel modele).
    ///
    /// Surchargeable par `OMEGA_TOOL_SURFACE=fs|mcp|off` ; la racine du
    /// workspace FS vient de `OMEGA_WORKSPACE_ROOT` (defaut : repertoire
    /// courant du processus).
    pub tool_surface: Option<ToolSurfaceKind>,
    /// Budget d'executions d'outils par run (P1-1, borne stricte).
    pub tool_budget: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            ollama_base_url: DEFAULT_OLLAMA_URL.to_string(),
            ollama_model: DEFAULT_OLLAMA_MODEL.to_string(),
            event_dir: std::env::temp_dir().join("omega-server"),
            max_tokens: DEFAULT_MAX_TOKENS,
            ui_dir: Some(default_ui_dir()),
            tool_surface: Some(ToolSurfaceKind::LocalFs {
                root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            }),
            tool_budget: DEFAULT_TOOL_BUDGET,
        }
    }
}

/// Repertoire par defaut de l'UI buildee, resolu a la compilation
/// (`<crate>/../../ui/dist` == `omega/ui/dist`).
pub fn default_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui/dist")
}

impl ServerConfig {
    /// Construit la config depuis l'environnement, avec repli sur les defauts.
    ///
    /// `OMEGA_PORT` (prioritaire) puis `PORT` ; valeur non numerique = ignoree.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Some(port) = std::env::var("OMEGA_PORT")
            .ok()
            .or_else(|| std::env::var("PORT").ok())
            .and_then(|v| v.parse::<u16>().ok())
        {
            cfg.port = port;
        }
        if let Ok(url) = std::env::var("OLLAMA_URL") {
            if !url.trim().is_empty() {
                cfg.ollama_base_url = url.trim_end_matches('/').to_string();
            }
        }
        if let Ok(model) = std::env::var("OLLAMA_MODEL") {
            if !model.trim().is_empty() {
                cfg.ollama_model = model;
            }
        }
        if let Ok(dir) = std::env::var("OMEGA_EVENT_DIR") {
            if !dir.trim().is_empty() {
                cfg.event_dir = PathBuf::from(dir);
            }
        }
        // `OMEGA_UI_DIR=""` (vide) desactive le service statique de l'UI.
        if let Ok(dir) = std::env::var("OMEGA_UI_DIR") {
            cfg.ui_dir = if dir.trim().is_empty() {
                None
            } else {
                Some(PathBuf::from(dir))
            };
        }
        // Surface d'outils : `OMEGA_TOOL_SURFACE=fs|mcp|off` (defaut `fs`),
        // racine du workspace via `OMEGA_WORKSPACE_ROOT`.
        let workspace_root = std::env::var("OMEGA_WORKSPACE_ROOT")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        cfg.tool_surface = match std::env::var("OMEGA_TOOL_SURFACE")
            .ok()
            .as_deref()
            .map(str::trim)
        {
            Some("off") | Some("none") => None,
            Some("mcp") => Some(ToolSurfaceKind::Mcp),
            _ => Some(ToolSurfaceKind::LocalFs {
                root: workspace_root,
            }),
        };
        if let Some(b) = std::env::var("OMEGA_TOOL_BUDGET")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
        {
            cfg.tool_budget = b;
        }
        if let Some(n) = std::env::var("OMEGA_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
        {
            cfg.max_tokens = n;
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_w7_convention() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.port, 8787);
        assert_eq!(cfg.ollama_base_url, "http://localhost:11434");
        assert_eq!(cfg.ollama_model, "qwen2.5-coder:7b");
        assert_eq!(cfg.max_tokens, 8000);
        assert!(cfg.event_dir.to_string_lossy().contains("omega-server"));
    }

    #[test]
    fn from_env_falls_back_to_defaults() {
        // Sans toucher a l'environnement partage : une valeur non numerique
        // positionnee via une config partielle ne doit jamais paniquer.
        // On teste uniquement que le defaut reste coherent.
        let cfg = ServerConfig::default();
        assert_eq!(ServerConfig::from_env().max_tokens, cfg.max_tokens);
    }
}
