"""Compatibility shim — loads archive/legacy/llm_core.py into this module's namespace.

The canonical LLM dispatch (stream_llm / llm_call / llm_call_async / …) physically
lives in archive/legacy/llm_core.py. We must NOT import it as a separate top-level
module (``import llm_core``) because that splits the module globals in two: the
imported functions resolve ``_get_http_client`` / ``stream_llm`` / ``LLMConfig`` /
… from the *other* module's ``__dict__``, so monkeypatching or ``importlib.reload``
on ``src.llm_core`` has no effect. Instead the legacy source is exec'd right here,
so every function resolves its globals from THIS module — ``src.llm_core`` — exactly
as callers, monkeypatchers and ``importlib.reload`` expect.
"""

import os

_archive_dir = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "archive",
    "legacy",
)
_legacy_path = os.path.join(_archive_dir, "llm_core.py")

with open(_legacy_path, "rb") as _f:
    _code = compile(_f.read(), _legacy_path, "exec")

exec(_code, globals())
