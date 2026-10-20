//! `tool_loop` — **boucle d'outils** (P1-1, plan §6 ; ADR-0006).
//!
//! Détection du tool-call → exécution via le port [`ToolSurface`] → observation
//! → multi-tours. Le domaine ne connaît que les traits : l'inference passe par
//! le port `Provider` (`omega-ports`), les outils concrets (jail filesystem,
//! ...) sont derrière [`ToolSurface`].
//!
//! ADR-0006 (une seule décision, deux formes) :
//! - **A — forme native** : `[{"tool_calls": [{"function": {...}}]}]`
//!   ([`detect_tool_calls`]) — détectée ici ; le branchement wire vit dans la
//!   couche providers/protocol (bump optionnel du schéma, hors noyau) ;
//! - **C — repli ReAct** : le modèle produit un JSON texte canonique
//!   `{"tool": "...", "arguments": {...}}` ([[`detect_react_call`] — parseur
//!   unique côté système). C'est la forme utilisée par la boucle tant que le
//!   wire ne transporte pas `tool_calls`.
//!
//! Invariants (§7-A2) : budget monotone — un budget `N` borne stricement le
//! nombre d'exécutions d'outils, quelle que soit la suite des appels du
//! modèle ; outil inconnu ou échec = observation structurée (`ok=false`),
//! jamais de panic ni d'arrêt de la boucle.

use async_trait::async_trait;
use omega_ports::Provider;
use omega_types::Message;
use omega_types::{ChatRequest, OmegaResult, Role};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{Event, Operation, RunContext};

/// Origine du contenu d'un tool-call détecté (traçabilité ADR-0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCallOrigin {
    /// Champ `tool_calls` natif du wire.
    Native,
    /// Repli ReAct : JSON texte canonique dans le contenu assistant.
    ReAct,
}

impl ToolCallOrigin {
    pub fn as_str(self) -> &'static str {
        match self {
            ToolCallOrigin::Native => "native",
            ToolCallOrigin::ReAct => "react",
        }
    }
}

/// Appel d'outil détecté. `call_id` : identifiant fourni par le wire (forme
/// native) ou attribué par la boucle (`call-N` par numéro de round, forme ReAct).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub call_id: String,
    pub name: String,
    pub arguments: Value,
    pub origin: ToolCallOrigin,
}

/// Observation renvoyée au modèle. L'échec de l'outil est `ok=false` avec un
/// contenu structuré `{"error": ...}` — l'exécution est tracée comme les autres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolObservation {
    pub call_id: String,
    pub ok: bool,
    pub content: Value,
}

impl ToolObservation {
    pub fn ok(call_id: impl Into<String>, content: Value) -> Self {
        Self {
            call_id: call_id.into(),
            ok: true,
            content,
        }
    }

    pub fn error(call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            ok: false,
            content: json!({ "error": message.into() }),
        }
    }
}

/// Port de la surface d'outils exécutable (P1-1). Les outils concrets
/// (`omega-tools`, MCP...) s'implémentent derrière ce trait.
///
/// Contrat : `execute` renvoie **toujours** une observation structurée — un
/// outil inconnu ou en échec donne `ok=false`, jamais de panic.
#[async_trait]
pub trait ToolSurface: Send + Sync {
    async fn execute(&self, call: &ToolCall) -> OmegaResult<ToolObservation>;
}

/// Extrait le premier objet JSON plausible d'un contenu texte (trim, clôtures
/// de code ```..., ou fenêtre du premier `{` au dernier `}`). `None` si rien
/// de lisible : l'entrée illisible n'est jamais un crash.
fn extract_json_object(text: &str) -> Option<Value> {
    let mut t = text.trim();
    if t.starts_with("```") {
        // Clôture de code : retire la ligne de garde et la clôture finale.
        t = t.split_once('\n').map(|(_, rest)| rest).unwrap_or(t);
        if let Some(end) = t.rfind("```") {
            t = t[..end].trim();
        }
    }
    if let Ok(v) = serde_json::from_str::<Value>(t) {
        return Some(v);
    }
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end <= start {
        return None;
    }
    serde_json::from_str::<Value>(&t[start..=end]).ok()
}

/// Détecte un tool-call en **repli ReAct** (ADR-0006-C) : contenu assistant
/// contenant `{"tool": "...", "arguments": {...}}`. `arguments` absent
/// = objet vide ; contenu non conforme = `None`.
pub fn detect_react_call(text: &str, round: usize) -> Option<ToolCall> {
    match extract_json_object(text) {
        Some(Value::Object(map)) => {
            let name = map.get("tool")?.as_str()?.to_string();
            if name.is_empty() {
                return None;
            }
            let arguments = match map.get("arguments") {
                Some(v @ Value::Object(_)) => v.clone(),
                None => json!({}),
                Some(_) => return None,
            };
            let call_id = map
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("call-{}", round));
            Some(ToolCall {
                call_id,
                name,
                arguments,
                origin: ToolCallOrigin::ReAct,
            })
        }
        _ => None,
    }
}

/// Détecte les tool-calls en **forme native** (ADR-0006-A) : un message
/// (objet JSON) portant `tool_calls: [{id?, function: {name, arguments}}]`,
/// `arguments` étant une **chaîne JSON** (convention Ollama/OpenAI) ou un
/// objet. Entrée illisible = vecteur vide, jamais de panic.
pub fn detect_tool_calls(native: &Value) -> Vec<ToolCall> {
    let calls = match native.get("tool_calls") {
        Some(Value::Array(calls)) => calls,
        _ => return Vec::new(),
    };
    calls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let function = c.get("function")?;
            let name = function.get("name")?.as_str()?.to_string();
            if name.is_empty() {
                return None;
            }
            let arguments = match function.get("arguments") {
                Some(Value::String(s)) => serde_json::from_str::<Value>(s).unwrap_or(json!({})),
                Some(v @ Value::Object(_)) => v.clone(),
                _ => json!({}),
            };
            let call_id = c
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("call-{}", i + 1));
            Some(ToolCall {
                call_id,
                name,
                arguments,
                origin: ToolCallOrigin::Native,
            })
        })
        .collect()
}

/// Operation de boucle d'outils : infère via `Provider`, détecte les
/// tool-calls (fusion ReAct ; forme native dès que le wire la transporte),
/// exécute via [`ToolSurface`], réinjecte les observations, multi-tours tant
/// que le modèle appelle et que le budget n'est pas épuisé.
///
/// Terminaison : chaque tour se termine (pas d'appel), exécute (borne par le
/// budget) ou s'arrête proprement (budget épuisé) — `rounds <= budget + 1`.
pub struct ToolLoopOperation {
    pub provider: std::sync::Arc<dyn Provider>,
    pub model: omega_types::ModelId,
    pub surface: std::sync::Arc<dyn ToolSurface>,
    /// Messages de conversation d'entrée (état initial de la boucle).
    pub messages: Vec<Message>,
    /// Budget d'exécutions d'outils (§7-A2 : monotone, strict).
    pub budget: usize,
    /// Identifiant du run, propagé dans chaque événement d'outil.
    pub run_id: String,
}

#[async_trait]
impl Operation for ToolLoopOperation {
    fn name(&self) -> &str {
        "tool_loop"
    }

    async fn run(&self, ctx: &mut RunContext<'_>) -> OmegaResult<()> {
        let mut messages = self.messages.clone();
        let mut executed: usize = 0;
        let mut rounds: usize = 0;

        loop {
            // Annulation cooperative : vérifiée à chaque tour.
            ctx.check_cancellation()?;
            rounds += 1;

            let response = self
                .provider
                .complete(ChatRequest {
                    model: self.model.clone(),
                    messages: messages.clone(),
                })
                .await?;

            let Some(call) = detect_react_call(&response.message.content, rounds) else {
                // Pas de tool-call : la boucle se termine.
                ctx.publish(Event::new(
                    "tool.loop.completed",
                    json!({
                        "rounds": rounds,
                        "tool_executions": executed,
                    }),
                ));
                ctx.state.0 = json!({
                    "content": response.message.content,
                    "rounds": rounds,
                    "tool_executions": executed,
                    "status": "done",
                });
                return Ok(());
            };

            if executed >= self.budget {
                // Budget épuisé : arrêt propre et tracé (§7-A2, monotone).
                ctx.publish(Event::new(
                    "tool.budget.exhausted",
                    json!({
                        "budget": self.budget,
                        "requested": call.name,
                        "run_id": self.run_id,
                    }),
                ));
                ctx.state.0 = json!({
                    "content": response.message.content,
                    "rounds": rounds,
                    "tool_executions": executed,
                    "status": "budget-exhausted",
                });
                return Ok(());
            }

            ctx.publish(Event::new(
                "tool.call.started",
                json!({
                    "run_id": self.run_id,
                    "call_id": call.call_id,
                    "tool": call.name,
                    "origin": call.origin.as_str(),
                }),
            ));

            let observation = self.surface.execute(&call).await?;
            executed += 1;

            ctx.publish(Event::new(
                "tool.call.completed",
                json!({
                    "run_id": self.run_id,
                    "call_id": observation.call_id,
                    "tool": call.name,
                    "origin": call.origin.as_str(),
                    "ok": observation.ok,
                    "content": observation.content,
                }),
            ));

            messages.push(Message::new(Role::Assistant, response.message.content));
            messages.push(Message::new(
                Role::Tool,
                json!({
                    "call_id": observation.call_id,
                    "ok": observation.ok,
                    "content": observation.content,
                })
                .to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- Détecteurs ----------------------------------------------------------

    #[test]
    fn react_detects_canonical_json() {
        let call = detect_react_call("{\"tool\": \"read_file\", \"arguments\": {\"path\": \"a\"}}", 1)
            .expect("forme canonique doit etre detectee");
        assert_eq!(call.name, "read_file");
        assert_eq!(call.arguments["path"], "a");
        assert_eq!(call.origin, ToolCallOrigin::ReAct);
        assert_eq!(call.call_id, "call-1");
    }

    #[test]
    fn react_detects_fenced_text() {
        let call = detect_react_call(
            "Je vais lire le fichier.\n```json\n{\"tool\": \"ls\", \"arguments\": {\"path\": \".\"}}\n```",
            2,
        )
        .expect("json enclos doit etre detecte");
        assert_eq!(call.name, "ls");
        assert_eq!(call.arguments["path"], ".");
        assert_eq!(call.call_id, "call-2");
    }

    #[test]
    fn arguments_absent_equivalent_objet_vide() {
        let call = detect_react_call("{\"tool\":\"get_workspace\"}", 3).expect("detecte");
        assert_eq!(call.arguments, json!({}));
        assert_eq!(call.call_id, "call-3");
    }

    #[test]
    fn react_illisible_ou_non_conforme_retourne_none() {
        // Garbage, JSON non-objet, objet sans "tool", "tool" non-string,
        // arguments non-objet : aucun case ne panique, tous = None.
        assert!(detect_react_call("{pas du json", 1).is_none());
        assert!(detect_react_call("[1,2,3]", 1).is_none());
        assert!(detect_react_call("{\"arguments\": {}}", 1).is_none());
        assert!(detect_react_call("{\"tool\": 4}", 1).is_none());
        assert!(detect_react_call("{\"tool\": \"x\", \"arguments\": [1]}", 1).is_none());
        assert!(detect_react_call("", 1).is_none());
    }

    #[test]
    fn native_detecte_la_forme_wire_et_l_arg_string() {
        let msg = json!({
            "tool_calls": [
                {"id": "tc-9", "function": {"name": "write_file", "arguments": "{\"path\":\"b\"}"}}
            ]
        });
        let calls = detect_tool_calls(&msg);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "write_file");
        assert_eq!(calls[0].arguments["path"], "b");
        assert_eq!(calls[0].origin, ToolCallOrigin::Native);
        // id wire prioritaire sur l'attribution locale.
        assert_eq!(calls[0].call_id, "tc-9");
    }

    #[test]
    fn native_illisible_est_inerte() {
        // Garbage : aucun panic, vecteur vide.
        assert!(detect_tool_calls(&json!({"tool_calls": "oops"})).is_empty());
        assert!(detect_tool_calls(&json!({"tool_calls": [{"function": {}}]})).is_empty());
        assert!(detect_tool_calls(&json!({"tool_calls": [13]})).is_empty());
        assert!(detect_tool_calls(&json!(null)).is_empty());
    }
}
