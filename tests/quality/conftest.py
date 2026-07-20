"""Shared fixtures for tests/quality/ — RAG quality evaluation suite.

All tests in this package are gated behind the ODYSSEUS_RAGAS kill-switch.
Set ``ODYSSEUS_RAGAS=on`` to run; CI leaves it unset so these heavy tests
are skipped by default.
"""
import os
import pytest


def pytest_configure(config):
    """Register the ``ragas`` marker so ``-m ragas`` works."""
    config.addinivalue_line(
        "markers",
        "ragas: RAG quality evaluation tests (require ragas + ODYSSEUS_RAGAS=on)",
    )


@pytest.fixture(autouse=True)
def _gate_ragas():
    """Skip every test in this package unless the kill-switch is ON."""
    val = os.getenv("ODYSSEUS_RAGAS", "off").strip().lower()
    if val not in {"on", "1", "true", "yes"}:
        pytest.skip(
            "ODYSSEUS_RAGAS kill-switch is OFF — "
            "set ODYSSEUS_RAGAS=on to run RAG quality tests"
        )
