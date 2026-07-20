# Agent A11: Ragas — RAG Evaluation

## TASK
Integrate Ragas (Apache 2.0) for evaluating the RAG pipeline quality — measuring retrieval precision/recall, generation faithfulness, and benchmarking RRF vs naive blend.

## CONTEXT
- Odysseus RAG: `src/rag_vector.py` — ChromaDB + fastembed, optional RRF fusion (gated OFF)
- No metrics on whether RAG actually retrieves the right context
- Ragas: context precision, context recall, faithfulness, answer relevancy, context entity recall

## REQUIREMENTS

### 1. Dependency
Add `ragas` to `requirements-optional.txt`

### 2. Test Suite
Create `tests/quality/test_rag_quality.py`:
```python
from ragas import evaluate
from ragas.metrics import (
    context_precision, context_recall, 
    faithfulness, answer_relevancy
)
from datasets import Dataset

def test_rag_retrieval_quality():
    """Benchmark RAG against known document set"""
    dataset = Dataset.from_dict({
        "question": ["What is Odysseus?", "How to install?", ...],
        "answer": [...],
        "contexts": [[...], [...]],
        "ground_truth": [...]
    })
    result = evaluate(dataset, metrics=[
        context_precision, context_recall,
        faithfulness, answer_relevancy
    ])
    assert result['context_precision'] > 0.7
    assert result['faithfulness'] > 0.8

def test_rrf_vs_naive_blend():
    """Compare RRF fusion (ODYSSEUS_RRF_FUSION) vs naive blend"""
    # Benchmark both, assert RRF >= naive
    ...
```

### 3. Synthetic Data
Use Ragas `TestsetGenerator` to create synthetic QA pairs from Odysseus docs.

### 4. Kill-Switch
`ODYSSEUS_RAGAS=off` — skip in CI without ChromaDB

## VERIFICATION
- RAG context precision > 0.7
- RRF fusion >= naive blend
- Synthetic test generation works with Odysseus docs

## OUTPUT
Test files, benchmark results (RRF vs naive)
