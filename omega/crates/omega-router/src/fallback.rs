//! Politique de fallback + circuit breaker a etats.
//!
//! Machine a etats par modele :
//! `Closed` -> `Open` (apres N echecs consecutifs) -> `HalfOpen` (delai
//! `open_timeout` ecoule, une sonde autorisee) -> `Closed` (sonde reussie)
//! ou `Open` (sonde echouee, le delai repart de zero).
//!
//! Le circuit breaker persiste sur l'instance ; l'ensemble des modeles deja
//! essayes dans un run (`tried`) est porte par l'appelant et reinitialise a
//! chaque requete (REVUE-W4 N2). Une sonde `HalfOpen` consomme un marqueur
//! interne dans `tried` : une seule sonde par run.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use omega_ports::FallbackPolicy;
use omega_types::{Capabilities, ModelRecord, ModelTier, RouterDecision, TaskType};

/// Defaut : nombre d'echecs consecutifs avant ouverture du circuit.
pub const DEFAULT_MAX_FAILURES: u32 = 3;
/// Defaut : duree d'ouverture avant passage en `HalfOpen` (sonde autorisee).
pub const DEFAULT_OPEN_TIMEOUT: Duration = Duration::from_secs(60);

/// Marqueur interne insere dans `tried` quand une sonde `HalfOpen` est
/// proposee : garantit une seule sonde par run. Un `tried` frais par requete
/// (contrat `FallbackPolicy`) ne le contient jamais.
const HALF_OPEN_PROBE_MARKER: &str = "__omega_half_open_probe__";

/// Etat du circuit breaker pour un modele.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Trafic normal ; les echecs consecutifs sont comptes.
    Closed,
    /// Modele exclu du fallback jusqu'a la fin du delai d'ouverture.
    Open,
    /// Delai ecoule : une sonde est autorisee. Succes -> `Closed`,
    /// echec -> `Open` (delai reparti de zero).
    HalfOpen,
}

/// Compteur persistant par modele.
struct BreakerEntry {
    consecutive_failures: u32,
    opened_at: Option<Instant>,
}

/// Fallback par tier avec circuit breaker a etats.
///
/// Le circuit breaker (`breakers`) persiste sur l'instance. L'ensemble des
/// modeles deja essayes dans un run (`tried`) est porte par l'appelant et
/// reinitialise a chaque requete : il ne fuit pas d'une requete a l'autre
/// (REVUE-W4 N2).
///
/// `next_fallback` prefere les modeles `Closed` (escalade meme tier -> tier
/// superieur -> `SelfHosted`) et n'autorise une sonde `HalfOpen` que si aucun
/// `Closed` n'est eligible, a raison d'une seule sonde par run.
pub struct TierFallbackPolicy<S: omega_ports::ScoringPolicy> {
    scoring: S,
    max_failures: u32,
    open_timeout: Duration,
    breakers: HashMap<String, BreakerEntry>,
    now: Arc<dyn Fn() -> Instant + Send + Sync>,
}

impl<S: omega_ports::ScoringPolicy> TierFallbackPolicy<S> {
    /// Construit la politique. `max_failures` (N) est sature a >= 1 ;
    /// le delai d'ouverture vaut [`DEFAULT_OPEN_TIMEOUT`] et l'horloge est
    /// l'horloge systeme (voir [`Self::with_clock`] pour les tests).
    pub fn new(scoring: S, max_failures: u32) -> Self {
        Self {
            scoring,
            max_failures: max_failures.max(1),
            open_timeout: DEFAULT_OPEN_TIMEOUT,
            breakers: HashMap::new(),
            now: Arc::new(Instant::now),
        }
    }

    /// Configure le delai d'ouverture avant sonde `HalfOpen` (builder).
    pub fn with_open_timeout(mut self, timeout: Duration) -> Self {
        self.open_timeout = timeout;
        self
    }

    /// Injecte l'horloge (builder). Permet des tests deterministes sans
    /// `sleep` : l'appelant fournit une horloge manuelle avancee a la main.
    pub fn with_clock<F>(mut self, clock: F) -> Self
    where
        F: Fn() -> Instant + Send + Sync + 'static,
    {
        self.now = Arc::new(clock);
        self
    }

    /// Seuil N d'echecs consecutifs avant ouverture.
    pub fn max_failures(&self) -> u32 {
        self.max_failures
    }

    /// Delai d'ouverture avant sonde `HalfOpen`.
    pub fn open_timeout(&self) -> Duration {
        self.open_timeout
    }

    /// Nombre d'echecs consecutifs (0 si `Closed` sans echec ou inconnu ;
    /// vaut le seuil tant que le circuit est `Open`).
    pub fn failure_count(&self, model_id: &str) -> u32 {
        self.breakers
            .get(model_id)
            .map(|e| e.consecutive_failures)
            .unwrap_or(0)
    }

    /// Etat courant du circuit pour un modele. Un modele inconnu est
    /// `Closed`. Le passage `Open` -> `HalfOpen` est derive paresseusement
    /// de l'horloge injectee : aucun `sleep` ni mutation sur lecture.
    pub fn state(&self, model_id: &str) -> CircuitState {
        match self.breakers.get(model_id) {
            None => CircuitState::Closed,
            Some(entry) => match entry.opened_at {
                None => CircuitState::Closed,
                Some(opened_at) => {
                    if (self.now)() >= opened_at + self.open_timeout {
                        CircuitState::HalfOpen
                    } else {
                        CircuitState::Open
                    }
                }
            },
        }
    }

    /// Enregistre un succes pour un modele : le compteur est remis a zero
    /// et le circuit repasse `Closed` (REVUE-W4 M4). C'est aussi la voie de
    /// fermeture apres une sonde `HalfOpen` reussie. Sans effet sur un
    /// modele inconnu.
    pub fn record_success(&mut self, model_id: &str) {
        self.breakers.remove(model_id);
    }

    /// Selection par escalade de tier dans un vivier pre-filtre :
    /// meme tier, puis tier superieur, puis `SelfHosted`.
    fn select_by_tier(
        &self,
        task: TaskType,
        required_context: u32,
        candidates: &[ModelRecord],
        pool: &[&ModelRecord],
        current_tier_rank: u8,
    ) -> Option<RouterDecision> {
        // 1. Meme tier (redondance catalogue).
        let same_tier: Vec<&ModelRecord> = pool
            .iter()
            .copied()
            .filter(|m| m.tier.rank() == current_tier_rank)
            .collect();
        if let Some(next) = pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &same_tier,
        ) {
            return Some(next);
        }

        // 2. Tier superieur.
        let higher_tier: Vec<&ModelRecord> = pool
            .iter()
            .copied()
            .filter(|m| m.tier.rank() > current_tier_rank)
            .collect();
        if let Some(next) = pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &higher_tier,
        ) {
            return Some(next);
        }

        // 3. Self-hosted.
        let self_hosted: Vec<&ModelRecord> = pool
            .iter()
            .copied()
            .filter(|m| matches!(m.tier, ModelTier::SelfHosted))
            .collect();
        pick_best(
            &self.scoring,
            task,
            required_context,
            candidates,
            &self_hosted,
        )
    }
}

impl<S: omega_ports::ScoringPolicy> FallbackPolicy for TierFallbackPolicy<S> {
    fn record_failure(&mut self, model_id: &str) {
        let now = (self.now)();
        let max = self.max_failures;
        let timeout = self.open_timeout;
        let entry = self
            .breakers
            .entry(model_id.to_string())
            .or_insert(BreakerEntry {
                consecutive_failures: 0,
                opened_at: None,
            });
        match entry.opened_at {
            // Circuit ouvert : un echec avant la fin du delai ne prolonge pas
            // l'ouverture (on garde `opened_at` d'origine). Un echec apres le
            // delai signifie que la sonde `HalfOpen` a echoue : on re-ouvre
            // et le delai repart de zero.
            Some(opened_at) => {
                if now >= opened_at + timeout {
                    entry.opened_at = Some(now);
                    entry.consecutive_failures = max;
                }
            }
            // Circuit ferme : on compte, et on ouvre au seuil N.
            None => {
                entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
                if entry.consecutive_failures >= max {
                    entry.opened_at = Some(now);
                }
            }
        }
    }

    fn is_available(&self, model_id: &str) -> bool {
        // Faux si `Open` ; vrai si `Closed`, ou `HalfOpen` (delai ecoule).
        !matches!(self.state(model_id), CircuitState::Open)
    }

    fn next_fallback(
        &self,
        task: TaskType,
        required_capabilities: Capabilities,
        required_context: u32,
        candidates: &[ModelRecord],
        failed: &RouterDecision,
        tried: &mut HashSet<String>,
    ) -> Option<RouterDecision> {
        let failed_model = candidates.iter().find(|m| m.id == failed.model_id)?;
        let current_tier_rank = failed_model.tier.rank();

        // Memorise le modele qui vient d'echouer comme "deja essaye".
        tried.insert(failed.model_id.clone());

        // Candidats `Closed` eligibles et non deja essayes.
        let pool_closed: Vec<&ModelRecord> = candidates
            .iter()
            .filter(|m| {
                m.id != failed.model_id
                    && !tried.contains(&m.id)
                    && m.context_length >= required_context
                    && m.capabilities.covers(required_capabilities)
                    && self.state(&m.id) == CircuitState::Closed
            })
            .collect();

        if !pool_closed.is_empty() {
            if let Some(next) = self.select_by_tier(
                task,
                required_context,
                candidates,
                &pool_closed,
                current_tier_rank,
            ) {
                tried.insert(next.model_id.clone());
                return Some(next);
            }
        }

        // Aucun `Closed` : une seule sonde `HalfOpen` par run.
        if tried.contains(HALF_OPEN_PROBE_MARKER) {
            return None;
        }
        let pool_half_open: Vec<&ModelRecord> = candidates
            .iter()
            .filter(|m| {
                m.id != failed.model_id
                    && !tried.contains(&m.id)
                    && m.context_length >= required_context
                    && m.capabilities.covers(required_capabilities)
                    && self.state(&m.id) == CircuitState::HalfOpen
            })
            .collect();
        if pool_half_open.is_empty() {
            return None;
        }
        let next = self.select_by_tier(
            task,
            required_context,
            candidates,
            &pool_half_open,
            current_tier_rank,
        )?;
        tried.insert(next.model_id.clone());
        tried.insert(HALF_OPEN_PROBE_MARKER.to_string());
        Some(next)
    }
}

fn pick_best<S: omega_ports::ScoringPolicy>(
    scoring: &S,
    task: TaskType,
    required_context: u32,
    all: &[ModelRecord],
    subset: &[&ModelRecord],
) -> Option<RouterDecision> {
    subset
        .iter()
        .max_by(|a, b| {
            let sa = scoring.score(task, required_context, all, a);
            let sb = scoring.score(task, required_context, all, b);
            sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|m| RouterDecision {
            model_id: m.id.clone(),
            score: scoring.score(task, required_context, all, m),
            reason: format!("fallback from failed model to {}", m.id),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::CompositeScoringPolicy;
    use omega_types::{Capabilities, ModelTier, QualityAxes};
    use std::sync::Mutex;
    use std::time::Instant;

    #[allow(clippy::too_many_arguments)]
    fn model(
        id: &str,
        tier: ModelTier,
        cost_in: f64,
        cost_out: f64,
        speed: f32,
        ctx: u32,
        caps: Capabilities,
        axes: QualityAxes,
    ) -> ModelRecord {
        ModelRecord {
            id: id.to_string(),
            provider: "test".to_string(),
            context_length: ctx,
            cost_input_per_1m: cost_in,
            cost_output_per_1m: cost_out,
            tokens_per_second: speed,
            ttft_ms: 100,
            axes,
            capabilities: caps,
            tier,
        }
    }

    /// Horloge manuelle : deterministe, sans `sleep`.
    struct TestClock {
        now: Arc<Mutex<Instant>>,
    }

    impl TestClock {
        fn new() -> Self {
            Self {
                now: Arc::new(Mutex::new(Instant::now())),
            }
        }

        fn policy(&self, max_failures: u32) -> TierFallbackPolicy<CompositeScoringPolicy> {
            let handle = Arc::clone(&self.now);
            TierFallbackPolicy::new(CompositeScoringPolicy::new(), max_failures)
                .with_clock(move || *handle.lock().unwrap())
        }

        fn advance(&self, delta: Duration) {
            *self.now.lock().unwrap() += delta;
        }
    }

    fn chat_model(id: &str, tier: ModelTier, chat: u8) -> ModelRecord {
        model(
            id,
            tier,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat,
                ..Default::default()
            },
        )
    }

    #[test]
    fn circuit_breaker_trips_after_repeated_failures() {
        let mut fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        assert!(!fallback.is_available("m1"));
    }

    #[test]
    fn threshold_is_configurable() {
        // N = 2 : le circuit ouvre des le 2e echec consecutif.
        let clock = TestClock::new();
        let mut fallback = clock.policy(2);
        assert_eq!(fallback.max_failures(), 2);
        assert_eq!(fallback.state("m1"), CircuitState::Closed);
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Closed);
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        assert!(!fallback.is_available("m1"));
    }

    #[test]
    fn success_resets_consecutive_failures() {
        // REVUE-W4 M4 : `record_success` remet le compteur a zero.
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert_eq!(fallback.failure_count("m1"), 2);
        fallback.record_success("m1");
        assert_eq!(fallback.failure_count("m1"), 0);
        assert_eq!(fallback.state("m1"), CircuitState::Closed);
        // Apres reset, il faut a nouveau N echecs consecutifs pour ouvrir.
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert!(fallback.is_available("m1"));
        fallback.record_failure("m1");
        assert!(!fallback.is_available("m1"));
    }

    #[test]
    fn open_half_open_closed_full_cycle() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        assert_eq!(fallback.open_timeout(), DEFAULT_OPEN_TIMEOUT);

        // Closed -> Open apres N echecs.
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        assert!(!fallback.is_available("m1"));

        // Open -> HalfOpen apres le delai (60 s par defaut).
        clock.advance(Duration::from_secs(59));
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        assert!(!fallback.is_available("m1"));
        clock.advance(Duration::from_secs(1));
        assert_eq!(fallback.state("m1"), CircuitState::HalfOpen);
        assert!(fallback.is_available("m1"));

        // Sonde reussie -> Closed, compteur remis a zero.
        fallback.record_success("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Closed);
        assert_eq!(fallback.failure_count("m1"), 0);
        assert!(fallback.is_available("m1"));
    }

    #[test]
    fn half_open_probe_failure_reopens_with_fresh_timeout() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        clock.advance(DEFAULT_OPEN_TIMEOUT);
        assert_eq!(fallback.state("m1"), CircuitState::HalfOpen);

        // Sonde echouee -> Open immediat, le delai repart de zero.
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        assert!(!fallback.is_available("m1"));

        clock.advance(DEFAULT_OPEN_TIMEOUT - Duration::from_secs(1));
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        clock.advance(Duration::from_secs(1));
        assert_eq!(fallback.state("m1"), CircuitState::HalfOpen);
    }

    #[test]
    fn open_timeout_is_configurable() {
        let clock = TestClock::new();
        let handle = Arc::clone(&clock.now);
        let mut fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3)
            .with_open_timeout(Duration::from_secs(10))
            .with_clock(move || *handle.lock().unwrap());
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);

        clock.advance(Duration::from_secs(9));
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        clock.advance(Duration::from_secs(1));
        assert_eq!(fallback.state("m1"), CircuitState::HalfOpen);
    }

    #[test]
    fn extra_failures_while_open_do_not_extend_timeout() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        fallback.record_failure("m1");

        // Echec supplementaire avant la fin du delai : l'ouverture n'est pas
        // prolongee, HalfOpen arrive toujours 60 s apres l'ouverture initiale.
        clock.advance(Duration::from_secs(59));
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);
        clock.advance(Duration::from_secs(1));
        assert_eq!(fallback.state("m1"), CircuitState::HalfOpen);
    }

    #[test]
    fn unknown_model_is_closed_and_available() {
        let clock = TestClock::new();
        let fallback = clock.policy(3);
        assert_eq!(fallback.state("unknown"), CircuitState::Closed);
        assert!(fallback.is_available("unknown"));
        assert_eq!(fallback.failure_count("unknown"), 0);
    }

    #[test]
    fn fallback_picks_different_model() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let m2 = model(
            "m2",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                ..Default::default()
            },
        );
        let failed = RouterDecision::new("m1", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[m1, m2],
                &failed,
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "m2");
    }

    #[test]
    fn fallback_escalates_tier_when_same_tier_exhausted() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let standard = model(
            "standard",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let elite = model(
            "elite",
            ModelTier::Elite,
            5.0,
            15.0,
            50.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 90,
                ..Default::default()
            },
        );
        let failed = RouterDecision::new("standard", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[standard, elite],
                &failed,
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "elite");
    }

    #[test]
    fn fallback_avoids_ping_pong() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let m2 = model(
            "m2",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                ..Default::default()
            },
        );

        // m1 echoue -> fallback m2.
        let mut tried = HashSet::new();
        let step0 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[m1.clone(), m2.clone()],
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut tried,
            )
            .unwrap();
        assert_eq!(step0.model_id, "m2");

        // m2 echoue -> on ne doit PAS reselectionner m1 (deja essaye).
        let step1 = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &[m1, m2],
            &RouterDecision::new("m2", 0.5, "fallback"),
            &mut tried,
        );
        assert!(step1.is_none(), "should exhaust candidates, not ping-pong");
    }

    #[test]
    fn fallback_escalates_to_self_hosted_after_exhaustion() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let standard = model(
            "standard",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let self_hosted = model(
            "self-hosted",
            ModelTier::SelfHosted,
            0.0,
            0.0,
            80.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 65,
                ..Default::default()
            },
        );

        let failed = RouterDecision::new("standard", 0.5, "initial");
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &[standard, self_hosted],
                &failed,
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "self-hosted");
    }

    #[test]
    fn fallback_returns_none_when_failed_model_unknown() {
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes::default(),
        );
        let next = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &[m1],
            &RouterDecision::new("unknown", 0.5, "initial"),
            &mut HashSet::new(),
        );
        assert!(next.is_none());
    }

    #[test]
    fn tried_state_is_per_run_not_shared_between_requests() {
        // REVUE-W4 N2 : l'ensemble "deja essaye" ne doit pas fuir d'une requete
        // a l'autre. Meme instance, deux runs avec un `tried` frais => meme
        // fallback complet depuis zero.
        let fallback = TierFallbackPolicy::new(CompositeScoringPolicy::new(), 3);
        let m1 = model(
            "m1",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 70,
                ..Default::default()
            },
        );
        let m2 = model(
            "m2",
            ModelTier::Standard,
            1.0,
            3.0,
            100.0,
            128_000,
            Capabilities::default(),
            QualityAxes {
                chat: 80,
                ..Default::default()
            },
        );
        let candidates = [m1, m2];

        // Run 1 : m1 echoue -> m2.
        let run1 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(run1.model_id, "m2");

        // Run 2 : `tried` reinitialise => le fallback repart de zero (m2 a nouveau).
        let run2 = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(run2.model_id, "m2");
    }

    #[test]
    fn fallback_excludes_open_models() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        let m1 = chat_model("m1", ModelTier::Standard, 70);
        let m2 = chat_model("m2", ModelTier::Standard, 80);
        let candidates = [m1, m2];

        // m1 ouvre son circuit : il n'est plus propose.
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        fallback.record_failure("m1");
        assert_eq!(fallback.state("m1"), CircuitState::Open);

        // m2 echoue -> le fallback saute m1 (Open) : aucun candidat Closed.
        // m1 n'est pas encore HalfOpen (delai non ecoule) => None.
        let none = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &candidates,
            &RouterDecision::new("m2", 0.5, "initial"),
            &mut HashSet::new(),
        );
        assert!(none.is_none(), "open model must be excluded");
    }

    #[test]
    fn fallback_prefers_closed_over_higher_scoring_half_open() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        let failed = chat_model("failed", ModelTier::Standard, 70);
        let closed = chat_model("closed", ModelTier::Standard, 60);
        // Score superieur mais circuit ouvert.
        let half_open = chat_model("half", ModelTier::Standard, 95);
        let candidates = [failed, closed, half_open];

        for _ in 0..3 {
            fallback.record_failure("half");
        }
        clock.advance(DEFAULT_OPEN_TIMEOUT);
        assert_eq!(fallback.state("half"), CircuitState::HalfOpen);

        // Le `Closed` gagne meme avec un score inferieur.
        let next = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("failed", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(next.model_id, "closed");
    }

    #[test]
    fn half_open_probe_is_limited_to_one_per_run() {
        let clock = TestClock::new();
        let mut fallback = clock.policy(3);
        let m1 = chat_model("m1", ModelTier::Standard, 70);
        let m2 = chat_model("m2", ModelTier::Standard, 80);
        let candidates = [m1, m2];

        // Les deux circuits sont ouverts puis passent HalfOpen.
        for _ in 0..3 {
            fallback.record_failure("m1");
            fallback.record_failure("m2");
        }
        clock.advance(DEFAULT_OPEN_TIMEOUT);
        assert_eq!(fallback.state("m2"), CircuitState::HalfOpen);

        // Run : premiere sonde HalfOpen autorisee...
        let mut tried = HashSet::new();
        let probe = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut tried,
            )
            .unwrap();
        assert_eq!(probe.model_id, "m2");
        assert!(
            tried.contains(HALF_OPEN_PROBE_MARKER),
            "probe must consume the per-run marker"
        );

        // ...mais pas de seconde sonde dans le meme run.
        let second = fallback.next_fallback(
            TaskType::Chat,
            Capabilities::default(),
            0,
            &candidates,
            &RouterDecision::new("m2", 0.5, "probe"),
            &mut tried,
        );
        assert!(
            second.is_none(),
            "only one half-open probe per run is allowed"
        );

        // Run suivant (`tried` frais) : la sonde est a nouveau autorisee.
        let retry = fallback
            .next_fallback(
                TaskType::Chat,
                Capabilities::default(),
                0,
                &candidates,
                &RouterDecision::new("m1", 0.5, "initial"),
                &mut HashSet::new(),
            )
            .unwrap();
        assert_eq!(retry.model_id, "m2");
    }
}
