//! Classifieur de tache 100 % deterministe (regex + longueur + pieces jointes).
//!
//! Aucun LLM n'est implique. Le trait `TaskClassifier` vit desormais dans
//! `omega-ports` ; `HeuristicTaskClassifier` en est l'implementation par defaut.

use regex::Regex;

use omega_ports::TaskClassifier;
use omega_types::{AttachmentKind, RoutingRequest, TaskType};

/// Classifieur heuristique deterministe.
pub struct HeuristicTaskClassifier {
    re_media: Regex,
    re_embedding: Regex,
    re_agentic: Regex,
    re_math: Regex,
    re_research: Regex,
    re_implementation: Regex,
    re_refactor: Regex,
    re_edit: Regex,
    re_summarize: Regex,
    re_reasoning: Regex,
}

impl Default for HeuristicTaskClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicTaskClassifier {
    pub fn new() -> Self {
        Self {
            re_media: Regex::new(
                r"(?i)\b(generer?|generate|image|photo|dessine|tts|video|musique|audio)\b",
            )
            .unwrap(),
            re_embedding: Regex::new(r"(?i)\b(embed|embedding|indexer|indexation|similarite|vector|rag)\b").unwrap(),
            re_agentic: Regex::new(
                r"(?i)\b(shell|bash|commande|script|ci/cd|terminal|docker|deploy|ops|kubectl)\b",
            )
            .unwrap(),
            re_math: Regex::new(r"(?i)\b(math|mathematique|preuve|equation|calcul|formule|gpqa|physique|algebre)\b").unwrap(),
            re_research: Regex::new(r"(?i)\b(cherche|recherche|compare|sources|synthese|etat de l art|web)\b").unwrap(),
            re_implementation: Regex::new(r"(?i)\b(creer|implementer|implemente|build|scaffold|from scratch|ecrire|developper)\b").unwrap(),
            re_refactor: Regex::new(r"(?i)\b(refactor|refactorise|plusieurs fichiers|multi-fichiers|restructurer|rewriting)\b").unwrap(),
            re_edit: Regex::new(r"(?i)\b(patch|corrige|corriger|bug|fix|edition|fichier|diff)\b").unwrap(),
            re_summarize: Regex::new(r"(?i)\b(resume|resumer|liste|convertis|reformate|summary|bullets)\b").unwrap(),
            re_reasoning: Regex::new(r"(?i)\b(concevoir|concois|architecte|strategie|plan|analyse|raisonnement|design)\b").unwrap(),
        }
    }
}

impl TaskClassifier for HeuristicTaskClassifier {
    fn classify(&self, req: &RoutingRequest) -> TaskType {
        // Priorite absolue : modalite en entree.
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

        let text = req.lower();

        if self.re_embedding.is_match(&text) {
            return TaskType::Embedding;
        }
        if self.re_media.is_match(&text) {
            return TaskType::MediaGeneration;
        }
        if self.re_agentic.is_match(&text) {
            return TaskType::AgenticTerminal;
        }
        if self.re_math.is_match(&text) {
            return TaskType::Math;
        }
        if self.re_research.is_match(&text) {
            return TaskType::Research;
        }
        if self.re_implementation.is_match(&text) {
            return TaskType::Implementation;
        }
        if self.re_refactor.is_match(&text) {
            return TaskType::Refactor;
        }
        if self.re_edit.is_match(&text) {
            return TaskType::TargetedEdit;
        }
        if self.re_summarize.is_match(&text) {
            return TaskType::Summarization;
        }
        if self.re_reasoning.is_match(&text) || req.word_count() > 40 {
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
    fn targeted_edit() {
        assert_eq!(classify("corrige ce bug"), TaskType::TargetedEdit);
        assert_eq!(classify("patch le fichier main.rs"), TaskType::TargetedEdit);
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
    fn math() {
        assert_eq!(classify("resous cette equation"), TaskType::Math);
        assert_eq!(classify("preuve du theoreme"), TaskType::Math);
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
    }

    #[test]
    fn chat_default() {
        assert_eq!(
            classify("quelle est la meteo a Paris aujourd hui"),
            TaskType::Chat
        );
    }
}
