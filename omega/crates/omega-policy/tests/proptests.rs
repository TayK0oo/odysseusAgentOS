//! Property tests (catalogue A2 du PLAN-MAITRE.md §7) sur le budget monotone
//! et le fail-closed des paliers.
//!
//! Garanties nommées :
//! - **budget monotone** : toute suite d'appels acceptés ne dépasse jamais le
//!   plafond — jamais N+1 exécutions pour un budget de N, usage == somme des
//!   demandes acceptées (un refus ne débite RIEN), y compris entrelacé
//!   (`prop_threaded_concurrent_monotone` : le budget étant thread-safe, des
//!   consommations concurrentes restent bornées ; on exige la même monotone
//!   séquentielle, rien de plus) ;
//! - **fail-closed des paliers** : une action non classée est toujours refusée,
//!   et une action T2 n'est jamais autorisée par défaut (palier absent,
//!   G1.sandbox fermé, approbations par défaut = `DenyAllApprovals`) ;
//! - **erreurs structurées** : un refus porte son axe / usage / plafond et sa
//!   demande — jamais de panic sur entrée utilisateur ;
//! - **données serde** : les types de budget restent sérialisables (config de
//!   donnée, pas de magie).
//!
//! Convention du fichier : les helpers `is_deny`/`deny_reason`/`allow_tier`
//! évitent les patterns `matches!(.., X { .. })` et les littéraux de struct
//! *dans les arguments* des macros `prop_assert!` (le parsing format-string
//! interne de ces macros refuse des braces à cet endroit).

use omega_policy::budget::{Budget, BudgetAxis, Demand, Limits, Usage};
use omega_policy::tiers::{
    ApprovalGate, DenyAllApprovals, ElevationRequest, Tier, TierDecision, TierPolicy,
};
use proptest::prelude::*;
use std::sync::Arc;

// ── Helpers de décision (sans braces dans les macros proptest) ──────────────

fn is_deny(d: &TierDecision) -> bool {
    matches!(d, TierDecision::Deny { .. })
}

fn deny_reason(d: &TierDecision) -> Option<&str> {
    match d {
        TierDecision::Deny { reason } => Some(reason),
        TierDecision::Allow { .. } => None,
    }
}

/// Égalité flottante tolérante (serde_json/ryu peut dériver d'1 ULP sur un f64
/// arbitraire ; les axes entiers restent exacts). Suffisant pour un plafond de
/// config : la frontière JSON ne change pas la décision sr-pratique.
fn approx_f64(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
}

fn allow_tier(d: &TierDecision) -> Option<Tier> {
    match d {
        TierDecision::Allow { tier } => Some(*tier),
        TierDecision::Deny { .. } => None,
    }
}

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

/// Les 11 entrées de la table standard documentée (source unique des sondes).
const STANDARD: [(&str, Tier); 11] = [
    ("read", Tier::T0),
    ("list", Tier::T0),
    ("grep", Tier::T0),
    ("write", Tier::T1),
    ("create", Tier::T1),
    ("edit", Tier::T1),
    ("remove", Tier::T1),
    ("network", Tier::T2),
    ("exec", Tier::T2),
    ("subagent", Tier::T3),
    ("sub-plan", Tier::T3),
];

// ── Stratégies proptest ─────────────────────────────────────────────────────

/// Plafonds bornés, strictement positifs : un budget finit toujours par
/// s'épuiser (l'illimité est couvert par le test unitaire `zero_limit…`).
fn arb_limits() -> impl Strategy<Value = Limits> {
    (1u64..=50, 10u64..=2000, 0.01f64..=10.0, 1u64..=60_000).prop_map(
        |(steps, tokens, cost, ms)| Limits {
            max_steps: steps,
            max_tokens: tokens,
            max_cost_usd: cost,
            max_time_ms: ms,
        },
    )
}

/// Une demande d'appel, bornée mais potentiellement plus grande que le budget
/// tiré séparément.
fn arb_demand(
    max_steps: u64,
    max_tokens: u64,
    max_cost: f64,
    max_ms: u64,
) -> impl Strategy<Value = Demand> {
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

/// Suite aléatoire de consommations (propriété 1 : aucun entrelacement ne
/// dépasse le plafond).
fn arb_demand_seq() -> impl Strategy<Value = Vec<Demand>> {
    prop::collection::vec(
        (0u64..=5, 0u64..=50, 0.0f64..=3.0, 0u64..=5_000).prop_map(|(steps, tokens, cost, ms)| {
            Demand {
                steps,
                tokens,
                cost_usd: cost,
                time_ms: ms,
            }
        }),
        12..100,
    )
}

/// Action tirée hors de la table standard : fail-closed vérifiable.
fn arb_unknown_action() -> impl Strategy<Value = String> {
    const KNOWN: [&str; 11] = [
        "read", "list", "grep", "write", "create", "edit", "remove", "network", "exec", "subagent",
        "sub-plan",
    ];
    proptest::string::string_regex("[a-z_]{1,40}[a-z0-9_]{0,20}")
        .unwrap()
        .prop_filter("action doit être hors table", move |s| {
            !KNOWN.contains(&s.as_str())
        })
}

/// Palier tiré à l'aveugle (pour polluer une table de config).
fn arb_tier() -> impl Strategy<Value = Tier> {
    prop_oneof![
        Just(Tier::T0),
        Just(Tier::T1),
        Just(Tier::T2),
        Just(Tier::T3),
    ]
}

/// Entrée (action, palier) de config de donnée, hors table standard.
fn arb_classified_entry() -> impl Strategy<Value = (String, Tier)> {
    (arb_unknown_action(), arb_tier())
}

/// Cas refusable : (plafonds, demande dont au moins un axe déborde à lui seul
/// dès le premier appel) — garantit la terminaison des propriétés en boucle.
fn arb_overflow_case() -> impl Strategy<Value = (Limits, Demand)> {
    arb_limits().prop_flat_map(|lim| {
        (
            Just(lim),
            arb_demand(10_000, 500_000, 1_000.0, 600_000).prop_filter(
                "au moins un axe doit déborder le plafond à lui seul",
                move |d| {
                    d.steps > lim.max_steps
                        || d.tokens > lim.max_tokens
                        || d.cost_usd > lim.max_cost_usd
                        || d.time_ms > lim.max_time_ms
                },
            ),
        )
    })
}

/// Cas complet de refus puis relance : (plafonds, demande qui déborde,
/// demande bornée pour la relance).
fn arb_refusal_case() -> impl Strategy<Value = (Limits, Demand, Demand)> {
    arb_limits().prop_flat_map(|lim| {
        (
            Just(lim),
            arb_demand(
                lim.max_steps + 200,
                lim.max_tokens + 5000,
                lim.max_cost_usd + 50.0,
                lim.max_time_ms + 30_000,
            )
            .prop_filter("la demande doit déborder (un axe à lui seul)", move |d| {
                d.steps > lim.max_steps
                    || d.tokens > lim.max_tokens
                    || d.cost_usd > lim.max_cost_usd
                    || d.time_ms > lim.max_time_ms
            }),
            arb_demand(
                lim.max_steps,
                lim.max_tokens,
                lim.max_cost_usd,
                lim.max_time_ms,
            ),
        )
    })
}

/// Demande qui déborde sur exactement UN axe documenté, les autres à zéro :
/// l'erreur doit nommer CET axe (erreur structurée).
fn arb_single_overflow(lim: Limits) -> impl Strategy<Value = (BudgetAxis, Demand)> {
    (0u8..4, 2u64..6).prop_map(move |(axis, mult)| {
        let mut d = Demand::default();
        let a = match axis {
            0 => {
                d.steps = lim.max_steps * mult;
                BudgetAxis::Steps
            }
            1 => {
                d.tokens = lim.max_tokens * mult;
                BudgetAxis::Tokens
            }
            2 => {
                d.cost_usd = lim.max_cost_usd * mult as f64;
                BudgetAxis::Cost
            }
            _ => {
                d.time_ms = lim.max_time_ms * mult;
                BudgetAxis::Time
            }
        };
        (a, d)
    })
}

/// Demande qui déborde sur les QUATRE axes à la fois : l'erreur décrit le
/// premier axe dans l'ordre documenté (steps, tokens, coût, temps).
fn arb_quadruple_overflow(lim: Limits) -> impl Strategy<Value = Demand> {
    Just(Demand {
        steps: lim.max_steps + 1,
        tokens: lim.max_tokens + 1,
        cost_usd: lim.max_cost_usd + 1.0,
        time_ms: lim.max_time_ms + 1,
    })
}

// ── Propriétés BUDGET ───────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // Propriété 1 : jamais N+1 exécutions pour un budget N, sur une suite
    // aléatoire complète.
    #[test]
    fn prop_budget_monotone_never_exceeds(
        limits in arb_limits(),
        calls in arb_demand_seq(),
    ) {
        let b = Budget::new(limits);
        let mut used = Usage::default();
        for d in &calls {
            if let Ok(new_usage) = b.consume_call(*d) {
                // L'usage ne bouge que sur accepté, et reste monotone croissant.
                prop_assert!(new_usage.steps >= used.steps);
                prop_assert!(new_usage.tokens >= used.tokens);
                prop_assert!(new_usage.cost_usd >= used.cost_usd);
                prop_assert!(new_usage.time_ms >= used.time_ms);
                used = new_usage;
            }
            // Invariant central : chaque axe borné par le plafond.
            prop_assert!(used.steps <= limits.max_steps);
            prop_assert!(used.tokens <= limits.max_tokens);
            prop_assert!(used.cost_usd <= limits.max_cost_usd);
            prop_assert!(used.time_ms <= limits.max_time_ms);
        }
        prop_assert_eq!(b.usage(), used);
    }

    // Propriété 1 bis : l'usage ne croît QUE sur accepté, vérifié appel par
    // appel (un refus laisse le compteur exactement inchangé).
    #[test]
    fn prop_usage_only_grows_on_accept(
        limits in arb_limits(),
        calls in arb_demand_seq(),
    ) {
        let b = Budget::new(limits);
        let mut used = Usage::default();
        for d in &calls {
            if let Ok(after) = b.consume_call(*d) {
                prop_assert!(after.steps >= used.steps);
                prop_assert!(after.time_ms >= used.time_ms);
                used = after;
            }
            prop_assert_eq!(b.usage(), used, "un refus laisse l'usage inchangé");
        }
    }

    // Propriété 2 : un refus ne débite RIEN (tout-ou-riten atomique) — et la
    // relance suivante qui tient passe toujours.
    #[test]
    fn prop_refusal_debits_nothing(refusal in arb_refusal_case()) {
        let (limits, over, then) = refusal;
        let b = Budget::new(limits);
        let before = b.usage();
        let e = b.consume_call(over).unwrap_err();
        prop_assert_eq!(b.usage(), before, "un refus ne débite RIEN (tout-ou-rien)");
        prop_assert_eq!(e.used, before);
        // Un appel qui tient passe toujours après un refus.
        if then.steps <= limits.max_steps
            && then.tokens <= limits.max_tokens
            && then.cost_usd <= limits.max_cost_usd
            && then.time_ms <= limits.max_time_ms
        {
            prop_assert!(b.consume_call(then).is_ok());
        }
    }

    // Propriété 3 : l'erreur d'épuisement est STRUCTURÉE (axe nommé, usage et
    // plafonnants exacts) — un panic serait un échec (interdit sur entrée
    // utilisateur, règle P1-3).
    #[test]
    fn prop_exhaustion_error_is_structured(overflow in arb_overflow_case()) {
        let (limits, d) = overflow;
        let b = Budget::new(limits);
        let e = b.consume_call(d).unwrap_err();
        prop_assert!(e.axis.is_some() || e.to_string().contains("fail-closed"));
        prop_assert_eq!(e.used, Usage::default());
        prop_assert_eq!(e.limit, limits);
    }

    // Propriété 4 : l'erreur de refus porte la demande qui l'a causée.
    #[test]
    fn prop_refusal_carries_the_demand(overflow in arb_overflow_case()) {
        let (limits, d) = overflow;
        let b = Budget::new(limits);
        prop_assert_eq!(b.consume_call(d).unwrap_err().requested, d);
    }

    // Propriété 5 : débordement sur UN axe ⇒ l'erreur nomme CET axe.
    #[test]
    fn prop_single_axis_overflow_is_named(overflow in arb_limits().prop_flat_map(|lim| {
        (Just(lim), arb_single_overflow(lim))
    })) {
        let (limits, (axis, d)) = overflow;
        let b = Budget::new(limits);
        let e = b.consume_call(d).unwrap_err();
        prop_assert_eq!(e.axis, Some(axis), "l'axe refusé doit être nommé");
    }

    // Propriété 6 : débordement des quatre axes ⇒ premier axe de l'ordre
    // documenté (steps, tokens, coût, temps) — décision déterministe.
    #[test]
    fn prop_multi_axis_overflow_reports_first_documented_axis(
        overflow in arb_limits().prop_flat_map(|lim| (Just(lim), arb_quadruple_overflow(lim))),
    ) {
        let (limits, d) = overflow;
        let b = Budget::new(limits);
        let e = b.consume_call(d).unwrap_err();
        prop_assert_eq!(e.axis, Some(BudgetAxis::Steps));
    }

    // Propriété 7 : une demande nulle n'est jamais refusée (proposé == utilisé,
    // toujours ≤ plafond) — quel que soit l'état du budget.
    #[test]
    fn prop_zero_demand_always_accepted(
        limits in arb_limits(),
        calls in arb_demand_seq(),
    ) {
        let b = Budget::new(limits);
        for d in calls {
            let _ = b.consume_call(d);
        }
        prop_assert!(b.consume_call(Demand::default()).is_ok());
    }

    // Propriété 8 : monotone sous entrelacement — des consommations
    // concurrentes (threads) restent bornées par les plafonds.
    #[test]
    fn prop_threaded_concurrent_monotone(
        limits in arb_limits(),
        demands in prop::collection::vec(
            (1u64..=5, 1u64..=50, 0.0f64..=1.0, 1u64..=2_000)
                .prop_map(|(steps, tokens, cost, ms)| Demand {
                    steps,
                    tokens,
                    cost_usd: cost,
                    time_ms: ms,
                }),
            4..24,
        ),
    ) {
        let budget = Arc::new(Budget::new(limits));
        std::thread::scope(|s| {
            for chunk in demands.chunks(4) {
                let b = Arc::clone(&budget);
                let slice: Vec<Demand> = chunk.to_vec();
                s.spawn(move || {
                    for d in slice {
                        let _ = b.consume_call(d);
                    }
                });
            }
        });
        let u = budget.usage();
        prop_assert!(u.steps <= limits.max_steps);
        prop_assert!(u.tokens <= limits.max_tokens);
        prop_assert!(u.cost_usd <= limits.max_cost_usd);
        prop_assert!(u.time_ms <= limits.max_time_ms);
    }

    // Propriété 9 : les types de budget restent des données serde (config).
    #[test]
    fn prop_serde_budget_types_roundtrip(
        limits in arb_limits(),
        demand in arb_demand(50, 2_000, 10.0, 60_000),
    ) {
        let ls = serde_json::to_string(&limits).unwrap();
        let back: Limits = serde_json::from_str(&ls).unwrap();
        prop_assert_eq!(back.max_steps, limits.max_steps);
        prop_assert_eq!(back.max_tokens, limits.max_tokens);
        prop_assert_eq!(back.max_time_ms, limits.max_time_ms);
        prop_assert!(approx_f64(back.max_cost_usd, limits.max_cost_usd), "plafond coût stable en serde");
        let ds = serde_json::to_string(&demand).unwrap();
        let back: Demand = serde_json::from_str(&ds).unwrap();
        prop_assert_eq!(back.steps, demand.steps);
        prop_assert_eq!(back.tokens, demand.tokens);
        prop_assert_eq!(back.time_ms, demand.time_ms);
        prop_assert!(approx_f64(back.cost_usd, demand.cost_usd), "coût demandé stable en serde");
    }

    // ── Propriétés PALIERS (défaut = REFUS) ─────────────────────────────────

    // Propriété 10 : action hors table → toujours DENY, quelle que soit
    // l'ouverture T2 (fail-closed).
    #[test]
    fn prop_unknown_action_always_denied(
        action in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        let p = TierPolicy::standard().with_t2_open(t2_open);
        prop_assert!(p.classify(&action).is_none());
        prop_assert!(is_deny(&p.authorize(&action)), "défaut = refus");
    }

    // Propriété 11 : T2 JAMAIS autorisé par défaut — ni G1.sandbox fermé,
    // ni approbations défaut, même avec un approbateur toujours-oui.
    #[test]
    fn prop_t2_never_allowed_without_open_and_approval(
        actions in proptest::collection::vec(
            prop_oneof![Just("network"), Just("exec")],
            1..10,
        ),
        sandbox_green_anyway in proptest::bool::ANY,
    ) {
        // Cas 1 : T2 fermé (G1.sandbox pas vert) → refus malgré l'approbateur.
        let p_closed = TierPolicy::standard()
            .with_t2_open(false)
            .with_approvals(Box::new(AlwaysApprove));
        // Cas 2 : T2 « ouvert » mais approbations défaut (refus) → refus.
        let p_open_default = TierPolicy::standard().with_t2_open(sandbox_green_anyway);
        for action in actions {
            if p_closed.classify(action) == Some(Tier::T2) {
                prop_assert!(is_deny(&p_closed.authorize(action)));
            }
            if p_open_default.classify(action) == Some(Tier::T2) {
                prop_assert!(is_deny(&p_open_default.authorize(action)));
            }
        }
    }

    // Propriété 12 : T2 ouvert + approbateur consentant ⇒ autorisé (le seul
    // chemin bleu du T2, contracté ici de bout en bout).
    #[test]
    fn prop_t2_open_and_approved_allows(
        extras in prop::collection::vec(arb_classified_entry(), 0..10),
    ) {
        let mut cases = vec!["network".to_string(), "exec".to_string()];
        for (action, tier) in extras {
            if tier == Tier::T2 {
                cases.push(action);
            }
        }
        for action in cases {
            let p = TierPolicy::standard()
                .with_classification(action.clone(), Tier::T2)
                .with_t2_open(true)
                .with_approvals(Box::new(AlwaysApprove));
            prop_assert_eq!(
                allow_tier(&p.authorize(&action)),
                Some(Tier::T2),
                "T2 ouvert + approuvé doit passer : {}",
                action
            );
        }
    }

    // Propriété 13 : T2 resté classé dans une table personnalisée avec
    // approbations défaut → refus, même si T2 est marqué ouvert.
    #[test]
    fn prop_custom_t2_open_still_needs_approval(
        action in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        let p = TierPolicy::standard()
            .with_classification(action.clone(), Tier::T2)
            .with_t2_open(t2_open);
        let d = p.authorize(&action);
        prop_assert!(is_deny(&d), "approbation défaut = refus T2 : {}", d);
    }

    // Propriété 14 : la table standard reflète exactement le palier documenté
    // (classification ET décision cohérentes sur les 11 entrées).
    #[test]
    fn prop_standard_table_tier_matches_decision(t2_open in proptest::bool::ANY) {
        let p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in STANDARD {
            prop_assert_eq!(p.classify(action), Some(tier));
            let d = p.authorize(action);
            if tier == Tier::T2 {
                prop_assert!(is_deny(&d), "T2 fermé par défaut : {}", d);
            } else {
                prop_assert_eq!(allow_tier(&d), Some(tier));
            }
        }
    }

    // Propriété 15 : T0 lecture seule reste stable (autorisé) quel que soit le
    // nombre d'actions polluées T1/T2/T3 ajoutées par la config.
    #[test]
    fn prop_t0_read_only_stability(
        junk_classified in prop::collection::vec(arb_classified_entry(), 1..20),
        t2_open in proptest::bool::ANY,
    ) {
        let mut p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in junk_classified {
            p = p.with_classification(action, tier);
        }
        prop_assert_eq!(allow_tier(&p.authorize("read")), Some(Tier::T0));
    }

    // Propriété 16 : T1 écriture canari reste autorisé, quelle que soit la
    // pollution de table et l'état de T2.
    #[test]
    fn prop_t1_actions_always_allowed(
        junk_classified in prop::collection::vec(arb_classified_entry(), 1..20),
        t2_open in proptest::bool::ANY,
    ) {
        let mut p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in junk_classified {
            p = p.with_classification(action, tier);
        }
        for (action, tier) in STANDARD {
            if tier == Tier::T1 {
                prop_assert_eq!(
                    allow_tier(&p.authorize(action)),
                    Some(Tier::T1),
                    "T1 stable : {}",
                    action
                );
            }
        }
    }

    // Propriété 17 : T3 coût/sous-plan autorisé — le garde quantitatif est le
    // Budget, seul et séparé ; la politique par paliers doit passer.
    #[test]
    fn prop_t3_actions_always_allowed(
        junk_classified in prop::collection::vec(arb_classified_entry(), 1..20),
        t2_open in proptest::bool::ANY,
    ) {
        let mut p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in junk_classified {
            p = p.with_classification(action, tier);
        }
        for (action, tier) in STANDARD {
            if tier == Tier::T3 {
                prop_assert_eq!(allow_tier(&p.authorize(action)), Some(Tier::T3));
            }
        }
    }

    // Propriété 18 : table personnalisée (config de donnée) — toute action ABSENTE
    // de la table reste refusée, fail-closed.
    #[test]
    fn prop_custom_table_fail_closed(
        entries in prop::collection::vec(arb_classified_entry(), 0..20),
        probe in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        prop_assume!(!entries.iter().any(|(a, _)| *a == probe));
        let mut p = TierPolicy::standard().with_t2_open(t2_open);
        for (action, tier) in entries {
            p = p.with_classification(action, tier);
        }
        prop_assert!(p.classify(&probe).is_none());
        prop_assert!(is_deny(&p.authorize(&probe)), "après table, absence = refus");
    }

    // Propriété 19 : tout refus coté décision porte une raisons traçable non
    // vide (jamais un refus muet).
    #[test]
    fn prop_deny_decision_carries_a_reason(
        action in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        let p = TierPolicy::standard().with_t2_open(t2_open);
        let d = p.authorize(&action);
        prop_assert!(deny_reason(&d).map(|r| !r.is_empty()).unwrap_or(false), "{}", d);
    }

    // Propriété 20 : l'affichage d'une décision est lisible (allow*/deny*) —
    // traçable dans la journal (A4 P1-4 s'appuiera dessus).
    #[test]
    fn prop_decision_display_is_traced(
        action in arb_unknown_action(),
        t2_open in proptest::bool::ANY,
    ) {
        let p = TierPolicy::standard().with_t2_open(t2_open);
        let shown = p.authorize(&action).to_string();
        prop_assert!(
            shown.starts_with("allow ") || shown.starts_with("deny "),
            "décision tracable, obtenu : {shown}"
        );
    }

    // Propriété 21 : le port d'approbation défaut refuse TOUTE demande
    // d'élévation T1→T2 (défaut-refus contractuel).
    #[test]
    fn prop_default_elevation_always_refused(
        action in proptest::string::string_regex("[a-z_]{1,30}").unwrap(),
        reason in proptest::string::string_regex("[a-zA-Z0-9 ]{0,80}").unwrap(),
    ) {
        let denied = DenyAllApprovals.approve(&ElevationRequest {
            action: action.clone(),
            tier: Tier::T2,
            reason,
        });
        prop_assert!(!denied, "défaut d'approbation = refus, pour: {}", action);
    }

    // Propriété 22 : une demande d'élévation reste une donnée serde
    // (transportable vers la boucle UI A2 sans frontière perdue).
    #[test]
    fn prop_elevation_request_serde_roundtrip(
        action in proptest::string::string_regex("[a-z_]{1,30}").unwrap(),
        reason in proptest::string::string_regex("[a-zA-Z0-9 ]{0,80}").unwrap(),
    ) {
        let req = ElevationRequest {
            action: action.clone(),
            tier: Tier::T2,
            reason: reason.clone(),
        };
        let back: ElevationRequest =
            serde_json::from_str(&serde_json::to_string(&req).unwrap()).unwrap();
        prop_assert_eq!(back.action, action);
        prop_assert_eq!(back.tier, Tier::T2);
        prop_assert_eq!(back.reason, reason);
    }
}
