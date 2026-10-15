//! File d'approbations humaines à **usage unique** (incrément 2).
//!
//! Cycle de vie : `request(commande, motif)` crée une demande `Pending` et rend
//! un [`ApprovalId`] ; un opérateur (ou un test) appelle `approve(id)` ou
//! `deny(id)`. Devant une décision `Prompt`, le backend consomme **une**
//! approbation `Approved` pour la **commande exacte** ([`ApprovalQueue::consume_for`])
//! puis exécute ; l'approbation est supprimée (usage unique) : le second appel
//! avec la même commande re-demande une approbation.
//!
//! `ApprovalQueue` est partageable entre threads (`&self` + `Mutex` interne) :
//! le backend la détient via `Arc<ApprovalQueue>`.

use std::collections::HashMap;
use std::sync::Mutex;

/// Identifiant d'une demande d'approbation (monotone, démarrant à 1).
pub type ApprovalId = u64;

/// État d'une demande d'approbation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalState {
    /// En attente de décision humaine.
    Pending,
    /// Approuvée, en attente de consommation (usage unique).
    Approved,
    /// Refusée (ne sera jamais consommée).
    Denied,
}

/// Demande d'approbation (vue clonée, état figé à l'appel).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    pub id: ApprovalId,
    pub command: String,
    pub reason: String,
    pub state: ApprovalState,
}

#[derive(Debug)]
struct Inner {
    next_id: ApprovalId,
    entries: HashMap<ApprovalId, Approval>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            next_id: 1,
            entries: HashMap::new(),
        }
    }
}

/// File d'approbations à usage unique, partageable (`Arc`).
#[derive(Debug, Default)]
pub struct ApprovalQueue {
    inner: Mutex<Inner>,
}

impl ApprovalQueue {
    /// Crée une file vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enregistre une demande `Pending` pour la commande exacte et rend son id.
    pub fn request(&self, command: impl Into<String>, reason: impl Into<String>) -> ApprovalId {
        let mut inner = self.inner.lock().expect("approval queue poisoned");
        let id = inner.next_id;
        inner.next_id += 1;
        inner.entries.insert(
            id,
            Approval {
                id,
                command: command.into(),
                reason: reason.into(),
                state: ApprovalState::Pending,
            },
        );
        id
    }

    /// Approuve la demande `id` (rend `false` si inconnue).
    pub fn approve(&self, id: ApprovalId) -> bool {
        let mut inner = self.inner.lock().expect("approval queue poisoned");
        match inner.entries.get_mut(&id) {
            Some(entry) => {
                entry.state = ApprovalState::Approved;
                true
            }
            None => false,
        }
    }

    /// Refuse la demande `id` (rend `false` si inconnue).
    pub fn deny(&self, id: ApprovalId) -> bool {
        let mut inner = self.inner.lock().expect("approval queue poisoned");
        match inner.entries.get_mut(&id) {
            Some(entry) => {
                entry.state = ApprovalState::Denied;
                true
            }
            None => false,
        }
    }

    /// Liste les demandes encore `Pending`, triées par id croissant.
    pub fn pending(&self) -> Vec<Approval> {
        let inner = self.inner.lock().expect("approval queue poisoned");
        let mut out: Vec<Approval> = inner
            .entries
            .values()
            .filter(|e| e.state == ApprovalState::Pending)
            .cloned()
            .collect();
        out.sort_by_key(|e| e.id);
        out
    }

    /// Supprime les demandes `Denied` (historique inutile : un refus n'est jamais
    /// consommé). Sans appel, les refus s'accumulent — fuite mémoire sur runs longs.
    /// Rend le nombre d'entrées supprimées.
    pub fn prune_denied(&self) -> usize {
        let mut inner = self.inner.lock().expect("approval queue poisoned");
        let before = inner.entries.len();
        inner
            .entries
            .retain(|_, e| e.state != ApprovalState::Denied);
        before - inner.entries.len()
    }

    /// Consomme (supprime, usage unique) la plus ancienne approbation `Approved`
    /// pour la **commande exacte**. Rend `true` si une approbation a été
    /// consommée, `false` sinon (aucune, refusée, commande différente ou déjà
    /// consommée).
    pub fn consume_for(&self, command: &str) -> bool {
        let mut inner = self.inner.lock().expect("approval queue poisoned");
        let target = inner
            .entries
            .values()
            .filter(|e| e.state == ApprovalState::Approved && e.command == command)
            .map(|e| e.id)
            .min();
        match target {
            Some(id) => {
                inner.entries.remove(&id);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_approve_pending_flow() {
        let q = ApprovalQueue::new();
        let a = q.request("sudo ls", "élévation");
        let b = q.request("sudo rm", "destructeur ?");
        assert_eq!(q.pending().len(), 2);
        assert!(q.approve(a));
        let pending = q.pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, b);
        assert_eq!(pending[0].command, "sudo rm");
    }

    #[test]
    fn approval_is_single_use() {
        let q = ApprovalQueue::new();
        let id = q.request("sudo ls", "élévation");
        assert!(q.approve(id));
        assert!(q.consume_for("sudo ls"));
        // Deuxième utilisation : déjà consommée → refus.
        assert!(!q.consume_for("sudo ls"));
    }

    #[test]
    fn deny_is_never_consumed() {
        let q = ApprovalQueue::new();
        let id = q.request("sudo ls", "élévation");
        assert!(q.deny(id));
        assert!(!q.consume_for("sudo ls"));
        assert!(q.pending().is_empty());
    }

    #[test]
    fn consume_matches_exact_command_only() {
        let q = ApprovalQueue::new();
        let id = q.request("sudo ls", "élévation");
        assert!(q.approve(id));
        assert!(!q.consume_for("sudo ls /tmp"));
        assert!(!q.consume_for("SUDO LS"));
        assert!(q.consume_for("sudo ls"));
    }

    #[test]
    fn unknown_ids_return_false() {
        let q = ApprovalQueue::new();
        assert!(!q.approve(999));
        assert!(!q.deny(999));
        assert!(!q.consume_for("sudo ls"));
    }

    #[test]
    fn prune_denied_reaps_refusals_only() {
        let q = ApprovalQueue::new();
        let a = q.request("sudo a", "x");
        let b = q.request("sudo b", "x");
        assert!(q.deny(a));
        assert!(q.approve(b));
        assert_eq!(q.prune_denied(), 1);
        assert!(q.consume_for("sudo b"));
        assert_eq!(q.prune_denied(), 0);
    }
}
