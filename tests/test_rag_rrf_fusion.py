"""M3.2 — repair the orphan RRF hybrid_search.

The shipped hybrid_search used a reflection hack that scanned module-level
functions for a `search`; the real search is a class *method*, so the loop
never found it, always fell to vector_results=[], and returned [] every time
(dead code). This repairs it to a pure, testable fusion: the caller passes the
ranked vector results in, and hybrid_search fuses them with BM25 (over the same
docs) via Reciprocal Rank Fusion. These tests pin the new contract.
"""
import pytest

from src.rag_vector import hybrid_search, _rrf_score


def _docs():
    return [
        {"id": "a", "content": "python programming language guide"},
        {"id": "b", "content": "docker container orchestration platform"},
        {"id": "c", "content": "python web framework tutorial"},
    ]


def test_empty_vector_results_returns_empty():
    assert hybrid_search("python", []) == []


def test_returns_same_doc_objects_capped_at_top_k():
    docs = _docs()
    out = hybrid_search("python framework", docs, top_k=2, alpha=0.5)
    assert len(out) <= 2
    # every returned object is one of the inputs (identity preserved)
    for r in out:
        assert any(r is d for d in docs)


def test_alpha_one_is_pure_vector_order():
    # alpha=1.0 → BM25 term is zero → fused order == input vector order.
    docs = _docs()
    out = hybrid_search("python framework", docs, top_k=3, alpha=1.0)
    assert [r["id"] for r in out] == ["a", "b", "c"]


def test_rrf_boosts_doc_matching_keywords():
    pytest.importorskip("rank_bm25")
    # Query terms hit 'a' and 'c' (python / framework), never 'b'.
    # Under pure vector order b sits at rank 1; RRF fusion with BM25 must push
    # the keyword-matching 'c' above the non-matching 'b'.
    docs = _docs()
    out = hybrid_search("python framework", docs, top_k=3, alpha=0.5)
    ids = [r["id"] for r in out]
    assert set(ids) == {"a", "b", "c"}
    assert ids.index("c") < ids.index("b")


def test_rrf_score_monotonic_decreasing():
    assert _rrf_score(0) > _rrf_score(1) > _rrf_score(5)
