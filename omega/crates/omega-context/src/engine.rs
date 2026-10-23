//! Moteur de contexte **par defaut** (`name = "default"`, `owns_compaction = false`).

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use omega_ports::ContextEngine;
use omega_types::{Message, OmegaError, OmegaResult, Role};

use crate::budget::ContextBudget;
use crate::injection::{ContextInjector, InjectionTrace, MemoryInjection};
use crate::token::TokenEstimator;

/// Rapport d'assemblage : journalise ce qui a ete coupe (payload de test).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AssembleReport {
    pub kept: usize,
    pub dropped: usize,
    pub estimated_tokens: usize,
    /// Vrai si le resultat depasse le budget (systeme seul plus grand que le plafond).
    ///
    /// Le prefixe systeme est toujours preserve (primaute du systeme, cf. W6 §4.6) ;
    /// ce drapeau rend le depassement **explicite** au lieu de le laisser silencieux.
    pub over_budget: bool,
    /// Apercus des messages non-systeme coupes (contenu tronque), dans l'ordre.
    pub dropped_previews: Vec<String>,
}

/// Nombre maximum de caracteres d'un apercu de message coupe.
const PREVIEW_MAX_CHARS: usize = 40;

fn preview(text: &str) -> String {
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(PREVIEW_MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

/// Moteur par defaut : assemble systeme + non-systeme sous budget, en tronquant les
/// plus anciens messages **non-systeme**. Le prefixe systeme est toujours preserve.
pub struct DefaultContextEngine {
    budget: ContextBudget,
    estimator: TokenEstimator,
    last_report: Mutex<AssembleReport>,
    /// Injecteur optionnel de memoire (consulte avant chaque assemble).
    injector: Option<Arc<dyn ContextInjector>>,
    /// Derniere decision d'injection (trace vide = absence injection).
    last_injection: Mutex<InjectionTrace>,
    /// Journal des decisions d'injection, dans l'ordre des consultes.
    injection_journal: Mutex<Vec<InjectionTrace>>,
}

impl DefaultContextEngine {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            budget: ContextBudget::new(max_tokens),
            estimator: TokenEstimator::new(),
            last_report: Mutex::new(AssembleReport::default()),
            injector: None,
            last_injection: Mutex::new(InjectionTrace::default()),
            injection_journal: Mutex::new(Vec::new()),
        }
    }

    /// Moteur avec injection memoire : les faits pertinents sont consultes sur
    /// le `MemoryStore` avant chaque assemble, sous un budget explicite.
    pub fn with_memory(store: Arc<dyn omega_ports::MemoryStore>, max_tokens: usize) -> Self {
        let mut e = Self::new(max_tokens);
        e.injector = Some(Arc::new(MemoryInjection::new(store)));
        e
    }

    /// Derniere decision d'injection (trace vide = rien n'a ete injecte).
    pub fn last_injection(&self) -> InjectionTrace {
        self.last_injection
            .lock()
            .expect("trace empoisonnee")
            .clone()
    }

    /// Journal complet des decisions d'injection de ce moteur, dans l'ordre.
    pub fn injection_journal(&self) -> Vec<InjectionTrace> {
        self.injection_journal
            .lock()
            .expect("journal empoisonne")
            .clone()
    }

    /// Plafond du budget de contexte du moteur.
    pub fn max_tokens(&self) -> usize {
        self.budget.max_tokens()
    }

    /// Dernier rapport d'assemblage (coupes journalisees).
    pub fn last_report(&self) -> AssembleReport {
        self.last_report.lock().expect("rapport empoisonne").clone()
    }

    /// Consulte l'injecteur de memoire (si branche) puis assemble le resultat.
    ///
    /// L'injection s'applique a la tranche recue par [`Self::assemble_sync`] :
    /// elle est inseree apres le prefixe systeme par l'injecteur, la coupe
    /// budgetaire ensuite s'effectue sur le flux enrichi sans jamais couper le
    /// systeme. En l'absence d'injecteur, `assemble_with_injection` est un
    /// passthrough vers `assemble_sync`.
    pub async fn assemble_with_injection(
        &self,
        messages: &[Message],
    ) -> OmegaResult<(Vec<Message>, AssembleReport)> {
        let (flux, trace) = match &self.injector {
            None => (messages.to_vec(), InjectionTrace::default()),
            Some(injector) => {
                // Budget explicite : l'enveloppe totale du moteur plafonne
                // l'injection (la coupe garde la main apres, systeme premier).
                let budget = self.budget.max_tokens();
                injector.inject(messages.to_vec(), budget).await?
            }
        };
        *self.last_injection.lock().expect("trace empoisonnee") = trace.clone();
        self.injection_journal
            .lock()
            .expect("journal empoisonne")
            .push(trace);
        let (result, report) = self.assemble_sync(&flux);
        Ok((result, report))
    }

    /// Assemble en mode synchrone (coeur deterministe, testable sans runtime).
    ///
    /// Le prefixe systeme est toujours preserve. Si le systeme seul depasse deja le
    /// budget, le depassement est journalise via `AssembleReport.over_budget` (le
    /// resultat n'est jamais coupe au-dela du systeme, qui est intouchable).
    ///
    /// Retourne le resultat **et** son rapport ensemble : l'appelant async utilise le
    /// rapport retourne (pas une relecture du `Mutex` partage), ce qui rend la decision
    /// `Ok`/`BudgetExceeded` insensible aux appels concurrents (REVIEW-W6 N1).
    pub fn assemble_sync(&self, messages: &[Message]) -> (Vec<Message>, AssembleReport) {
        let total = self.estimator.estimate_messages(messages);
        if total <= self.budget.max_tokens() {
            let report = AssembleReport {
                kept: messages.len(),
                dropped: 0,
                estimated_tokens: total,
                over_budget: false,
                dropped_previews: Vec::new(),
            };
            *self.last_report.lock().expect("rapport empoisonne") = report.clone();
            return (messages.to_vec(), report);
        }

        let system: Vec<Message> = messages
            .iter()
            .take_while(|m| m.role == Role::System)
            .cloned()
            .collect();
        let non_system = &messages[system.len()..];

        // Suffixe maximal des messages les plus recents (on tronque les plus anciens).
        let mut kept: Vec<Message> = Vec::new();
        let mut kept_tokens = self.estimator.estimate_messages(&system);
        for msg in non_system.iter().rev() {
            let t = self.estimator.estimate(&msg.content);
            if kept_tokens + t <= self.budget.max_tokens() {
                kept_tokens += t;
                kept.push(msg.clone());
            } else {
                break;
            }
        }
        kept.reverse();
        let dropped = non_system.len() - kept.len();

        let dropped_previews: Vec<String> = non_system[..dropped]
            .iter()
            .map(|m| preview(&m.content))
            .collect();

        let mut result = system;
        result.extend(kept);

        let estimated_tokens = self.estimator.estimate_messages(&result);
        let report = AssembleReport {
            kept: result.len(),
            dropped,
            estimated_tokens,
            over_budget: estimated_tokens > self.budget.max_tokens(),
            dropped_previews,
        };
        *self.last_report.lock().expect("rapport empoisonne") = report.clone();
        (result, report)
    }
}

#[async_trait]
impl ContextEngine for DefaultContextEngine {
    fn name(&self) -> &str {
        "default"
    }

    fn owns_compaction(&self) -> bool {
        false
    }

    fn max_tokens(&self) -> usize {
        self.budget.max_tokens()
    }

    async fn assemble(&self, messages: Vec<Message>) -> OmegaResult<Vec<Message>> {
        let (result, report) = self.assemble_with_injection(&messages).await?;
        if report.over_budget {
            // Le systeme seul depasse le budget : signal type, jamais de resultat
            // hors contrat silencieux (REVIEW-W6 M1).
            return Err(OmegaError::BudgetExceeded {
                max: self.max_tokens(),
                used: report.estimated_tokens,
                additional: 0,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_types::Role;
    use std::sync::Arc;

    fn sys(content: &str) -> Message {
        Message::new(Role::System, content)
    }

    fn user(content: &str) -> Message {
        Message::new(Role::User, content)
    }

    #[test]
    fn under_budget_is_passthrough() {
        let e = DefaultContextEngine::new(1000);
        let msgs = vec![sys("S"), user("hello")];
        let (out, _) = e.assemble_sync(&msgs);
        assert_eq!(out, msgs);
        assert_eq!(e.last_report().dropped, 0);
    }

    #[test]
    fn assemble_respects_budget_and_drops_oldest_non_system() {
        // system 2 tokens ; u1..u3 2 tokens chacun -> total 8, budget 6.
        let e = DefaultContextEngine::new(6);
        let msgs = vec![
            sys("SSSSSSSS"),
            user("11111111"),
            user("22222222"),
            user("33333333"),
        ];
        let (out, _) = e.assemble_sync(&msgs);

        // systeme + les 2 plus recents uniquement.
        assert_eq!(
            out,
            vec![sys("SSSSSSSS"), user("22222222"), user("33333333")]
        );
        let report = e.last_report();
        assert_eq!(report.dropped, 1);
        assert_eq!(report.kept, 3);
        assert_eq!(report.dropped_previews, vec!["11111111".to_string()]);
        assert!(report.estimated_tokens <= 6);
    }

    #[test]
    fn assemble_preserves_system_even_when_budget_tight() {
        // system 2 tokens ; budget 3 : aucun message non-systeme ne rentre.
        let e = DefaultContextEngine::new(3);
        let msgs = vec![sys("SSSSSSSS"), user("11111111"), user("22222222")];
        let (out, _) = e.assemble_sync(&msgs);

        assert_eq!(out, vec![sys("SSSSSSSS")]);
        let report = e.last_report();
        assert_eq!(report.dropped, 2);
        assert!(!report.over_budget);
    }

    #[test]
    fn assemble_system_alone_over_budget_is_signaled() {
        // Non-regression REVIEW-W6 M1 : systeme seul (10 tokens) > budget (5).
        // Le systeme est preserve (primaute), mais le depassement est explicite.
        let e = DefaultContextEngine::new(5);
        let msgs = vec![sys(&"S".repeat(40))]; // 40 caracteres -> 10 tokens
        let (out, _) = e.assemble_sync(&msgs);

        assert_eq!(out, msgs);
        let report = e.last_report();
        assert!(report.over_budget);
        assert_eq!(report.estimated_tokens, 10);
        assert_eq!(report.dropped, 0);
    }

    #[tokio::test]
    async fn assemble_returns_typed_error_when_system_alone_over_budget() {
        // Le port async expose le depassement en `BudgetExceeded` (jamais de `Ok`
        // hors budget).
        let e = DefaultContextEngine::new(5);
        let msgs = vec![sys(&"S".repeat(40))];
        let err = e.assemble(msgs).await.unwrap_err();
        assert!(matches!(
            err,
            OmegaError::BudgetExceeded {
                max: 5,
                used: 10,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn concurrent_assembles_use_own_report_not_shared() {
        // Non-regression REVIEW-W6 N1 : deux `assemble` concurrents sur le meme moteur
        // prennent chacun la decision coherente avec LEUR entree (pas de faux
        // negatif/positif par relecture du rapport partage).
        use std::sync::Arc;
        let engine = Arc::new(DefaultContextEngine::new(5));
        let over: Vec<Message> = vec![sys(&"S".repeat(40))]; // 10 tokens > 5
        let under: Vec<Message> = vec![user("hi")];
        for _ in 0..50 {
            let (e1, e2) = (engine.clone(), engine.clone());
            let (o1, o2) = (over.clone(), under.clone());
            let (r1, r2) = tokio::join!(e1.assemble(o1), e2.assemble(o2));
            assert!(matches!(r1, Err(OmegaError::BudgetExceeded { .. })));
            assert!(r2.is_ok());
        }
    }

    // ═══ T2 (commit 1 — rouge) : injection memoire dans le moteur ═══
    // Double MemoryStore minimal : restitue les faits dont un mot-cle de la
    // requete apparait dans le texte (memes regles de pertinence que le module).
    struct StaticMemory {
        facts: Vec<String>,
    }

    #[async_trait]
    impl omega_ports::MemoryStore for StaticMemory {
        fn name(&self) -> &str {
            "static-test"
        }
        async fn recall(&self, query: &str, limit: usize) -> OmegaResult<Vec<String>> {
            let q = query.to_lowercase();
            let hits: Vec<String> = self
                .facts
                .iter()
                .filter(|f| {
                    q.split_whitespace()
                        .filter(|w| w.chars().count() >= 4)
                        .any(|w| f.to_lowercase().contains(w))
                })
                .take(limit)
                .cloned()
                .collect();
            Ok(hits)
        }
        async fn remember(&self, _fact: &str) -> OmegaResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn assemble_injecte_la_memoire_relevante_avant_assemble_sync() {
        // Un fait pertinent doit arriver au modele, sans changer la coupe
        // budgetaire (systeme preserve, plus anciens non-systeme d'abord).
        let store: Arc<dyn omega_ports::MemoryStore> = Arc::new(StaticMemory {
            facts: vec!["OSAKA: le run budget se lit via RunBudget.usage()".to_string()],
        });
        let e = DefaultContextEngine::with_memory(store, 60);
        let msgs = vec![user("explique le run budget d'OSAKA stp")];
        let (out, report) = e.assemble_with_injection(&msgs).await.unwrap();

        // Le fait pertinent est arrive au modele, insere apres le prefixe systeme.
        assert_eq!(out[0].role, Role::System);
        assert!(out[0].content.contains("RunBudget.usage()"));
        assert_eq!(out[1].role, Role::User);
        assert_eq!(report.kept, 2);
        assert_eq!(e.last_injection().lines.len(), 1);
    }

    #[tokio::test]
    async fn assemble_sans_injecteur_passthrough() {
        // Sans injecteur, assemble_with_injection = assemble_sync (passthrough).
        let e = DefaultContextEngine::new(100);
        let msgs = vec![user("hello")];
        let (out, _) = e.assemble_with_injection(&msgs).await.unwrap();
        assert_eq!(out, msgs);
        assert!(e.last_injection().is_empty());
        assert_eq!(e.injection_journal().len(), 1); // decision passthrough journalisee
    }

    #[tokio::test]
    async fn injection_journalisé_à_chaque_consulte() {
        // Chaque consulte du MemoryStore laisse une trace dans le journal du
        // moteur : deux appels -> deux entrees, injectante ou pas.
        let store: Arc<dyn omega_ports::MemoryStore> = Arc::new(StaticMemory {
            facts: vec!["OSAKA: le run budget se lit via RunBudget.usage()".to_string()],
        });
        let e = DefaultContextEngine::with_memory(store, 60);
        let _ = e
            .assemble_with_injection(&[user("explique le run budget d'OSAKA")])
            .await
            .unwrap();
        let _ = e
            .assemble_with_injection(&[user("questions sans rapport")])
            .await
            .unwrap();

        let journal = e.injection_journal();
        assert_eq!(journal.len(), 2);
        assert_eq!(journal[0].lines.len(), 1); // injecte
        assert!(journal[1].is_empty()); // pertinent absente -> decision vide
        assert_eq!(e.last_injection().lines.len(), 0);
    }

    #[tokio::test]
    async fn trait_contract_name_and_ownership() {
        let e = DefaultContextEngine::new(100);
        let engine: Arc<dyn ContextEngine> = Arc::new(e);
        assert_eq!(engine.name(), "default");
        assert!(!engine.owns_compaction());

        let out = engine.assemble(vec![user("hi")]).await.unwrap();
        assert_eq!(out, vec![user("hi")]);
    }

    #[test]
    fn assemble_sync_n_injecte_pas_depuis_memory_store() {
        let store: Arc<dyn omega_ports::MemoryStore> = Arc::new(StaticMemory {
            facts: vec!["CANARY injection marker value 42".to_string()],
        });
        let e = DefaultContextEngine::with_memory(store, 100);
        let msgs = vec![user("tell me about canary please")];
        let (out, _) = e.assemble_sync(&msgs);
        assert!(
            !out.iter().any(|m| m.content.contains("CANARY")),
            "assemble_sync ne doit pas injecter (injection reservee a assemble_with_injection)"
        );
    }
}
