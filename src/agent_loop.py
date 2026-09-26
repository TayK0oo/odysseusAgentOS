"""Compatibility shim — loads archive/legacy/agent_loop.py into this module's namespace.

The canonical streaming agent loop (stream_agent_loop / _resolve_tool_blocks /
parse helpers / …) physically lives in archive/legacy/agent_loop.py. We must NOT
import it as a separate top-level module (``import agent_loop``) because that
splits the module globals in two: the imported functions resolve
``stream_llm_with_fallback`` / ``execute_tool_block`` / ``blocked_tools_for_owner``
/ … from the *other* module's ``__dict__``, so monkeypatching or
``importlib.reload`` on ``src.agent_loop`` has no effect. Instead the legacy
source is exec'd right here, so every function resolves its globals from THIS
module — ``src.agent_loop`` — exactly as callers, monkeypatchers and
``importlib.reload`` expect.
"""

import os

_archive_dir = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "archive",
    "legacy",
)
_legacy_path = os.path.join(_archive_dir, "agent_loop.py")

with open(_legacy_path, "rb") as _f:
    _code = compile(_f.read(), _legacy_path, "exec")

exec(_code, globals())
