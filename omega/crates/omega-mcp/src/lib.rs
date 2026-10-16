//! `omega-mcp` — **pont MCP** (Model Context Protocol) du noyau Omega.
//!
//! Objectif (pilier « everything is a plugin ») : brancher n'importe quel serveur
//! MCP comme **fournisseur d'outils** du registre, sans toucher au noyau. Le
//! premier client visé est **REA** (`morluto/rea`, `npx -y rea-agents@latest --mcp`),
//! l'outil de reverse-engineering de notre workstream W5.
//!
//! ## Ce que ce crate fournit
//!
//! - [`McpClient`] : client **JSON-RPC 2.0 sur stdio**, avec les trois méthodes
//!   normalisées `initialize`, `tools/list`, `tools/call` ;
//! - [`McpToolSurface`] : surface d'outils d'un serveur, résolvable depuis le
//!   registre (`Capability::ToolSurface`) — c'est la brique « branchée » ;
//! - [`McpToolExecutor`] : adaptateur d'un outil MCP en [`ToolExecutor`]
//!   (un outil = un port), enregistrable en `Capability::Tool` ;
//! - [`McpContributor`] : contributes le tout au registre.
//!
//! ## Choix assumés (Phase 1)
//!
//! - **Cycle de vie court** : la surface lance un serveur par opération
//!   (`spawn -> initialize -> list/call -> kill`). Sûr et déterministe ; un mode
//!   « serveur persistant » viendra quand un profil de latence l'exigera.
//! - **Aucun secret dans les journaux** : le client ne journalise jamais les
//!   arguments ni les variables d'environnement.
//! - **Erreurs mappées** : une erreur JSON-RPC devient `OmegaError::PluginFailed`,
//!   un `isError: true` dans un résultat devient `OmegaError::PluginFailed` aussi
//!   (un outil qui échoue n'est pas un succès).
//!
//! Protocole : `docs/omega/reverse-eng/dissections/rea.md` et
//! `https://modelcontextprotocol.io` (révision `2025-06-18`).

use std::collections::BTreeMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use omega_plugin_api::{Contributor, InstallContext};
use omega_registry::{Manifest, RegistryBuilder};
use omega_types::{Capability, OmegaError, OmegaResult, ToolSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// Révision MCP annoncée par le client.
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

/// Délai maximal d'une opération unitaire (spawn + une requête).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Descripteur d'un serveur MCP lançable en stdio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerSpec {
    /// Exécutable (ex. `npx`, `/usr/bin/node`).
    pub command: String,
    /// Arguments (ex. `["-y", "rea-agents@latest", "--mcp"]`).
    pub args: Vec<String>,
    /// Variables d'environnement additionnelles (jamais journalisées).
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Répertoire de travail (optionnel).
    #[serde(default)]
    pub cwd: Option<String>,
}

impl McpServerSpec {
    pub fn new(
        command: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Spécification du serveur MCP de **REA** (`npx -y rea-agents@latest --mcp`).
    ///
    /// `REA_PWNTOOLS_PYTHON` doit pointer vers un interpréteur fourni par l'appelant
    /// (pwntools/pyelftools/Unicorn) : REA n'installe jamais de paquets.
    pub fn rea() -> Self {
        let mut spec = Self::new("npx", ["-y", "rea-agents@latest", "--mcp"]);
        if let Ok(python) = std::env::var("REA_PWNTOOLS_PYTHON") {
            spec = spec.with_env("REA_PWNTOOLS_PYTHON", python);
        }
        spec
    }

    /// Identifiant stable, dérivé de la ligne de commande.
    pub fn id(&self) -> String {
        let mut s = self.command.clone();
        for a in &self.args {
            s.push(' ');
            s.push_str(a);
        }
        s
    }
}

/// Un outil exposé par un serveur MCP (`tools/list`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Value,
}

impl McpToolDescriptor {
    /// Convertit le descripteur MCP en [`ToolSpec`] du noyau.
    pub fn to_tool_spec(&self, prefix: &str) -> ToolSpec {
        ToolSpec {
            name: format!("{prefix}{}", self.name),
            description: self.description.clone(),
            schema: self.input_schema.clone(),
        }
    }
}

/// Erreur JSON-RPC renvoyée par un serveur MCP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Réponse à `initialize`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerInfo {
    #[serde(rename = "protocolVersion", default)]
    pub protocol_version: String,
    #[serde(rename = "serverInfo", default)]
    pub server_info: Value,
    #[serde(default)]
    pub capabilities: Value,
}

/// Client MCP (JSON-RPC 2.0 sur stdio : une requête par ligne).
pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    timeout: Duration,
    spec: McpServerSpec,
    server: Option<McpServerInfo>,
}

impl McpClient {
    /// Lance le serveur et ouvre le dialogue stdio.
    pub async fn spawn(spec: &McpServerSpec) -> OmegaResult<Self> {
        let mut cmd = Command::new(&spec.command);
        cmd.args(&spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        if let Some(cwd) = &spec.cwd {
            cmd.current_dir(cwd);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| OmegaError::Io(format!("spawn MCP '{}' : {e}", spec.command)))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| OmegaError::Io("MCP : stdin indisponible".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| OmegaError::Io("MCP : stdout indisponible".into()))?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            timeout: DEFAULT_TIMEOUT,
            spec: spec.clone(),
            server: None,
        })
    }

    pub fn spec(&self) -> &McpServerSpec {
        &self.spec
    }

    pub fn server_info(&self) -> Option<&McpServerInfo> {
        self.server.as_ref()
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// `initialize` + notification `notifications/initialized`.
    pub async fn initialize(&mut self) -> OmegaResult<&McpServerInfo> {
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": MCP_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": { "name": "omega", "version": env!("CARGO_PKG_VERSION") }
                }),
            )
            .await?;
        let info: McpServerInfo = serde_json::from_value(result).map_err(|e| {
            OmegaError::PluginFailed("mcp:initialize".into(), format!("réponse illisible : {e}"))
        })?;
        self.notify("notifications/initialized", json!({})).await?;
        self.server = Some(info);
        Ok(self.server.as_ref().expect("server posé juste avant"))
    }

    /// `tools/list`.
    pub async fn list_tools(&mut self) -> OmegaResult<Vec<McpToolDescriptor>> {
        let result = self.request("tools/list", json!({})).await?;
        #[derive(Deserialize)]
        struct Listed {
            #[serde(default)]
            tools: Vec<McpToolDescriptor>,
        }
        let parsed: Listed = serde_json::from_value(result).map_err(|e| {
            OmegaError::PluginFailed("mcp:tools/list".into(), format!("réponse illisible : {e}"))
        })?;
        Ok(parsed.tools)
    }

    /// `tools/call` — renvoie le **résultat brut** MCP.
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> OmegaResult<Value> {
        let result = self
            .request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await?;
        if result.get("isError").and_then(Value::as_bool) == Some(true) {
            let text = first_text(&result);
            return Err(OmegaError::PluginFailed(
                format!("mcp:{}", self.spec.id()),
                format!("l'outil '{name}' a signalé un échec : {text}"),
            ));
        }
        Ok(result)
    }

    async fn notify(&mut self, method: &str, params: Value) -> OmegaResult<()> {
        let msg = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        let mut line = serde_json::to_string(&msg).map_err(|e| {
            OmegaError::PluginFailed("mcp:encode".into(), format!("encodage impossible : {e}"))
        })?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| OmegaError::Io(format!("écriture MCP : {e}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| OmegaError::Io(format!("flush MCP : {e}")))?;
        Ok(())
    }

    async fn request(&mut self, method: &str, params: Value) -> OmegaResult<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let mut line = serde_json::to_string(&msg).map_err(|e| {
            OmegaError::PluginFailed("mcp:encode".into(), format!("encodage impossible : {e}"))
        })?;
        line.push('\n');

        let timeout = self.timeout;
        let stdin = &mut self.stdin;
        let stdout = &mut self.stdout;
        let server_id = self.spec.id();

        let read = async {
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| OmegaError::Io(format!("écriture MCP : {e}")))?;
            stdin
                .flush()
                .await
                .map_err(|e| OmegaError::Io(format!("flush MCP : {e}")))?;
            loop {
                let mut buf = String::new();
                let n = stdout
                    .read_line(&mut buf)
                    .await
                    .map_err(|e| OmegaError::Io(format!("lecture MCP : {e}")))?;
                if n == 0 {
                    return Err(OmegaError::Io(
                        "MCP : le serveur a fermé stdout avant de répondre".into(),
                    ));
                }
                let msg: Value = match serde_json::from_str(buf.trim()) {
                    Ok(v) => v,
                    Err(_) => continue, // ligne non-JSON (bruit du serveur)
                };
                if msg.get("id").and_then(Value::as_u64) != Some(id) {
                    continue; // notification ou réponse à une autre requête
                }
                if let Some(err) = msg.get("error") {
                    let rpc: McpRpcError =
                        serde_json::from_value(err.clone()).unwrap_or(McpRpcError {
                            code: -1,
                            message: err.to_string(),
                            data: None,
                        });
                    return Err(OmegaError::PluginFailed(
                        format!("mcp:{server_id}"),
                        format!("{} (code {})", rpc.message, rpc.code),
                    ));
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
        };

        match tokio::time::timeout(timeout, read).await {
            Ok(res) => res,
            Err(_) => Err(OmegaError::Io(format!(
                "MCP : délai de {}s dépassé (méthode '{method}')",
                timeout.as_secs()
            ))),
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // `kill_on_drop` couvre déjà le cas ; on tente un arrêt propre sans bloquer.
        let _ = self.child.start_kill();
    }
}

/// Instantané embarqué du catalogue REA (`rea doctor`/`capabilities`, capté le
/// 2026-10-17 pour `rea 6.5.0`). Données versionnées dans `data/` du crate.
///
/// Sert la vue « REA » du cockpit **sans lancer `npx`** (déterministe et rapide) ;
/// les appels live passent par [`McpToolSurface::rea`].
pub fn rea_snapshot() -> Value {
    serde_json::from_str(include_str!("../data/rea-snapshot.json"))
        .expect("instantané REA embarqué valide (vérifié par test)")
}

/// Capacités brutes de REA (providers + opérations disponibles/indisponibles).
pub fn rea_capabilities() -> Value {
    serde_json::from_str(include_str!("../data/rea-capabilities.json"))
        .expect("capacités REA embarquées valides (vérifié par test)")
}

fn first_text(result: &Value) -> String {
    result
        .get("content")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find_map(|c| c.get("text").and_then(Value::as_str).map(str::to_string))
        })
        .unwrap_or_else(|| "<sans texte>".to_string())
}

/// Surface d'outils d'un serveur MCP, **branchable au registre**
/// (`Capability::ToolSurface`).
///
/// Elle ne détient aucun processus : chaque opération lance un serveur éphémère
/// (cf. « Choix assumés » dans la doc du crate).
#[derive(Debug, Clone)]
pub struct McpToolSurface {
    spec: McpServerSpec,
    timeout: Duration,
    prefix: String,
}

impl McpToolSurface {
    pub fn new(spec: McpServerSpec) -> Self {
        Self {
            spec,
            timeout: DEFAULT_TIMEOUT,
            prefix: String::new(),
        }
    }

    /// Surface pour REA, outils exposés sous le préfixe `rea::`.
    pub fn rea() -> Self {
        Self::new(McpServerSpec::rea()).with_prefix("rea::")
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn spec(&self) -> &McpServerSpec {
        &self.spec
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn name(&self) -> String {
        if self.prefix.is_empty() {
            format!("mcp:{}", self.spec.command)
        } else {
            format!("mcp:{}", self.prefix.trim_end_matches("::"))
        }
    }

    /// Liste les outils (serveur éphémère).
    pub async fn list_tools(&self) -> OmegaResult<Vec<McpToolDescriptor>> {
        let mut client = McpClient::spawn(&self.spec).await?;
        client.set_timeout(self.timeout);
        client.initialize().await?;
        client.list_tools().await
    }

    /// Appelle un outil **par son nom nu** (ex. `analyze`).
    pub async fn call_tool(&self, tool: &str, arguments: Value) -> OmegaResult<Value> {
        let mut client = McpClient::spawn(&self.spec).await?;
        client.set_timeout(self.timeout);
        client.initialize().await?;
        client.call_tool(tool, arguments).await
    }

    /// Enveloppe chaque descripteur en [`McpToolExecutor`] (un outil = un port).
    pub fn executors(
        self: &Arc<Self>,
        tools: &[McpToolDescriptor],
    ) -> Vec<Arc<dyn omega_ports::ToolExecutor>> {
        tools
            .iter()
            .map(|t| {
                Arc::new(McpToolExecutor {
                    surface: Arc::clone(self),
                    descriptor: t.clone(),
                }) as Arc<dyn omega_ports::ToolExecutor>
            })
            .collect()
    }
}

/// Un outil MCP exposé comme [`ToolExecutor`] (« un outil = un port »).
pub struct McpToolExecutor {
    surface: Arc<McpToolSurface>,
    descriptor: McpToolDescriptor,
}

impl McpToolExecutor {
    pub fn new(surface: Arc<McpToolSurface>, descriptor: McpToolDescriptor) -> Self {
        Self {
            surface,
            descriptor,
        }
    }

    pub fn descriptor(&self) -> &McpToolDescriptor {
        &self.descriptor
    }
}

#[async_trait]
impl omega_ports::ToolExecutor for McpToolExecutor {
    fn name(&self) -> &str {
        &self.descriptor.name
    }

    fn spec(&self) -> ToolSpec {
        self.descriptor.to_tool_spec(self.surface.prefix())
    }

    async fn execute(&self, args: Value) -> OmegaResult<Value> {
        self.surface
            .call_tool(&self.descriptor.name, args)
            .await
            .map_err(|e| OmegaError::PluginFailed(self.name().to_string(), e.to_string()))
    }
}

/// Contributeur : enregistre la surface MCP en `Capability::ToolSurface`.
///
/// Câblage (manifeste → registre) identique aux autres adaptateurs : aucune
/// connaissance du noyau n'est requise.
pub struct McpContributor {
    spec: McpServerSpec,
    prefix: String,
}

impl McpContributor {
    pub fn new(spec: McpServerSpec) -> Self {
        Self {
            spec,
            prefix: String::new(),
        }
    }

    pub fn rea() -> Self {
        Self::new(McpServerSpec::rea()).with_prefix("rea::")
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

impl Contributor for McpContributor {
    fn name(&self) -> &str {
        "mcp-contributor"
    }

    fn contribute(&self, builder: &mut RegistryBuilder, _ctx: &InstallContext) -> OmegaResult<()> {
        let surface =
            Arc::new(McpToolSurface::new(self.spec.clone()).with_prefix(self.prefix.clone()));
        let manifest =
            Manifest::new(surface.name(), Capability::ToolSurface, "1").with_priority(10);
        builder.register_typed::<McpToolSurface>(manifest, surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributor_registers_tool_surface() {
        let contributor = McpContributor::rea();
        let mut builder = RegistryBuilder::new();
        contributor
            .contribute(&mut builder, &InstallContext::new())
            .unwrap();
        let registry = builder.build();
        let ids = registry.active_ids(Capability::ToolSurface);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "mcp:rea");
        let surface = registry
            .resolve::<McpToolSurface>(Capability::ToolSurface)
            .unwrap();
        assert_eq!(surface.prefix(), "rea::");
    }

    #[test]
    fn rea_spec_targets_mcp_stdio() {
        let spec = McpServerSpec::rea();
        assert_eq!(spec.command, "npx");
        assert_eq!(spec.args, vec!["-y", "rea-agents@latest", "--mcp"]);
    }

    #[test]
    fn descriptor_maps_to_prefixed_tool_spec() {
        let d = McpToolDescriptor {
            name: "analyze".into(),
            description: "Analyse une app.".into(),
            input_schema: json!({"type": "object"}),
        };
        let spec = d.to_tool_spec("rea::");
        assert_eq!(spec.name, "rea::analyze");
        assert_eq!(spec.description, "Analyse une app.");
    }

    #[test]
    fn first_text_extracts_content_text() {
        let r = json!({"content": [{"type": "text", "text": "hello"}], "isError": true});
        assert_eq!(first_text(&r), "hello");
        assert_eq!(first_text(&json!({})), "<sans texte>");
    }

    #[test]
    fn embedded_rea_snapshot_is_valid() {
        let s = rea_snapshot();
        assert_eq!(s["rea_version"], "6.5.0");
        assert_eq!(s["catalog"]["mcp_tools"], 207);
        assert_eq!(s["catalog"]["cli_commands"], 153);
        assert!(s["providers"].as_array().unwrap().len() >= 4);
    }

    #[test]
    fn embedded_rea_capabilities_is_valid() {
        let c = rea_capabilities();
        assert!(c["capabilities"].is_array());
        assert!(!c["capabilities"].as_array().unwrap().is_empty());
    }
}
