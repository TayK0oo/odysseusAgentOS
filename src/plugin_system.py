"""
OpenCode Plugin System for Odysseus.

Simple plugin architecture: a plugin is a Python module in plugins/
that registers hooks (on_startup, on_phase, on_tool_call, on_shutdown).
"""

import importlib, logging, os, sys
from typing import Any, Callable, Optional

logger = logging.getLogger(__name__)

_plugins: dict[str, dict] = {}

def discover_plugins(plugins_dir: str = "plugins") -> list[str]:
    """Discover plugin modules in the plugins/ directory."""
    if not os.path.exists(plugins_dir):
        return []
    plugins = []
    for name in os.listdir(plugins_dir):
        path = os.path.join(plugins_dir, name)
        if os.path.isdir(path) and os.path.exists(os.path.join(path, "__init__.py")):
            plugins.append(f"plugins.{name}")
        elif name.endswith(".py") and not name.startswith("_"):
            plugins.append(f"plugins.{name[:-3]}")
    return plugins

def load_plugin(module_name: str) -> Optional[dict]:
    """Load a plugin and extract its hooks."""
    try:
        mod = importlib.import_module(module_name)
        hooks = {}
        for attr in dir(mod):
            obj = getattr(mod, attr)
            if callable(obj) and hasattr(obj, "_plugin_hook"):
                hooks[obj._plugin_hook] = obj
        if hooks:
            _plugins[module_name] = {
                "module": mod,
                "hooks": hooks,
            }
            logger.info("Plugin loaded: %s (%d hooks)", module_name, len(hooks))
            return _plugins[module_name]
    except Exception as e:
        logger.warning("Failed to load plugin %s: %s", module_name, e)
    return None

def load_all(plugins_dir: str = "plugins") -> int:
    """Discover and load all plugins."""
    discovered = discover_plugins(plugins_dir)
    loaded = 0
    for name in discovered:
        if load_plugin(name):
            loaded += 1
    logger.info("Plugins: %d discovered, %d loaded", len(discovered), loaded)
    return loaded

def fire_hook(hook_name: str, *args, **kwargs) -> list[Any]:
    """Fire a hook across all loaded plugins."""
    results = []
    for name, plugin in _plugins.items():
        if hook_name in plugin["hooks"]:
            try:
                result = plugin["hooks"][hook_name](*args, **kwargs)
                results.append(result)
            except Exception as e:
                logger.warning("Plugin %s hook %s failed: %s", name, hook_name, e)
    return results

# Decorator for plugin authors
def hook(name: str):
    """Mark a function as a plugin hook."""
    def decorator(fn):
        fn._plugin_hook = name
        return fn
    return decorator

# Built-in hooks
@hook("on_startup")
def _builtin_startup():
    logger.info("Plugin system initialized, %d plugins loaded", len(_plugins))
