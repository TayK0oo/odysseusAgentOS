//! Contrats comportementaux ciblant les mutants survivants de `omega-policy`
//! (`mutants.out/missed.txt`). Doubles locaux uniquement, API publique.
//!
//! Les mutants équipotents (preuve formelle) :
//! - `:259`, `:260` (`||` -> `&&` dans `PathRulePermissionPolicy::matches`) :
//!   les trois disjoncts `== pattern`, `ends_with("/pattern")` et
//!   `split('/').any(seg == pattern)` sont liés par implication
//!   (`a ⇒ c` et `b ⇒ c`), donc `a || b || c ≡ c` ; muter un `||` en `&&`
//!   laisse `c` dominant ⇒ sortie identique.
//! - `:337` (`>` -> `>=` sur `segments.len() > 1`) : `split('|')` donne
//!   toujours ≥ 1 segment ; la branche pipeline ne s'active que si un segment
//!   aval est un shell, ce qui exige `segments.len() >= 2`. Le mutant abaisse
//!   le seuil à `>= 1` mais la garde interne `DOWNLOADERS.contains(base)` plus
//!   la boucle sur `segments[1..]` (vide) rendent le comportement identique.
//! - `:160` (`>` -> `>=` sur `max_cost_usd > 0.0`) : avec `max_cost_usd > 0`,
//!   le garde original et muté sont tous deux vrais ; avec `max_cost_usd == 0`,
//!   `pct` retourne 0.0 dans les deux cas ⇒ aucun refus. Équipotent.
//! - `:130:55` (delete `!`) dans `LlmCompactionEngine` : déjà documenté
//!   (chemin indépendant).

use omega_policy::budget::{Budget, Limits};
use omega_policy::tiers::{ApprovalGate, DenyAllApprovals};
use omega_policy::{
    BudgetEnforcer, CommandPrefixExecPolicy, PathRulePermissionPolicy, PolicyRunBudget,
    ProjectBudget,
};
use omega_ports::{ExecPolicy, PermissionPolicy, RunBudget};
use omega_types::PermissionEffect;

// --- tue :92 (`run_id` -> "" et "xyzzy") ----------------------------------
#[test]
fn run_id_returns_constructor_value() {
    let e = BudgetEnforcer::new(ProjectBudget::default(), "run-42");
    assert_eq!(e.run_id(), "run-42");
}

// --- tue :104 (`>` -> `>=` dans consume_tokens, axe tokens) -----------------
#[test]
fn consume_tokens_accepts_exactly_at_limit() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 100,
            max_cost_usd: 10.0,
            max_iterations: 0,
            max_tool_calls: 0,
        },
        "r",
    );
    let v = b.consume_tokens(100, 0.0);
    assert!(
        v.ok,
        "exactement la limite doit être acceptée (mutant `>=` refuserait)"
    );
}

// --- tue :113 (`>` -> `>=` dans consume_iteration) --------------------------
#[test]
fn consume_iteration_accepts_exactly_at_limit() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 0,
            max_cost_usd: 0.0,
            max_iterations: 3,
            max_tool_calls: 0,
        },
        "r",
    );
    for _ in 0..3 {
        assert!(b.consume_iteration().ok);
    }
    assert!(!b.consume_iteration().ok, "le 4e doit être refusé");
}

// --- tue :144 (`+=` -> `*=`) et :145 (`>` -> `==`/`>=`) --------------------
#[test]
fn consume_tool_call_increments_and_accepts_at_limit() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 0,
            max_cost_usd: 0.0,
            max_iterations: 0,
            max_tool_calls: 2,
        },
        "r",
    );
    assert!(b.consume_tool_call().ok);
    assert!(b.consume_tool_call().ok); // exactement la limite
    assert_eq!(
        b.usage().tool_calls,
        2,
        "le compteur doit être 2 (mutant `*=` resterait 0)"
    );
    assert!(!b.consume_tool_call().ok, "le 3e doit être refusé");
}

// --- tue :160 (`>` -> `==` et `<`) sur le garde max_cost_usd > 0.0 ---------
#[test]
fn is_budget_ok_runs_cost_check_when_budget_positive() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 0,
            max_cost_usd: 1.0,
            max_iterations: 0,
            max_tool_calls: 0,
        },
        "r",
    );
    b.consume_tokens(0, 1.5);
    let v = b.is_budget_ok();
    assert!(
        !v.ok,
        "coût dépassé doit être détecté (mutants `==`/`<` sauteraient le garde)"
    );
}

// --- tue :162 et :190 (`>` -> `==`/`>=`) sur p > 100.0 ---------------------
#[test]
fn is_budget_ok_accepts_exactly_at_hundred_percent() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 100,
            max_cost_usd: 0.0,
            max_iterations: 0,
            max_tool_calls: 0,
        },
        "r",
    );
    b.consume_tokens(100, 0.0);
    let v = b.is_budget_ok();
    assert!(
        v.ok,
        "exactement 100 % doit être accepté (mutants `==`/`>=` refuseraient)"
    );
}

// --- tue :113 (`>` -> `>=` dans consume_tokens, axe coût) -------------------
#[test]
fn consume_tokens_cost_accepts_exactly_at_limit() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 0,
            max_cost_usd: 1.0,
            max_iterations: 0,
            max_tool_calls: 0,
        },
        "r",
    );
    let v = b.consume_tokens(0, 1.0);
    assert!(
        v.ok,
        "exactement la limite de coût doit être acceptée (mutant `>=` refuserait)"
    );
}

// --- tue :162 (`>` -> `==`/`>=` sur p > 100.0, axe coût) ------------------
#[test]
fn is_budget_ok_cost_accepts_exactly_at_hundred_percent() {
    let b = BudgetEnforcer::new(
        ProjectBudget {
            max_tokens: 0,
            max_cost_usd: 1.0,
            max_iterations: 0,
            max_tool_calls: 0,
        },
        "r",
    );
    b.consume_tokens(0, 1.0);
    let v = b.is_budget_ok();
    assert!(
        v.ok,
        "exactement 100 % de coût doit être accepté (mutants `==`/`>=` refuseraient)"
    );
}

#[test]
fn alert_threshold_not_reached_on_empty_usage() {
    let b = BudgetEnforcer::new(ProjectBudget::default(), "r");
    assert!(!b.alert_threshold_reached(), "0 % < 80 % : pas d'alerte");
}

// --- tue :256 (`||` -> `&&` sur ends_with/contains) ------------------------
#[test]
fn path_rule_matches_contains_without_ends_with() {
    let p = PathRulePermissionPolicy::new(&["*foo"], &[]);
    // "xfoo" ne finit pas par "foo" mais le contient -> l'original refuse (Deny).
    // Le mutant `&&` exigerait les deux -> Allow.
    assert_eq!(
        p.evaluate("any", "xfooy"),
        PermissionEffect::Deny,
        "contains suffit (mutant `&&` donnerait Allow)"
    );
}

// --- tue :267 (name -> "" et "xyzzy") -------------------------------------
#[test]
fn path_rule_name_is_path_rule() {
    let p = PathRulePermissionPolicy::new(&[], &[]);
    assert_eq!(p.name(), "path-rule");
}

// --- tue :316 (name -> "" et "xyzzy") -------------------------------------
#[test]
fn command_prefix_name_is_command_prefix() {
    let c = CommandPrefixExecPolicy::new(&[], &[]);
    assert_eq!(c.name(), "command-prefix");
}

// --- tue :337 (`>` -> `>=` sur segments.len() > 1) : EQUIVALENT ----------
// Pas de test : abaisser le seuil à `>= 1` ne change rien (segments[1..] vide
// pour un seul segment, et la garde DOWNLOADERS.contains(base) exige un
// downloader en tête). Preuve formelle dans le Livrable D.

// --- tue tiers.rs:74 (name -> "" et "xyzzy") ------------------------------
#[test]
fn deny_all_approvals_name_is_deny_all() {
    let g = DenyAllApprovals;
    assert_eq!(g.name(), "deny-all");
}

// --- tue budget.rs:134 (`limits` -> Default::default()) --------------------
#[test]
fn budget_limits_returns_configured_limits() {
    let b = Budget::new(Limits {
        max_steps: 3,
        max_tokens: 100,
        max_cost_usd: 1.0,
        max_time_ms: 10_000,
    });
    let l = b.limits();
    assert_eq!(
        l.max_tokens, 100,
        "limits() doit renvoyer les plafonds (mutant Default::default() donnerait 0)"
    );
}

// --- tue run_budget.rs:27 (`consume` -> Ok(Default::default())) -------------
#[test]
fn policy_run_budget_consume_returns_real_usage() {
    let inner = std::sync::Arc::new(Budget::new(Limits {
        max_steps: 3,
        max_tokens: 100,
        max_cost_usd: 1.0,
        max_time_ms: 10_000,
    }));
    let rb = PolicyRunBudget::new(inner);
    let d = omega_ports::BudgetDemand {
        steps: 1,
        tokens: 10,
        cost_usd: 0.1,
        time_ms: 100,
    };
    let u = rb.consume(d).expect("doit être accepté");
    assert_eq!(u.steps, 1);
    assert_eq!(u.tokens, 10);
    assert!(
        rb.consume(d).is_ok(),
        "deuxième appel doit aussi passer (2/3)"
    );
}

// --- tue run_budget.rs:54 (`usage` -> Default::default()) -------------------
#[test]
fn policy_run_budget_usage_reflects_consumed() {
    let inner = std::sync::Arc::new(Budget::new(Limits {
        max_steps: 3,
        max_tokens: 100,
        max_cost_usd: 1.0,
        max_time_ms: 10_000,
    }));
    let rb = PolicyRunBudget::new(inner);
    let d = omega_ports::BudgetDemand {
        steps: 1,
        tokens: 10,
        cost_usd: 0.1,
        time_ms: 100,
    };
    rb.consume(d).unwrap();
    let u = rb.usage();
    assert_eq!(u.steps, 1);
    assert_eq!(u.tokens, 10);
}

// --- tue tiers.rs:74 (name -> "" et "xyzzy") ------------------------------
// (déjà couvert par deny_all_approvals_name_is_deny_all ci-dessus)
