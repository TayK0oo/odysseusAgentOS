//! Erreurs de domaine Omega.
//!
//! Aucune dependance sur un crate `omega-*` autre que celui-ci.

use crate::Capability;

/// Erreur de domaine, agnostique du fournisseur et de l'adaptateur.
#[derive(Debug, thiserror::Error)]
pub enum OmegaError {
    /// Aucune implementation active pour cette capacite.
    #[error("capacite {0:?} : aucune implementation active")]
    NoImplementation(Capability),

    /// Une capacite en regime `Slot` a deja un proprietaire actif.
    #[error("capacite {0:?} : slot deja detenu par '{1}'")]
    SlotConflict(Capability, String),

    /// Identifiant deja enregistre pour cette capacite.
    #[error("capacite {0:?} : identifiant '{1}' deja enregistre")]
    DuplicateId(Capability, String),

    /// Un plugin a echoue en cours d'execution (doit etre mis en quarantaine, pas fatal).
    #[error("plugin '{0}' en echec : {1}")]
    PluginFailed(String, String),

    /// Annulation cooperative demandee.
    #[error("annulation demandee")]
    Cancelled,

    /// Erreur de configuration (couche, schema, valeur).
    #[error("erreur de configuration : {0}")]
    Config(String),

    /// Depassement du budget de contexte (estimation > plafond).
    #[error("budget de contexte depasse : {used} + {additional} > {max}")]
    BudgetExceeded {
        max: usize,
        used: usize,
        additional: usize,
    },

    /// Erreur d'entree/sortie (journal durable, fichiers, store).
    #[error("erreur d'entree/sortie : {0}")]
    Io(String),

    /// Le type concret demande ne correspond pas a l'instance enregistree.
    #[error("capacite {0:?} : type demande incompatible avec l'instance enregistree")]
    TypeMismatch(Capability),
}

/// Alias de resultat du domaine.
pub type OmegaResult<T> = Result<T, OmegaError>;
