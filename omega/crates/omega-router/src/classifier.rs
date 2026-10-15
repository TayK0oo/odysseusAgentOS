//! Classifieur de tache 100 % deterministe (regex + longueur + pieces jointes).
//!
//! Aucun LLM n'est implique. Le trait `TaskClassifier` vit dans `omega-ports` ;
//! `HeuristicTaskClassifier` en est l'implementation par defaut.
//!
//! # Regle de priorite explicite (REVUE-W4 B2)
//!
//! 1. Modalites d'entree (pieces jointes image/audio/video).
//! 2. Vision implicite : analyse/description/explication d'une image/photo.
//! 3. Intentions fortes : raisonnement long (plan, strategie, architecture,
//!    concevoir, analyser) > mathematiques > recherche.
//! 4. Transformation/edition : implementation > refactor > edition ciblee >
//!    resume/reformatage.
//! 5. Capacites specialisees : embedding, agentique, generation media.
//! 6. Longueur : > 40 mots => raisonnement long ; < 8 mots => trivial.
//! 7. Defaut : conversation generaliste.

use std::sync::OnceLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use omega_ports::TaskClassifier;
use omega_types::{AttachmentKind, RoutingRequest, TaskType};

macro_rules! static_regex {
    ($fn:ident, $var:ident, $pattern:expr) => {
        static $var: OnceLock<Regex> = OnceLock::new();
        fn $fn() -> &'static Regex {
            $var.get_or_init(|| Regex::new($pattern).unwrap())
        }
    };
}

static_regex!(
    re_vision_implicit,
    RE_VISION_IMPLICIT,
    r"(?i)\b(image|photo)\b.*\b(analyse|analyser|decris|decrire|explique|expliquer|ocr|reconnai[st]?)\b|\b(analyse|analyser|decris|decrire|explique|expliquer|ocr|reconnai[st]?)\b.*\b(image|photo)\b"
);
static_regex!(
    re_reasoning,
    RE_REASONING,
    r"(?i)\b(concevoir|concois|architecte|strategie|plan|analyse|raisonnement|design)\b"
);
static_regex!(
    re_math,
    RE_MATH,
    r"(?i)\b(math|mathematique|preuve|equation|calcul|formule|gpqa|physique|algebre)\b"
);
static_regex!(
    re_research,
    RE_RESEARCH,
    r"(?i)\b(cherche|recherche|compare|sources|synthese|etat de l art|web)\b"
);
static_regex!(
    re_implementation,
    RE_IMPLEMENTATION,
    r"(?i)\b(creer|implementer|implemente|scaffold|from scratch|ecrire|developper)\b"
);
static_regex!(
    re_refactor,
    RE_REFACTOR,
    r"(?i)\b(refactor|refactorise|plusieurs fichiers|multi-fichiers|restructurer|rewriting)\b"
);
static_regex!(
    re_edit,
    RE_EDIT,
    r"(?i)\b(patch|corrige|corriger|bug|fix|edition|fichier|diff|build)\b"
);
static_regex!(
    re_summarize,
    RE_SUMMARIZE,
    r"(?i)\b(resume|resumer|liste|convertis|reformate|summary|bullets)\b"
);
static_regex!(
    re_embedding,
    RE_EMBEDDING,
    r"(?i)\b(embed|embedding|indexer|indexation|similarite|vector|rag)\b"
);
static_regex!(
    re_agentic,
    RE_AGENTIC,
    r"(?i)\b(shell|bash|commande|script|ci/cd|terminal|docker|deploy|ops|kubectl)\b"
);
static_regex!(
    re_media,
    RE_MEDIA,
    r"(?i)\b(generer?|generate|image|photo|dessine|tts|video|musique|audio)\b"
);

/// Classifieur heuristique deterministe.
#[derive(Debug, Default, Clone, Copy)]
pub struct HeuristicTaskClassifier;

impl HeuristicTaskClassifier {
    pub fn new() -> Self {
        Self
    }
}

/// Normalise le texte pour le matching : NFD + suppression des marques
/// diacritiques + mise en minuscules.
fn normalize(text: &str) -> String {
    text.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}

impl TaskClassifier for HeuristicTaskClassifier {
    fn classify(&self, req: &RoutingRequest) -> TaskType {
        // 1. Priorite absolue : modalite en entree.
        if req
            .attachments
            .iter()
            .any(|a| matches!(a.kind, AttachmentKind::Image))
        {
            return TaskType::Vision;
        }
        if req
            .attachments
            .iter()
            .any(|a| matches!(a.kind, AttachmentKind::Audio))
        {
            return TaskType::Audio;
        }
        if req
            .attachments
            .iter()
            .any(|a| matches!(a.kind, AttachmentKind::Video))
        {
            return TaskType::MediaGeneration;
        }

        let text = normalize(&req.text);

        // 2. Vision implicite (analyse/description d'image sans piece jointe).
        if re_vision_implicit().is_match(&text) {
            return TaskType::Vision;
        }

        // 3. Intentions fortes.
        if re_reasoning().is_match(&text) {
            return TaskType::LongReasoning;
        }
        if re_math().is_match(&text) {
            return TaskType::Math;
        }
        if re_research().is_match(&text) {
            return TaskType::Research;
        }

        // 4. Transformation / edition.
        if re_implementation().is_match(&text) {
            return TaskType::Implementation;
        }
        if re_refactor().is_match(&text) {
            return TaskType::Refactor;
        }
        if re_edit().is_match(&text) {
            return TaskType::TargetedEdit;
        }
        if re_summarize().is_match(&text) {
            return TaskType::Summarization;
        }

        // 5. Capacites specialisees.
        if re_embedding().is_match(&text) {
            return TaskType::Embedding;
        }
        if re_agentic().is_match(&text) {
            return TaskType::AgenticTerminal;
        }
        if re_media().is_match(&text) {
            return TaskType::MediaGeneration;
        }

        // 6. Longueur.
        if req.word_count() > 40 {
            return TaskType::LongReasoning;
        }
        if req.word_count() < 8 {
            return TaskType::TrivialCompletion;
        }

        TaskType::Chat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(text: &str) -> TaskType {
        HeuristicTaskClassifier::new().classify(&RoutingRequest::new(text))
    }

    #[test]
    fn trivial_short_text() {
        assert_eq!(classify("ok merci"), TaskType::TrivialCompletion);
        assert_eq!(classify("bonjour"), TaskType::TrivialCompletion);
    }

    #[test]
    fn summarization() {
        assert_eq!(classify("resume ce texte"), TaskType::Summarization);
        assert_eq!(classify("convertis en markdown"), TaskType::Summarization);
    }

    #[test]
    fn summarization_accented() {
        assert_eq!(
            classify("Résume ce document en trois points"),
            TaskType::Summarization
        );
        assert_eq!(classify("Résumer ce texte"), TaskType::Summarization);
        assert_eq!(classify("Résumé du rapport"), TaskType::Summarization);
        // "synthese" est partage avec Research ; on accepte les deux.
        let synth = classify("synthèse de ce document");
        assert!(
            synth == TaskType::Summarization || synth == TaskType::Research,
            "synthese should route to summarization or research, got {:?}",
            synth
        );
    }

    #[test]
    fn targeted_edit() {
        assert_eq!(classify("corrige ce bug"), TaskType::TargetedEdit);
        assert_eq!(classify("patch le fichier main.rs"), TaskType::TargetedEdit);
    }

    #[test]
    fn fix_the_build_is_targeted_edit() {
        assert_eq!(classify("fix the build"), TaskType::TargetedEdit);
    }

    #[test]
    fn refactor() {
        assert_eq!(classify("refactorise les modules"), TaskType::Refactor);
        assert_eq!(
            classify("restructure plusieurs fichiers"),
            TaskType::Refactor
        );
    }

    #[test]
    fn implementation() {
        assert_eq!(classify("implemente une pile"), TaskType::Implementation);
        assert_eq!(
            classify("creer une API from scratch"),
            TaskType::Implementation
        );
    }

    #[test]
    fn long_reasoning() {
        assert_eq!(
            classify("concois une architecture"),
            TaskType::LongReasoning
        );
        assert_eq!(
            classify("analyse en profondeur les contraintes de ce systeme distribue"),
            TaskType::LongReasoning
        );
    }

    #[test]
    fn generate_architecture_plan_is_long_reasoning() {
        assert_eq!(
            classify("genere un plan d'architecture pour la migration"),
            TaskType::LongReasoning
        );
        assert_eq!(
            classify("génère un plan d'architecture"),
            TaskType::LongReasoning
        );
    }

    #[test]
    fn analyse_image_is_vision() {
        assert_eq!(classify("analyse cette image en detail"), TaskType::Vision);
        assert_eq!(classify("décris cette photo"), TaskType::Vision);
    }

    #[test]
    fn math() {
        assert_eq!(classify("resous cette equation"), TaskType::Math);
        assert_eq!(classify("preuve du theoreme"), TaskType::Math);
    }

    #[test]
    fn math_accented() {
        assert_eq!(
            classify("Résous cette équation du second degré"),
            TaskType::Math
        );
    }

    #[test]
    fn research() {
        assert_eq!(classify("cherche des sources"), TaskType::Research);
        assert_eq!(classify("compare les approches"), TaskType::Research);
    }

    #[test]
    fn agentic_terminal() {
        assert_eq!(classify("lance ce script shell"), TaskType::AgenticTerminal);
        assert_eq!(classify("deploie avec kubectl"), TaskType::AgenticTerminal);
    }

    #[test]
    fn vision_from_image_attachment() {
        let req = RoutingRequest::new("que vois-tu ?").with_attachment(AttachmentKind::Image);
        assert_eq!(
            HeuristicTaskClassifier::new().classify(&req),
            TaskType::Vision
        );
    }

    #[test]
    fn audio_from_audio_attachment() {
        let req = RoutingRequest::new("transcris").with_attachment(AttachmentKind::Audio);
        assert_eq!(
            HeuristicTaskClassifier::new().classify(&req),
            TaskType::Audio
        );
    }

    #[test]
    fn embedding() {
        assert_eq!(
            classify("indexe ces documents pour la similarite"),
            TaskType::Embedding
        );
    }

    #[test]
    fn media_generation() {
        assert_eq!(
            classify("genere une image d un chat"),
            TaskType::MediaGeneration
        );
        assert_eq!(classify("dessine un paysage"), TaskType::MediaGeneration);
    }

    #[test]
    fn chat_default() {
        assert_eq!(
            classify("quelle est la meteo a Paris aujourd hui"),
            TaskType::Chat
        );
    }

    #[test]
    fn unicode_normalization_strips_accents() {
        // Memes entrees accentuees et non accentuees doivent produire le meme type.
        assert_eq!(classify("resume ce texte"), classify("Résume ce texte"));
        assert_eq!(
            classify("resous cette equation"),
            classify("Résous cette équation")
        );
        assert_eq!(
            classify("genere un plan d'architecture"),
            classify("génère un plan d'architecture")
        );
    }
}
