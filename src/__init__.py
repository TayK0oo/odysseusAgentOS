# src/__init__.py
# Expose les modules Phase 2 (LiteLLM Router + IntentGate).
from src.llm_router import ModelRouter
from src.intent_gate import classify_intent, classify_by_word_count
from src.hash_edit_validator import compute_file_hash, validate_and_edit

__all__ = [
    "ModelRouter",
    "classify_intent",
    "classify_by_word_count",
    "compute_file_hash",
    "validate_and_edit",
]
