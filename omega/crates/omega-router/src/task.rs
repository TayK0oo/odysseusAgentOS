//! Taxonomie T1..T14 des types de taches pour le routeur.
//!
//! Source : `docs/omega/research/MODEL-ROUTING-BENCHMARKS.md` §3.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// T1 — Completion triviale / small-talk.
    TrivialCompletion,
    /// T2 — Resume / reformatage.
    Summarization,
    /// T3 — Edition ciblee (1 fichier, patch simple).
    TargetedEdit,
    /// T4 — Refactor / multi-fichiers.
    Refactor,
    /// T5 — Implementation from scratch.
    Implementation,
    /// T6 — Raisonnement long / plan.
    LongReasoning,
    /// T7 — Maths / sciences dures.
    Math,
    /// T8 — Recherche / synthese web.
    Research,
    /// T9 — Agentique terminal / ops.
    AgenticTerminal,
    /// T10 — Vision / OCR / UI.
    Vision,
    /// T11 — Transcription / audio.
    Audio,
    /// T12 — Embeddings / RAG.
    Embedding,
    /// T13 — Generation media (image/video/audio).
    MediaGeneration,
    /// T14 — Chat generaliste.
    Chat,
}

impl TaskType {
    /// Code court T1..T14.
    pub fn code(self) -> &'static str {
        match self {
            TaskType::TrivialCompletion => "T1",
            TaskType::Summarization => "T2",
            TaskType::TargetedEdit => "T3",
            TaskType::Refactor => "T4",
            TaskType::Implementation => "T5",
            TaskType::LongReasoning => "T6",
            TaskType::Math => "T7",
            TaskType::Research => "T8",
            TaskType::AgenticTerminal => "T9",
            TaskType::Vision => "T10",
            TaskType::Audio => "T11",
            TaskType::Embedding => "T12",
            TaskType::MediaGeneration => "T13",
            TaskType::Chat => "T14",
        }
    }
}
