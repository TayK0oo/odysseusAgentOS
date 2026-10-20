//! Property tests (catalogue A2 du PLAN-MAITRE.md §7) sur le budget monotone
//! et le fail-closed des paliers.
//!
//! Garanties nommées :
//! - **budget monotone** : toute suite d'appels sous budget N ⇒ jamais N+1
//!   exécutions, usage == somme des demandes acceptées (aucun débit de refus) ;
//! - **fail-closed des paliers** : une action inconnue est toujours refusée,
//!   et une action T2 n'est jamais autorisée par défaut (T0..T3 absents,
//!   G1.sandbox fermé, approbations par défaut).

use omega_policy::budget::{Budget, Demand, Limits};
use omega_policy::tiers::{
    ApprovalGate, DenyAllApprovals, ElevationRequest, Tier, TierDecision, TierPolicy,
};
use proptest::prelude::*;

// ── Stratégies proptest ─────────────────────────────────────────────────────

/// Approbateur toujours-oui : prouve que la fermeture T2 gagne malgré lui.
struct AlwaysApprove;
impl ApprovalGate for AlwaysApprove {
    fn name(&self) -> &str {
        "always-approve"
    }
    fn approve(&self, _req: &ElevationRequest) -> bool {
        true
    }
}

/// Plafonds bornés, strictement positifs : un budget finit toujours par
/// s'épuiser (l'illimité est couvert par le test unitaire `zero_limit…`).
fn arb_limits() -> impl Strategy<Value = Limits> {
    (
        1u64..=50,
        10u64..=2000,
        0.01f64..=10.0,
        1u64..=60_000,
    )
        .prop_map(|(steps, tokens, cost, ms)| Limits {
            max_steps: steps,
            max_tokens: tokens,
            max_cost_usd: cost,
            max_time_ms: ms,
        })
}

/// Une demande d'appel (taille bornée par rapport au budget, mais
/// potentiellement plus grande que le budget via les plafonds aléatoires).
fn arb_demand(max_steps: u64, max_tokens: u64, max_cost: f64, max_ms: u64) -> impl Strategy<Value = Demand> {
    (
        0u64..=max_steps,
        0u64..=max_tokens,
        (0.0f64..max_cost),
        0u64..=max_ms,
    )
        .prop_map(|(steps, tokens, cost, ms)| Demand {
            steps,
            tokens,
            cost_usd: cost,
            time_ms: ms,
        })
}

/// Demande dont au moins un axe dépasse à lui seul les plafonds : garantit
/// la terminaison de la propriété (le premier appel est refusé).
fn arb_overflowing(lim: Limits) -> impl Strategy<Value = Demand> {
    arb_demand(10_000, 500_000, 1_000.0, 600_000)
        .prop_filter(
            "au moins un axe doit dépasser le plafond à lui seul",
            move |d| {
                d.steps > lim.max_steps
                    || d.tokens > lim.max_tokens
                    || d.cost_usd > lim.max_cost_usd
                    || d.time_ms > lim.max_time_ms
            },
        )
}


fn arb_demand_seq() -> impl Strategy<Value = Vec<Demand>> {
    prop::collection::vec(
        (0u64..=5, 0u64..=50, (0.0f64..3.0), 0u64..=5_000).prop_map(
            |(steps, tokens, cost, ms)| Demand {
                steps,
                tokens,
                cost_usd: cost,
                time_ms: ms,
            },
        ),
        12..100,
    )
}

/// Action tirée hors de la table standard : fail-close vérifiable.
fn arb_unknown_action() -> impl Strategy<Value = String> {
    const KNOWN: [&str; 11] = [
        "read", "list", "grep", "write", "create", "edit", "remove", "network", "exec",
        "subagent", "sub-plan",
    ];
    proptest::string::string_regex("[a-z_]{1,40}[a-z0-9_]{0,20}").prop_filter(
        "action doit être hors table",
        |s| !KNOWN.contains(&s.as_str()),
    )
}

// ── Propriété 1 : jamais N+1 exécutions pour un budget N ────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn prop_budget_monotone_never_exceeds(
        limits in arb_limits(),
        calls in arb_demand_seq(),
    ) {
        let b = Budget::new(limits);
        let mut used = omega_policy::budget::Usage::default();
        for d in &calls {
            if let Ok(new_usage) = b.consume_call(*d) {
                // L'usage ne bouge que sur accepté, et un accepté est
                // toujours >= l'usage précédent (monotone croissant).
                prop_assert!(new_usage.steps >= used.steps);
                prop_assert!(new_usage.tokens >= used.tokens);
                prop_assert!(new_usage.cost_usd >= used.cost_usd);
                prop_assert!(new_usage.time_ms >= used.time_ms);
                used = new_usage;
            }
            // Invariant central : tout axe borné par le plafond — jamais
            // N+1 exécutions pour un budget de N.
            prop_assert!(used.steps <= limits.max_steps);
            prop_assert!(used.tokens <= limits.max_tokens);
            prop_assert!(used.cost_usd <= limits.max_cost_usd);
            prop_assert!(used.time_ms <= limits.max_time_ms);
        }
        prop_assert_eq!(b.usage(), used);
    }

    // ── Propriété 2 : refus d'appel ne débite rien ───────────────────────────
    #[test]
    fn prop_refusal_debits_nothing(
        limits in arb_limits(),
        over in arb_demand(limits.max_steps + 200, limits.max_tokens + 5000, limits.max_cost_usd + 50.0, limits.max_time_ms + 30_000),
        then in arb_demand(limits.max_steps, limits.max_tokens, limits.max_cost_usd, limits.max_time_ms),
    ) {
        let b = Budget::new(limits);
        let before = b.usage();
        let r = b.consume_call(over);
        if r.is_err() {
            prop_assert_eq!(b.usage(), before, "un refus ne débite RIEN");
            // Suite : un appel qui tient passe toujours (monotonie croissante
            // mais bornée).
            if then.steps <= limits.max_steps
                && then.tokens <= limits.max_tokens
                && then.cost_usd <= limits.max_cost_usd
                && then.time_ms <= limits.max_time_ms
            {
                prop_assert!(b.consume_call(then).is_ok());
            }
        }
    }

    // ── Propriété 3 : erreur structurée, jamais panic ────────────────────────
    #[test]
    fn prop_exhaustion_error_is_structured(
        limits in arb_limits(),
        d in arb_demand(10_000, 500_000, 1_000.0, 600_000),
    ) {
        let b = Budget::new(limits);
        // Conserver la SERIE jusqu'à l'échec : soit chaque erreur est
        // structurée (axis renseigné + used inchangé + limit), soit un panic
        // (interdit par la règle P1-3 « jamais panic sur entrée utilisateur »
        // — un panic est un échec de test).
        loop {
            let before = b.usage();
            match b.consume_call(d) {
                Ok(after) => prop_assert!(after.steps >= before.steps),
                Err(e) => {
                    prop_assert!(e.axis.is_some() || e.to_string().contains("fail-closed"));
                    prop_assert_eq!(e.used, before);
                    prop_assert_eq!(e.limit, limits);
                    break;
                }
            }
        }
    }

    // ── Propriété 4 : action hors table → toujours DENY (fail-closed) ────────
    #[test]
    fn prop_unknown_action_always_denied(
        action in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        let p = TierPolicy::standard().with_t2_open(t2_open);
        prop_assert!(p.classify(&action).is_none());
        prop_assert!(matches!(
            p.authorize(&action),
            TierDecision::Deny { .. }
        ));
    }

    // ── Propriété 5 : T2 jamais autorisé tant que fermé/appro refused ────────
    #[test]
    fn prop_t2_never_allowed_without_open_and_approval(
        actions in proptest::collection::vec(
            prop_oneof![Just("network"), Just("exec")],
            1..10,
        ),
        sandbox_green_anyway in proptest::bool::ANY,
    ) {
        // Cas 1 : T2 fermé (G1.sandbox pas vert) → DENY même si un
        // approbateur dit oui (fermeture plus forte que l'approbation).
        let p_closed = TierPolicy::standard()
            .with_t2_open(false)
            .with_approvals(Box::new(AlwaysApprove));
        // Cas 2 : approuvations quelconques mais défaut (refus) → DENY.
        let p_open_default = TierPolicy::standard().with_t2_open(sandbox_green_anyway);
        for action in actions {
            if p_closed.classify(action) == Some(Tier::T2) {
                prop_assert!(matches!(
                    p_closed.authorize(action),
                    TierDecision::Deny { .. }
                ));
            }
            if p_open_default.classify(action) == Some(Tier::T2) {
                prop_assert!(matches!(
                    p_open_default.authorize(action),
                    TierDecision::Deny { .. }
                ));
            }
        }
    }

    // ── Propriété 6 : le comportement "palier" ne modifie pas la décision
    //    T0 — la lecture seule reste autorisée dans TOUTE config de la table ──
    #[test]
    fn prop_t0_read_only_stability(
        junk_classified in proptest::collection::vec(
            (arb_unknown_action(), prop_oneof![Just(Tier::T1), Just(Tier::T2), Just(Tier::T3)]),
            1..20,
        ),
        t2_open in proptest::bool::ANY,
    ) {
        let mut p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in &junk_classified {
            p = p.with_classification(action.clone(), *tier);
        }
        prop_assert_eq!(p.authorize("read"), TierDecision::Allow { tier: Tier::T0 });
        // Avec un nombre quelconque d'actions classifiées T1/T2/T3,
        // la lecture (T0) reste stable seule.
        prop_assert_eq!(p.classify("read"), Some(Tier::T0));
    }

    // ── Propriété 7 : avec deny-all par défaut, T1→T2 demande est refuse ────
    #[test]
    fn prop_default_elevation_always_refused(
        action in proptest::string::string_regex("[a-z_]{1,30}"),
        reason in proptest::string::string_regex("[a-zA-Z0-9 ]{0,80}"),
    ) {
        // Contract check : DenyAllApprovals devrait toujours renvoyer false.
        let dfa = DenyAllApprovals;
        prop_assert!(!dfa.approve(&ElevationRequest {
            action: action.clone(),
            tier: Tier::T2,
            reason,
        }));
    }
}
