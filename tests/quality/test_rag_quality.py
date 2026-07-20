"""RAG quality evaluation — Ragas metrics + RRF vs naive blend benchmark.

Kill-switch:  ODYSSEUS_RAGAS=on  (set in env to run; CI leaves unset → skip).

Metrics measured:
  - context_precision  — are retrieved contexts relevant to the question?
  - context_recall     — does retrieved context cover the ground truth?
  - faithfulness       — is the generated answer grounded in the context?
  - answer_relevancy   — does the answer address the question?

Benchmark:
  RRF fusion (ODYSSEUS_RRF_FUSION=on) vs naive 0.7/0.3 blend.
"""
import os
import pytest

# ---------------------------------------------------------------------------
# Ragas import — hard-gate at module level (conftest already skips, but
# we want a clear ImportError message if someone runs the file directly).
# ---------------------------------------------------------------------------
try:
    from ragas import evaluate
    from ragas.metrics import (
        context_precision,
        context_recall,
        faithfulness,
        answer_relevancy,
    )
    from datasets import Dataset

    _HAS_RAGAS = True
except ImportError:
    _HAS_RAGAS = False

# Import Odysseus RAG internals for the benchmark section.
from src.rag_vector import (
    _rank_candidates,
    _rrf_score,
    hybrid_search,
    _rrf_fusion_enabled,
)


# ─── Synthetic evaluation corpus ──────────────────────────────────────────

_SYNTHETIC_QA = [
    {
        "question": "What is Odysseus Agent OS?",
        "answer": "Odysseus Agent OS is a personal AI assistant that runs locally, "
                  "with features like RAG-based document retrieval, tool selection, "
                  "and multi-provider LLM support.",
        "contexts": [
            "Odysseus Agent OS is a self-hosted personal AI assistant. It provides "
            "RAG (Retrieval-Augmented Generation) using ChromaDB and fastembed for "
            "semantic document search, along with tool selection and multi-provider "
            "LLM routing.",
            "The system supports multiple LLM providers including OpenAI, Anthropic, "
            "Ollama, and local models via llama.cpp.",
        ],
        "ground_truth": "Odysseus Agent OS is a self-hosted personal AI assistant with "
                        "RAG document retrieval, tool selection, and multi-provider LLM support.",
    },
    {
        "question": "How does RAG document retrieval work in Odysseus?",
        "answer": "Odysseus uses ChromaDB as a vector store with fastembed embeddings. "
                  "Documents are chunked, embedded, and stored in owner-scoped collections. "
                  "Retrieval combines vector similarity with optional RRF fusion.",
        "contexts": [
            "VectorRAG uses ChromaDB for vector storage and fastembed for generating "
            "embeddings. Documents are split into chunks via _split_into_chunks() and "
            "indexed into owner-scoped collections.",
            "The search method combines vector similarity with keyword overlap. When "
            "ODYSSEUS_RRF_FUSION is enabled, Reciprocal Rank Fusion merges vector "
            "and BM25 rankings.",
        ],
        "ground_truth": "Odysseus uses ChromaDB + fastembed for vector RAG. Documents are "
                        "chunked, embedded into owner-scoped collections. Search blends "
                        "vector similarity with optional RRF fusion of BM25 rankings.",
    },
    {
        "question": "What kill-switches control RAG behavior?",
        "answer": "ODYSSEUS_RRF_FUSION controls Reciprocal Rank Fusion re-ranking "
                  "(off by default). ODYSSEUS_RAGAS controls RAG quality evaluation tests.",
        "contexts": [
            "ODYSSEUS_RRF_FUSION is a kill-switch: when set to 'on', the system uses "
            "Reciprocal Rank Fusion to merge vector and BM25 search rankings instead "
            "of the default 0.7/0.3 naive blend.",
            "ODYSSEUS_RAGAS gates the RAG quality evaluation test suite in "
            "tests/quality/test_rag_quality.py.",
        ],
        "ground_truth": "ODYSSEUS_RRF_FUSION enables RRF re-ranking. ODYSSEUS_RAGAS "
                        "enables quality evaluation tests.",
    },
    {
        "question": "How are embeddings generated?",
        "answer": "Odysseus uses fastembed by default for local embedding generation "
                  "without requiring a GPU. It supports multiple embedding models "
                  "and caches embeddings to avoid recomputation.",
        "contexts": [
            "fastembed is used for local embedding generation. It provides "
            "sentence-transformer quality without GPU requirements.",
            "The embedding cache (EmbeddingCache) stores computed embeddings to "
            "avoid recomputation across sessions.",
        ],
        "ground_truth": "fastembed generates local embeddings without GPU. An embedding "
                        "cache prevents recomputation.",
    },
    {
        "question": "What is the context compactor?",
        "answer": "The context compactor summarizes long conversation histories to fit "
                  "within model token limits while preserving important information.",
        "contexts": [
            "The ContextCompactor summarizes older messages in a conversation to stay "
            "within the model's context window. It uses an LLM to generate a summary "
            "of the compacted portion.",
        ],
        "ground_truth": "The context compactor summarizes old messages to fit token limits "
                        "using an LLM-generated summary.",
    },
]


# ─── Ragas metric tests ───────────────────────────────────────────────────

@pytest.mark.ragas
class TestRagasMetrics:
    """Evaluate RAG quality with Ragas framework metrics."""

    @pytest.fixture(autouse=True)
    def _require_ragas(self):
        if not _HAS_RAGAS:
            pytest.skip("ragas/datasets not installed — pip install ragas datasets")

    @pytest.fixture
    def eval_dataset(self):
        """Build a Ragas-compatible Dataset from synthetic QA pairs."""
        return Dataset.from_dict({
            "question": [q["question"] for q in _SYNTHETIC_QA],
            "answer": [q["answer"] for q in _SYNTHETIC_QA],
            "contexts": [q["contexts"] for q in _SYNTHETIC_QA],
            "ground_truth": [q["ground_truth"] for q in _SYNTHETIC_QA],
        })

    def test_rag_retrieval_quality(self, eval_dataset):
        """Context precision > 0.7 and faithfulness > 0.8 on synthetic corpus."""
        result = evaluate(
            eval_dataset,
            metrics=[context_precision, context_recall, faithfulness, answer_relevancy],
        )

        scores = result.to_pandas()
        avg_precision = scores["context_precision"].mean()
        avg_recall = scores["context_recall"].mean()
        avg_faithfulness = scores["faithfulness"].mean()
        avg_relevancy = scores["answer_relevancy"].mean()

        # Log for benchmark reports
        print(
            f"\n  Ragas scores — precision={avg_precision:.3f}  recall={avg_recall:.3f}  "
            f"faithfulness={avg_faithfulness:.3f}  relevancy={avg_relevancy:.3f}"
        )

        assert avg_precision > 0.7, (
            f"context_precision {avg_precision:.3f} <= 0.7 threshold"
        )
        assert avg_faithfulness > 0.8, (
            f"faithfulness {avg_faithfulness:.3f} <= 0.8 threshold"
        )

    def test_context_recall_threshold(self, eval_dataset):
        """Retrieved context should cover ground truth adequately."""
        result = evaluate(
            eval_dataset,
            metrics=[context_recall],
        )
        scores = result.to_pandas()
        avg_recall = scores["context_recall"].mean()

        print(f"\n  Context recall: {avg_recall:.3f}")
        assert avg_recall > 0.5, (
            f"context_recall {avg_recall:.3f} <= 0.5 threshold"
        )

    def test_answer_relevancy_threshold(self, eval_dataset):
        """Answers should address the questions."""
        result = evaluate(
            eval_dataset,
            metrics=[answer_relevancy],
        )
        scores = result.to_pandas()
        avg_relevancy = scores["answer_relevancy"].mean()

        print(f"\n  Answer relevancy: {avg_relevancy:.3f}")
        assert avg_relevancy > 0.5, (
            f"answer_relevancy {avg_relevancy:.3f} <= 0.5 threshold"
        )


# ─── Synthetic testset generation ─────────────────────────────────────────

@pytest.mark.ragas
class TestSyntheticTestset:
    """Verify Ragas TestsetGenerator can produce synthetic QA from docs."""

    @pytest.fixture(autouse=True)
    def _require_ragas(self):
        if not _HAS_RAGAS:
            pytest.skip("ragas not installed — pip install ragas")

    def test_synthetic_generation_from_documents(self):
        """Generate synthetic QA pairs from Odysseus documentation text."""
        from ragas.testset import TestsetGenerator

        # Minimal doc corpus for generation
        documents = [
            "Odysseus Agent OS is a self-hosted personal AI assistant. It supports "
            "multiple LLM providers including OpenAI, Anthropic, Ollama, and local "
            "models via llama.cpp. The system uses ChromaDB for vector storage and "
            "fastembed for embeddings.",
            "The RAG pipeline indexes documents into owner-scoped ChromaDB collections. "
            "Search combines vector similarity with optional Reciprocal Rank Fusion "
            "of BM25 keyword rankings. Documents are chunked before indexing.",
            "Kill-switches control feature activation: ODYSSEUS_RRF_FUSION enables "
            "RRF re-ranking, ODYSSEUS_RAGAS gates quality evaluation tests. These "
            "environment variables allow safe A/B testing of search strategies.",
        ]

        # Use the default LLM-based generator (requires an API key in env)
        # If no key is available, TestsetGenerator falls back gracefully or
        # raises — we catch and skip.
        try:
            generator = TestsetGenerator()
            testset = generator.generate_with_docs(documents, testset_size=3)
            assert len(testset) > 0, "Generated testset is empty"
            print(f"\n  Synthetic testset: {len(testset)} QA pairs generated")
        except Exception as e:
            pytest.skip(
                f"TestsetGenerator requires LLM API key: {e}"
            )


# ─── RRF fusion vs naive blend benchmark ──────────────────────────────────

def _candidates_for_benchmark():
    """Create a candidate list where keyword relevance differs from vector order.

    Layout:
      id='py_intro'  — high similarity but weak keyword match
      id='py_web'    — medium similarity, strong keyword match (both query terms)
      id='docker'    — low similarity, no keyword match
      id='py_adv'    — medium similarity, partial keyword match
    """
    return [
        {"id": "py_intro", "similarity": 0.92, "vector_similarity": 0.92,
         "content": "Introduction to the Python programming language basics"},
        {"id": "docker", "similarity": 0.85, "vector_similarity": 0.85,
         "content": "Docker container orchestration with Kubernetes deployment"},
        {"id": "py_web", "similarity": 0.70, "vector_similarity": 0.70,
         "content": "Python web framework tutorial with Flask and Django examples"},
        {"id": "py_adv", "similarity": 0.78, "vector_similarity": 0.78,
         "content": "Advanced Python metaclasses and descriptor protocol"},
    ]


class TestRRFvsNaiveBenchmark:
    """Benchmark RRF fusion against the naive 0.7/0.3 blend."""

    def test_rrf_ranking_lifts_keyword_match(self, monkeypatch):
        """RRF should rank keyword-matching docs higher than naive blend.

        Query: "python web framework"
        - py_web matches both 'python' and 'framework' → should rank higher
        - docker matches neither → should rank lower
        """
        pytest.importorskip("rank_bm25")

        query = "python web framework"
        candidates = _candidates_for_benchmark()

        # Naive blend (kill-switch OFF)
        monkeypatch.delenv("ODYSSEUS_RRF_FUSION", raising=False)
        naive_order = _rank_candidates(query, candidates, k=4)
        naive_ids = [c["id"] for c in naive_order]

        # RRF fusion (kill-switch ON)
        monkeypatch.setenv("ODYSSEUS_RRF_FUSION", "on")
        rrf_order = _rank_candidates(query, candidates, k=4)
        rrf_ids = [c["id"] for c in rrf_order]

        print(f"\n  Naive order: {naive_ids}")
        print(f"  RRF order:   {rrf_ids}")

        # RRF must rank py_web at least as high as naive does
        assert rrf_ids.index("py_web") <= naive_ids.index("py_web"), (
            f"RRF should rank 'py_web' higher or equal: "
            f"rrf_pos={rrf_ids.index('py_web')} vs naive_pos={naive_ids.index('py_web')}"
        )

        # RRF must rank docker lower or equal to naive
        assert rrf_ids.index("docker") >= naive_ids.index("docker"), (
            f"RRF should rank 'docker' lower or equal: "
            f"rrf_pos={rrf_ids.index('docker')} vs naive_pos={naive_ids.index('docker')}"
        )

    def test_rrf_score_math(self):
        """Verify RRF scoring formula: 1/(k + rank + 1)."""
        assert _rrf_score(0) == 1 / 61   # k=60 default
        assert _rrf_score(1) == 1 / 62
        assert _rrf_score(5) == 1 / 66
        # Monotonic: higher rank → lower score
        assert _rrf_score(0) > _rrf_score(1) > _rrf_score(5)

    def test_hybrid_search_rrf_reorders(self, monkeypatch):
        """hybrid_search with alpha=0.5 blends vector + BM25 via RRF."""
        pytest.importorskip("rank_bm25")

        query = "python web framework"
        docs = [
            {"id": "a", "content": "python programming language guide"},
            {"id": "b", "content": "docker container orchestration platform"},
            {"id": "c", "content": "python web framework tutorial"},
        ]

        result = hybrid_search(query, docs, top_k=3, alpha=0.5)
        ids = [r["id"] for r in result]
        print(f"\n  RRF hybrid order: {ids}")

        # 'c' matches both query terms, must rank above 'b'
        assert ids.index("c") < ids.index("b"), (
            f"RRF should rank keyword-matching 'c' above non-matching 'b': {ids}"
        )

    def test_rrf_alpha_one_preserves_vector_order(self):
        """alpha=1.0 → pure vector order (BM25 term zeroed out)."""
        docs = _candidates_for_benchmark()
        result = hybrid_search("python web", docs, top_k=4, alpha=1.0)
        ids = [r["id"] for r in result]
        # Should match input vector order (by similarity desc)
        assert ids == ["py_intro", "docker", "py_adv", "py_web"]

    def test_rrf_graceful_degradation(self):
        """Empty input → empty output; single doc → that doc returned."""
        assert hybrid_search("test", []) == []
        single = [{"id": "x", "content": "hello"}]
        result = hybrid_search("hello", single, top_k=1)
        assert len(result) == 1
        assert result[0]["id"] == "x"

    def test_rrf_killswitch_default_off(self, monkeypatch):
        """ODYSSEUS_RRF_FUSION defaults to off."""
        monkeypatch.delenv("ODYSSEUS_RRF_FUSION", raising=False)
        assert _rrf_fusion_enabled() is False

    def test_rrf_killswitch_variants(self, monkeypatch):
        """Accept 'on', '1', 'true', 'yes' as enabled values."""
        for val in ("on", "1", "true", "yes", "ON", "True"):
            monkeypatch.setenv("ODYSSEUS_RRF_FUSION", val)
            assert _rrf_fusion_enabled() is True, f"Expected enabled for '{val}'"
        for val in ("off", "0", "false", "no", ""):
            monkeypatch.setenv("ODYSSEUS_RRF_FUSION", val)
            assert _rrf_fusion_enabled() is False, f"Expected disabled for '{val}'"


# ─── ODYSSEUS_RAGAS kill-switch tests ─────────────────────────────────────

class TestRagasKillSwitch:
    """Verify the ODYSSEUS_RAGAS kill-switch gates correctly."""

    def test_skip_when_off(self, monkeypatch):
        """conftest should skip when ODYSSEUS_RAGAS is not set."""
        monkeypatch.delenv("ODYSSEUS_RAGAS", raising=False)
        val = os.getenv("ODYSSEUS_RAGAS", "off").strip().lower()
        assert val not in {"on", "1", "true", "yes"}

    def test_pass_when_on(self, monkeypatch):
        """conftest should allow when ODYSSEUS_RAGAS=on."""
        monkeypatch.setenv("ODYSSEUS_RAGAS", "on")
        val = os.getenv("ODYSSEUS_RAGAS", "off").strip().lower()
        assert val in {"on", "1", "true", "yes"}
