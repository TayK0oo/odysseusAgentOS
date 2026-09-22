"""Compatibility wrapper — re-exports from archive/legacy/llm_core.py."""

import os
import sys

_archive_dir = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "archive",
    "legacy",
)
if _archive_dir not in sys.path:
    sys.path.insert(0, _archive_dir)

import llm_core as _mod  # noqa: E402

for _name in dir(_mod):
    if _name.startswith("__") and _name.endswith("__"):
        continue
    globals()[_name] = getattr(_mod, _name)
