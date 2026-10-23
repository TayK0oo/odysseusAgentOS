"""Rouge : sequences scriptees R6a (import + arite)."""
from scripted_provider import (
    seq_long_context_compaction,
    seq_memory_store_then_recall,
)


def test_seq_memory_store_then_recall_arite():
    seq = seq_memory_store_then_recall("CANARY42")
    assert len(seq) == 3
    assert all(callable(f) for f in seq)


def test_seq_long_context_compaction_arite():
    seq = seq_long_context_compaction(["fait-debut", "fait-milieu"])
    assert 8 <= len(seq) <= 15
    assert all(callable(f) for f in seq)
