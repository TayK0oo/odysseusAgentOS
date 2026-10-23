//! Injection memoire (T2) : consulte un [`MemoryStore`] **avant** assemble et
//! n'injecte que les faits qui **changent la reponse**, sous un budget explicite
//! de tokens. Deterministe, aucun appel LLM (RULES-OMEGA §3.2).
//!
//! Regles d'inclusion d'un fait (toutes obligatoires) :
//! 1. **Pertinence** : le texte contient au moins un mot-cle de la requete
//!    (mot >= 4 caractères, comparaison insensible a la casse) ;
//! 2. **Changement de reponse** : le fait n'apparait deja dans aucun message
//!    (dedup exact, sinon l'injection ne change rien et ne vaut pas ses tokens) ;
//! 3. **Budget** : son cout `ceil(chars / 4)` tient dans l'enveloppe restante.
//!
//! Chaque injection est journalisee dans [`InjectionTrace`].

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use omega_ports::MemoryStore;
use omega_types::{Message, OmegaResult, Role};

use crate::token::TokenEstimator;

/// Une ligne du journal d'injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectionLine {
    /// Apercu du fait injecte (40 premiers caracteres).
    pub fact: String,
    /// Cout estime du fait, en tokens.
    pub tokens: usize,
}

/// Trace (journal) d'une decision d'injection : vide = aucune injection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InjectionTrace {
    pub lines: Vec<InjectionLine>,
    /// Cout total estime de tout ce qui a ete injecte.
    pub tokens_used: usize,
    /// Enveloppe explicite (budget) passe a la decision.
    pub budget: usize,
}

impl InjectionTrace {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Port d'injection de contexte. Regime : *ensemble*.
#[async_trait]
pub trait ContextInjector: Send + Sync {
    fn name(&self) -> &str;
    /// Consulte la memoire pour `messages`, retourne les messages enrichis et
    /// le journal des injections. Ne tronque jamais (la coupe reste au moteur).
    async fn inject(
        &self,
        messages: Vec<Message>,
        budget_tokens: usize,
    ) -> OmegaResult<(Vec<Message>, InjectionTrace)>;
}

/// Injecteur par defaut : relit le dernier message utilisateur sur le
/// [`MemoryStore`], filtre par pertinence + dedup + budget, puis insere les
/// faits retenus en **un** message `System` unique juste apres le prefixe
/// systeme existant (jamais d'ordre de coupe perturbe).
pub struct MemoryInjection {
    store: Arc<dyn MemoryStore>,
    recall_limit: usize,
    journal: Mutex<Vec<InjectionTrace>>,
}

impl MemoryInjection {
    pub fn new(store: Arc<dyn MemoryStore>) -> Self {
        Self {
            store,
            recall_limit: 8,
            journal: Mutex::new(Vec::new()),
        }
    }

    /// Journal des injections effectuees, une trace par appel `inject`.
    pub fn journal(&self) -> Vec<InjectionTrace> {
        self.journal.lock().expect("journal empoisonne").clone()
    }

    /// Cles de pertinence d'une requete : mots de >= 4 caracteres, minuscules.
    fn keywords(query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() >= 4)
            .map(|w| w.to_string())
            .collect()
    }
}

#[async_trait]
impl ContextInjector for MemoryInjection {
    fn name(&self) -> &str {
        "memory-injection"
    }

    async fn inject(
        &self,
        messages: Vec<Message>,
        budget_tokens: usize,
    ) -> OmegaResult<(Vec<Message>, InjectionTrace)> {
        let query = messages
            .iter()
            .rev()
            .find(|m| m.role == Role::User)
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let keywords = Self::keywords(query);

        let mut selected: Vec<String> = Vec::new();
        let mut used = 0usize;
        if !keywords.is_empty() && budget_tokens > 0 {
            let recalled = self.store.recall(query, self.recall_limit).await?;
            for fact in recalled {
                // 1) pertinence
                if !keywords.iter().any(|w| fact.to_lowercase().contains(w)) {
                    continue;
                }
                // 2) change la reponse ? (si deja present, injecter ne changerait rien)
                let deja_compris = messages.iter().any(|m| m.content.contains(&fact));
                if deja_compris {
                    continue;
                }
                // 3) budget explicite
                let tokens = TokenEstimator::new().estimate(&fact);
                if used + tokens > budget_tokens {
                    break;
                }
                used += tokens;
                selected.push(fact);
            }
        }

        let trace = InjectionTrace {
            lines: selected
                .iter()
                .map(|f| InjectionLine {
                    fact: f.chars().take(40).collect(),
                    tokens: TokenEstimator::new().estimate(f),
                })
                .collect(),
            tokens_used: used,
            budget: budget_tokens,
        };
        self.journal.lock().expect("journal empoisonne").push(trace.clone());

        if selected.is_empty() {
            return Ok((messages, trace));
        }

        let bloc = {
            let mut b = String::from("memoire:\n");
            for f in &selected {
                b.push_str("- ");
                b.push_str(f);
                b.push('\n');
            }
            b
        };

        // Insertion apres le prefixe systeme existant, comme message System.
        let prefixe_systeme = messages
            .iter()
            .take_while(|m| m.role == Role::System)
            .count();
        let mut out: Vec<Message> = Vec::with_capacity(messages.len() + 1);
        out.extend(messages[..prefixe_systeme].iter().cloned());
        out.push(Message::new(Role::System, bloc));
        out.extend(messages[prefixe_systeme..].iter().cloned());
        Ok((out, trace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    /// Double MemoryStore : memoire statique en memoire, `recall` filtre les
    /// faits contenant un mot de la requete (meme heuristique que le module).
    struct StaticMemory {
        facts: Vec<String>,
    }

    #[async_trait]
    impl MemoryStore for StaticMemory {
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

    fn mem(facts: &[&str]) -> MemoryInjection {
        MemoryInjection::new(Arc::new(StaticMemory {
            facts: facts.iter().map(|s| s.to_string()).collect(),
        }))
    }

    fn user_msg(s: &str) -> Message {
        Message::new(Role::User, s)
    }

    // (a) pas d'injection si non pertinent
    #[tokio::test]
    async fn injection_a_aucune_injection_si_non_pertinent() {
        let inj = mem(&["Paris compte deux millions d'habitants"]);
        let msgs = vec![user_msg("how do I cook pasta properly")];
        let (out, trace) = inj.inject(msgs, 50).await.unwrap();
        assert_eq!(out.len(), 1);
        assert!(trace.is_empty());
        assert_eq!(trace.tokens_used, 0);
        assert_eq!(inj.journal().len(), 1); // la decision est journalisee, vide
    }

    // (b) injection + trace si pertinent
    #[tokio::test]
    async fn injection_b_injecte_et_trace_si_pertinent() {
        let inj = mem(&["OSAKA: le run budget se lit via RunBudget.usage()"]);
        let msgs = vec![user_msg("explique le run budget d'OSAKA stp")];
        let (out, trace) = inj.inject(msgs, 50).await.unwrap();
        assert_eq!(out.len(), 2);
        assert!(out[0].content.contains("RunBudget.usage()"));
        assert_eq!(out[0].role, Role::System); // insere avant le message user
        assert_eq!(trace.lines.len(), 1);
        assert!(trace.lines[0].fact.contains("OSAKA"));
        assert!(trace.lines[0].tokens > 0);
        assert_eq!(trace.tokens_used, trace.lines[0].tokens);
        // deja compris : pas d'injection, la reponse n'en changerait pas
        let deja = vec![
            user_msg("explique le run budget d'OSAKA stp"),
            Message::new(Role::Assistant, "cf RunBudget.usage()"),
        ];
        let (out2, trace2) = inj
            .inject(deja, 50)
            .await
            .expect("dedup");
        assert_eq!(out2.len(), 2);
        assert!(trace2.is_empty());
    }

    // (c) budget jamais depasse
    #[tokio::test]
    async fn injection_c_budget_jamais_depasses() {
        // 6 faits ~4 tokens chacun ; enveloppe 10 -> au plus 2 faits.
        let facts: Vec<String> = (1..=6)
            .map(|i| format!("OSAKA fait numero {i} sur le budget"))
            .collect();
        let inj = MemoryInjection::new(Arc::new(StaticMemory { facts }));
        let msgs = vec![user_msg("resume les faits OSAKA stp")];
        let budget = 10;
        let (out, trace) = inj.inject(msgs, budget).await.unwrap();
        assert!(trace.tokens_used <= budget);
        assert_eq!(trace.budget, budget);
        assert!(trace.lines.len() < 6);
        // chaque ligne est facturable et la somme couvre le bloc injecte
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].role, Role::System);
        for l in &trace.lines {
            assert!(l.tokens <= budget);
        }
        assert!(trace.tokens_used > 0);
    }
}
