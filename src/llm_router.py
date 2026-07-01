"""
Model Router basé sur LiteLLM.
Abstraction unique vers tous les providers (ollama, opencode_zen, openrouter, anthropic, groq).

MIGRATION: Ce module remplace progressivement src/llm_core.py pour les nouveaux appels LLM.
"""
import json
import time
import logging
import os
from pathlib import Path
from typing import Optional

try:
    import litellm
    _LITELLM_AVAILABLE = True
except ImportError:
    litellm = None  # type: ignore
    _LITELLM_AVAILABLE = False

logger = logging.getLogger(__name__)

# Répertoire racine du projet (parent de src/)
_PROJECT_ROOT = Path(__file__).parent.parent


class ModelRouter:
    """
    Router LiteLLM : sélectionne le modèle selon l'intent et le stage,
    gère le fallback automatique sur erreur 429/500/502/503.
    """

    def __init__(self, config_path: str = "model-routing.json"):
        config_file = Path(config_path)
        if not config_file.is_absolute():
            config_file = _PROJECT_ROOT / config_path
        self._config_path = config_file
        self._config: dict = {}
        self._reload_config()

    # ------------------------------------------------------------------
    # Config
    # ------------------------------------------------------------------

    def _load_stage_overrides(self) -> dict:
        """Charge .planning/stage-model-assignment.yaml si présent."""
        try:
            import yaml
            candidates = [
                Path(__file__).parent.parent / ".planning" / "stage-model-assignment.yaml",
                Path.cwd() / ".planning" / "stage-model-assignment.yaml",
            ]
            for p in candidates:
                if p.exists():
                    with open(p, "r", encoding="utf-8") as f:
                        data = yaml.safe_load(f) or {}
                    return data.get("stages", {})
        except Exception as exc:
            logger.debug("[ModelRouter] stage-model-assignment.yaml non chargé : %s", exc)
        return {}

    def _reload_config(self) -> None:
        """Recharge model-routing.json à chaud (appelé à chaque route())."""
        try:
            with open(self._config_path, "r", encoding="utf-8") as f:
                self._config = json.load(f)
            logger.debug("model-routing.json rechargé depuis %s", self._config_path)
        except FileNotFoundError:
            logger.error("model-routing.json introuvable : %s", self._config_path)
            self._config = {}
        except json.JSONDecodeError as exc:
            logger.error("model-routing.json invalide : %s", exc)

    # ------------------------------------------------------------------
    # Routing public
    # ------------------------------------------------------------------

    def route(self, intent_category: str, stage: Optional[str] = None) -> str:
        """
        Retourne l'identifiant LiteLLM à utiliser selon l'intent et le stage.

        Priorité :
          1. stage (si fourni) → résolution via config["stages"]
          2. intent_category   → résolution via config["intent_categories"]
          3. fallback "fast"

        Retourne une chaîne au format LiteLLM : "openai/deepseek-v4-flash", etc.
        """
        self._reload_config()

        # Override direct depuis stage-model-assignment.yaml
        if stage:
            overrides = self._load_stage_overrides()
            stage_cfg = overrides.get(stage, {})
            if stage_cfg.get("model"):
                logger.debug(
                    "[ModelRouter] stage=%s → model=%s (yaml override)",
                    stage, stage_cfg["model"],
                )
                return stage_cfg["model"]

        tier_name = self._resolve_tier(intent_category, stage)
        litellm_model = self._tier_to_litellm(tier_name)

        logger.info(
            "[ModelRouter] intent=%s stage=%s → tier=%s → model=%s",
            intent_category, stage, tier_name, litellm_model,
        )
        return litellm_model

    def complete(
        self,
        messages: list,
        intent_category: str = "utility",
        stage: Optional[str] = None,
        **kwargs,
    ):
        """
        Appel LiteLLM avec routing automatique + fallback.

        Paramètres supplémentaires (temperature, max_tokens, stream, …) sont
        transmis à litellm.completion via **kwargs.

        Retourne la réponse LiteLLM (ModelResponse) ou lève une exception si
        tous les fallbacks ont échoué.
        """
        if not _LITELLM_AVAILABLE:
            raise ImportError(
                "litellm n'est pas installé. "
                "Ajouter 'litellm' à requirements.txt et relancer pip install."
            )

        self._reload_config()
        tier_name = self._resolve_tier(intent_category, stage)
        fallback_chain = self._fallback_chain(tier_name)

        retry_statuses = set(
            self._config.get("runtime_fallback", {}).get("retry_on_status", [429, 500, 502, 503])
        )
        cooldown = self._config.get("runtime_fallback", {}).get("cooldown_seconds", 30)
        max_attempts = self._config.get("runtime_fallback", {}).get("max_attempts", 3)

        last_exc = None
        for attempt, tier in enumerate(fallback_chain[:max_attempts], start=1):
            model_str = self._tier_to_litellm(tier)
            api_base, api_key = self._provider_creds(tier)

            logger.info(
                "[ModelRouter] tentative %d/%d : %s (tier=%s)",
                attempt, min(len(fallback_chain), max_attempts), model_str, tier,
            )

            try:
                call_kwargs = dict(kwargs)
                if api_base:
                    call_kwargs["api_base"] = api_base
                if api_key:
                    call_kwargs["api_key"] = api_key

                response = litellm.completion(
                    model=model_str,
                    messages=messages,
                    **call_kwargs,
                )
                logger.info("[ModelRouter] succès avec %s", model_str)
                return response

            except Exception as exc:
                last_exc = exc
                status = getattr(exc, "status_code", None)
                logger.warning(
                    "[ModelRouter] échec %s (status=%s) : %s",
                    model_str, status, exc,
                )

                if status in retry_statuses or status is None:
                    if attempt < min(len(fallback_chain), max_attempts):
                        logger.info("[ModelRouter] cooldown %ss avant fallback…", cooldown)
                        time.sleep(cooldown)
                    continue
                # Erreur non-retriable (ex: 400 bad request) → on arrête
                raise

        raise RuntimeError(
            f"Tous les fallbacks ont échoué pour intent={intent_category} stage={stage}. "
            f"Dernière erreur : {last_exc}"
        ) from last_exc

    # ------------------------------------------------------------------
    # Helpers internes
    # ------------------------------------------------------------------

    def _resolve_tier(self, intent_category: str, stage: Optional[str]) -> str:
        """Résout le nom de tier (strong/fast/code/…) depuis stage ou intent."""
        stages = self._config.get("stages", {})
        intent_categories = self._config.get("intent_categories", {})

        if stage and stage in stages:
            return stages[stage]

        if intent_category in intent_categories:
            return intent_categories[intent_category]

        # Fallback universel
        return "fast"

    def _fallback_chain(self, tier_name: str) -> list[str]:
        """
        Retourne la chaîne de fallback complète pour un tier.
        Le tier lui-même est toujours en tête de liste.
        """
        models_cfg = self._config.get("models", {})
        tier_cfg = models_cfg.get(tier_name, {})
        fallbacks = tier_cfg.get("fallback", [])

        # On vérifie aussi la section fallback_chains si présente
        chains = self._config.get("fallback_chains", {})
        if tier_name in chains:
            fallbacks = chains[tier_name]

        return [tier_name] + [f for f in fallbacks if f != tier_name]

    def _tier_to_litellm(self, tier_name: str) -> str:
        """
        Convertit un nom de tier (ex: "code") en chaîne LiteLLM
        (ex: "openai/kimi-k2.7-code").
        """
        models_cfg = self._config.get("models", {})
        providers_litellm = self._config.get("providers_litellm", {})

        tier_cfg = models_cfg.get(tier_name, {})
        model_id = tier_cfg.get("model_id", tier_name)
        provider = tier_cfg.get("provider", "opencode_zen")

        prefix = providers_litellm.get(provider, "openai/{model}")
        return prefix.replace("{model}", model_id)

    def _provider_creds(self, tier_name: str) -> tuple[Optional[str], Optional[str]]:
        """Retourne (api_base, api_key) pour le provider associé au tier."""
        models_cfg = self._config.get("models", {})
        providers_cfg = self._config.get("providers", {})

        tier_cfg = models_cfg.get(tier_name, {})
        provider_name = tier_cfg.get("provider", "opencode_zen")
        provider = providers_cfg.get(provider_name, {})

        api_base = provider.get("base_url")
        # Retire le suffixe /chat/completions si présent (LiteLLM l'ajoute lui-même)
        if api_base and api_base.endswith("/chat/completions"):
            api_base = api_base[: -len("/chat/completions")]

        api_key_env = provider.get("api_key_env")
        api_key = os.getenv(api_key_env) if api_key_env else None

        return api_base, api_key
