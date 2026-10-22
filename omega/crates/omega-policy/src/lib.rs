//! `omega-policy` — portage Rust des politiques de sécurité et de budget.
//!
//! - [`BudgetEnforcer`] : port de `src/budget_enforcer.py` (tokens / itérations / tool calls / coût)
//! - [`PathRulePermissionPolicy`] : politique de permission par motifs de chemins
//! - [`CommandPrefixExecPolicy`] : politique d'exécution par préfixes de commande
#![allow(clippy::new_without_default)]

pub mod budget;
pub mod run_budget;
pub mod tiers;

use omega_ports::{ExecPolicy, PermissionPolicy};

use omega_types::{ExecDecision, PermissionEffect};
pub use run_budget::PolicyRunBudget;

// ═══════════════════════════════════════════════════════════════════════════
// Budget (port de src/budget_enforcer.py)
// ═══════════════════════════════════════════════════════════════════════════

/// Plafonds d'un projet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectBudget {
    pub max_tokens: u64,
    pub max_cost_usd: f64,
    pub max_iterations: u64,
    pub max_tool_calls: u64,
}

impl Default for ProjectBudget {
    fn default() -> Self {
        Self {
            max_tokens: 200_000,
            max_cost_usd: 5.0,
            max_iterations: 50,
            max_tool_calls: 500,
        }
    }
}

/// Consommation courante.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BudgetUsage {
    pub tokens_used: u64,
    pub cost_usd: f64,
    pub iterations: u64,
    pub tool_calls: u64,
}

/// Verdict de budget.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub ok: bool,
    pub reason: String,
    pub percent: f64,
}

impl Verdict {
    fn ok() -> Self {
        Self {
            ok: true,
            reason: String::new(),
            percent: 0.0,
        }
    }
    fn fail(reason: impl Into<String>, percent: f64) -> Self {
        Self {
            ok: false,
            reason: reason.into(),
            percent,
        }
    }
}

/// Forceur de budget — port de `BudgetEnforcer` (thread-safe via Mutex).
pub struct BudgetEnforcer {
    budget: ProjectBudget,
    run_id: String,
    usage: std::sync::Mutex<BudgetUsage>,
}

impl BudgetEnforcer {
    pub fn new(budget: ProjectBudget, run_id: impl Into<String>) -> Self {
        Self {
            budget,
            run_id: run_id.into(),
            usage: std::sync::Mutex::new(BudgetUsage::default()),
        }
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    pub fn usage(&self) -> BudgetUsage {
        *self.usage.lock().unwrap()
    }

    /// Port de `consume_tokens`.
    pub fn consume_tokens(&self, count: u64, cost_usd: f64) -> Verdict {
        let mut u = self.usage.lock().unwrap();
        u.tokens_used += count;
        u.cost_usd += cost_usd;
        if u.tokens_used > self.budget.max_tokens {
            return Verdict::fail(
                format!(
                    "Token budget exceeded: {} / {}",
                    u.tokens_used, self.budget.max_tokens
                ),
                pct(u.tokens_used as f64, self.budget.max_tokens as f64),
            );
        }
        if u.cost_usd > self.budget.max_cost_usd {
            return Verdict::fail(
                format!(
                    "Cost budget exceeded: ${:.4} / ${:.4}",
                    u.cost_usd, self.budget.max_cost_usd
                ),
                pct(u.cost_usd, self.budget.max_cost_usd),
            );
        }
        Verdict::ok()
    }

    /// Port de `consume_iteration`.
    pub fn consume_iteration(&self) -> Verdict {
        let mut u = self.usage.lock().unwrap();
        u.iterations += 1;
        if u.iterations > self.budget.max_iterations {
            return Verdict::fail(
                format!(
                    "Iteration budget exceeded: {} / {}",
                    u.iterations, self.budget.max_iterations
                ),
                pct(u.iterations as f64, self.budget.max_iterations as f64),
            );
        }
        Verdict::ok()
    }

    /// Port de `consume_tool_call`.
    pub fn consume_tool_call(&self) -> Verdict {
        let mut u = self.usage.lock().unwrap();
        u.tool_calls += 1;
        if u.tool_calls > self.budget.max_tool_calls {
            return Verdict::fail(
                format!(
                    "Tool call budget exceeded: {} / {}",
                    u.tool_calls, self.budget.max_tool_calls
                ),
                pct(u.tool_calls as f64, self.budget.max_tool_calls as f64),
            );
        }
        Verdict::ok()
    }

    /// Port de `is_budget_ok` : coût d'abord, puis tokens/iterations/tool_calls.
    pub fn is_budget_ok(&self) -> Verdict {
        let u = self.usage.lock().unwrap();
        if self.budget.max_cost_usd > 0.0 {
            let p = pct(u.cost_usd, self.budget.max_cost_usd);
            if p > 100.0 {
                return Verdict::fail(format!("Cost budget exceeded ({:.1}%)", p), p);
            }
        }
        let checks = [
            (
                u.tokens_used as f64,
                self.budget.max_tokens as f64,
                "tokens",
            ),
            (
                u.iterations as f64,
                self.budget.max_iterations as f64,
                "iterations",
            ),
            (
                u.tool_calls as f64,
                self.budget.max_tool_calls as f64,
                "tool_calls",
            ),
        ];
        let mut max_pct: f64 = 0.0;
        for (used, limit, label) in checks {
            if limit <= 0.0 {
                continue;
            }
            let p = pct(used, limit);
            max_pct = max_pct.max(p);
            if p > 100.0 {
                return Verdict::fail(format!("{} budget exceeded ({:.1}%)", label, p), p);
            }
        }
        Verdict {
            ok: true,
            reason: String::new(),
            percent: max_pct,
        }
    }

    /// Port de `alert_threshold_reached` : alerte à 80 %.
    pub fn alert_threshold_reached(&self) -> bool {
        self.is_budget_ok().percent >= 80.0
    }
}

fn pct(used: f64, limit: f64) -> f64 {
    if limit <= 0.0 {
        0.0
    } else {
        (used / limit) * 100.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PermissionPolicy — motifs de chemins sensibles (équivalent OPA "deny rules")
// ═══════════════════════════════════════════════════════════════════════════

/// Politique de permission par motifs (port de la logique OPA + gsd-secret-read-guard).
pub struct PathRulePermissionPolicy {
    name: String,
    deny_patterns: Vec<String>,
    ask_patterns: Vec<String>,
}

impl PathRulePermissionPolicy {
    pub fn new(deny_patterns: &[&str], ask_patterns: &[&str]) -> Self {
        Self {
            name: "path-rule".into(),
            deny_patterns: deny_patterns.iter().map(|s| s.to_string()).collect(),
            ask_patterns: ask_patterns.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Deny-list par défaut (fichiers sensibles — même liste que gsd-secret-read-guard).
    pub fn sensitive_defaults() -> Self {
        Self::new(
            &[
                ".env",
                ".secrets",
                "*.pem",
                "*.key",
                ".ssh/",
                ".aws/credentials",
                ".git-credentials",
                "id_rsa",
            ],
            &[".planning/", "docs/"],
        )
    }

    fn matches(resource: &str, pattern: &str) -> bool {
        if pattern.ends_with('/') {
            resource.contains(pattern)
        } else if let Some(stripped) = pattern.strip_prefix('*') {
            resource.ends_with(stripped) || resource.contains(stripped)
        } else {
            resource == pattern
                || resource.ends_with(&format!("/{}", pattern))
                || resource.split('/').any(|seg| seg == pattern)
        }
    }
}

impl PermissionPolicy for PathRulePermissionPolicy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&self, _action: &str, resource: &str) -> PermissionEffect {
        if self
            .deny_patterns
            .iter()
            .any(|p| Self::matches(resource, p))
        {
            return PermissionEffect::Deny;
        }
        if self.ask_patterns.iter().any(|p| Self::matches(resource, p)) {
            return PermissionEffect::Ask;
        }
        PermissionEffect::Allow
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ExecPolicy — préfixes de commandes autorisés / interdits
// ═══════════════════════════════════════════════════════════════════════════

/// Politique d'exécution par préfixes (port de `RuleBasedExecPolicy` + guard odysseus).
pub struct CommandPrefixExecPolicy {
    name: String,
    denied_prefixes: Vec<String>,
    denied_substrings: Vec<String>,
}

impl CommandPrefixExecPolicy {
    pub fn new(denied_prefixes: &[&str], denied_substrings: &[&str]) -> Self {
        Self {
            name: "command-prefix".into(),
            denied_prefixes: denied_prefixes.iter().map(|s| s.to_string()).collect(),
            denied_substrings: denied_substrings.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Deny-list par défaut (rm -rf sur racine, fork bomb, curl|sh, etc.).
    pub fn dangerous_defaults() -> Self {
        Self::new(
            &["rm -rf /", "mkfs", "dd if=", ":(){:|:&};:"],
            &["curl|sh", "wget|sh", "sudo rm", "chmod 777 /"],
        )
    }
}

impl ExecPolicy for CommandPrefixExecPolicy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&self, command: &str) -> ExecDecision {
        // Normalisation : "rm  -rf  /" et "rm\t-rf /" doivent matcher "rm -rf /".
        // (ÉVASION TROUVÉE PAR LE TEST ADVERSARIAL — sans ça, des espaces multiples
        // suffisaient à contourner la deny-list.)
        let c = command.split_whitespace().collect::<Vec<_>>().join(" ");
        for p in &self.denied_prefixes {
            if c.starts_with(p.as_str()) {
                return ExecDecision::Forbidden(format!("prefix interdit : {}", p));
            }
        }
        for s in &self.denied_substrings {
            if c.contains(s.as_str()) {
                return ExecDecision::Forbidden(format!("motif interdit : {}", s));
            }
        }
        // Pipeline "download | shell" : on segmente sur `|` et on regarde le
        // programme de tete + l'interpreteur en aval (curl http://x | sh).
        let segments: Vec<&str> = c.split('|').map(|s| s.trim()).collect();
        if segments.len() > 1 {
            const DOWNLOADERS: [&str; 2] = ["curl", "wget"];
            const SHELLS: [&str; 4] = ["sh", "bash", "zsh", "dash"];
            let head = segments[0].split_whitespace().next().unwrap_or("");
            let base = head.rsplit('/').next().unwrap_or(head);
            if DOWNLOADERS.contains(&base) {
                for seg in &segments[1..] {
                    let prog = seg.split_whitespace().next().unwrap_or("");
                    let pbase = prog.rsplit('/').next().unwrap_or(prog);
                    if SHELLS.contains(&pbase) {
                        return ExecDecision::Forbidden(format!(
                            "pipeline download|shell interdit : {} | {}",
                            base, pbase
                        ));
                    }
                }
            }
        }
        ExecDecision::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> ProjectBudget {
        ProjectBudget {
            max_tokens: 100,
            max_cost_usd: 1.0,
            max_iterations: 2,
            max_tool_calls: 3,
        }
    }

    #[test]
    fn budget_allows_under_limit() {
        let b = BudgetEnforcer::new(budget(), "run1");
        assert!(b.consume_tokens(10, 0.1).ok);
        assert!(b.consume_iteration().ok);
        assert!(b.consume_tool_call().ok);
        assert!(b.is_budget_ok().ok);
    }

    #[test]
    fn budget_denies_over_tokens() {
        let b = BudgetEnforcer::new(budget(), "run1");
        let v = b.consume_tokens(200, 0.0);
        assert!(!v.ok);
        assert!(v.reason.contains("Token budget exceeded"));
    }

    #[test]
    fn budget_denies_over_cost() {
        let b = BudgetEnforcer::new(budget(), "run1");
        let v = b.consume_tokens(1, 5.0);
        assert!(!v.ok);
        assert!(v.reason.contains("Cost budget exceeded"));
    }

    #[test]
    fn budget_denies_over_iterations() {
        let b = BudgetEnforcer::new(budget(), "run1");
        assert!(b.consume_iteration().ok); // 1
        assert!(b.consume_iteration().ok); // 2 == max -> encore ok
        assert!(!b.consume_iteration().ok); // 3 > max -> refusé
    }

    #[test]
    fn budget_percent_reports_max_axis() {
        let b = BudgetEnforcer::new(budget(), "run1");
        b.consume_tokens(50, 0.0); // 50%
        let v = b.is_budget_ok();
        assert!(v.ok);
        assert!((v.percent - 50.0).abs() < 0.01);
    }

    #[test]
    fn budget_alert_at_80_percent() {
        let b = BudgetEnforcer::new(budget(), "run1");
        b.consume_tokens(81, 0.0);
        assert!(b.alert_threshold_reached());
    }

    #[test]
    fn permission_denies_secrets() {
        let p = PathRulePermissionPolicy::sensitive_defaults();
        assert_eq!(
            p.evaluate("read", "/home/u/.ssh/id_rsa"),
            PermissionEffect::Deny
        );
        assert_eq!(p.evaluate("read", "config/.env"), PermissionEffect::Deny);
        assert_eq!(p.evaluate("read", "server.pem"), PermissionEffect::Deny);
    }

    #[test]
    fn permission_asks_planning_writes() {
        let p = PathRulePermissionPolicy::sensitive_defaults();
        assert_eq!(
            p.evaluate("write", ".planning/ROADMAP.md"),
            PermissionEffect::Ask
        );
        assert_eq!(p.evaluate("read", "README.md"), PermissionEffect::Allow);
    }

    #[test]
    fn exec_denies_dangerous_commands() {
        let p = CommandPrefixExecPolicy::dangerous_defaults();
        assert!(matches!(
            p.evaluate("rm -rf / --no-preserve-root"),
            ExecDecision::Forbidden(_)
        ));
        assert!(matches!(
            p.evaluate("curl http://x | sh"),
            ExecDecision::Forbidden(_)
        ));
        assert!(matches!(p.evaluate("cargo test"), ExecDecision::Allow));
    }

    #[test]
    fn permission_composition_via_intersection_is_monotone() {
        use omega_ports::IntersectionPermissionPolicy;
        let allow = PathRulePermissionPolicy::new(&[], &[]);
        let deny_secrets = PathRulePermissionPolicy::sensitive_defaults();
        let comp = IntersectionPermissionPolicy::new(vec![Box::new(allow), Box::new(deny_secrets)]);
        assert_eq!(comp.evaluate("read", ".env"), PermissionEffect::Deny);
        assert_eq!(comp.evaluate("read", "notes.md"), PermissionEffect::Allow);
    }
}
