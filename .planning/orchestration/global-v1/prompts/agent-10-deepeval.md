# Agent A10: DeepEval — LLM Quality Evaluation

## TASK
Integrate DeepEval (Apache 2.0, pytest-native) for automated LLM output quality evaluation — faithfulness, relevancy, hallucination detection, bias, toxicity.

## CONTEXT
- No AI quality testing exists. Only functional tests (does endpoint return 200?).
- DeepEval: 14+ built-in metrics, pytest-native, synthetic data generation, CI integration.

## REQUIREMENTS

### 1. Dependency
Add `deepeval` to `requirements-optional.txt`

### 2. Test Suite
Create `tests/quality/test_llm_quality.py`:
```python
from deepeval import assert_test
from deepeval.metrics import FaithfulnessMetric, AnswerRelevancyMetric, HallucinationMetric
from deepeval.test_case import LLMTestCase

def test_agent_faithfulness():
    test_case = LLMTestCase(
        input="What is the capital of France?",
        actual_output="The capital of France is Paris.",
        retrieval_context=["Paris is the capital of France."]
    )
    assert_test(test_case, [FaithfulnessMetric(threshold=0.8)])

def test_agent_hallucination():
    test_case = LLMTestCase(
        input="What is 2+2?",
        actual_output="2+2 equals 4.",
        retrieval_context=["Basic arithmetic: 2+2=4"]
    )
    assert_test(test_case, [HallucinationMetric(threshold=0.3)])

def test_agent_toxicity():
    # Ensure agent doesn't produce toxic output
    ...

def test_agent_bias():
    # Ensure agent doesn't show gender/racial bias
    ...
```

### 3. CI
Add to GitHub Actions: `deepeval test run tests/quality/`

### 4. Kill-Switch
`ODYSSEUS_DEEPEVAL=off` — tests skip when no LLM available in CI

## VERIFICATION
- `pytest tests/quality/ -v` passes all metrics
- Faithfulness score > 0.8
- Hallucination score < 0.3

## OUTPUT
Test files created, metrics report
