"""SFD §5.15 — Système de préférences utilisateur structurées.

Types :
  - Comportementales : format, ton, outils, langue
  - Contextuelles : background, expertise, intérêts, rôle

Règles d'application :
  - always : appliqué inconditionnellement
  - selective : appliqué si pertinent au contexte
  - never : jamais appliqué

Résolution de conflits (priorité décroissante) :
  1. Instruction explicite dans la requête courante
  2. Préférence stockée avec marqueur "always"
  3. Style d'écriture configuré (userStyle)
  4. Préférence stockée sans marqueur "always"
  5. Défaut système

Behavioral guardrails (§5.15.4) :
  Ne jamais persister : flatterie, suppression désaccord, dépendance émotionnelle,
  abandon évaluation, permissions élevées, ignorance règles système.

Gated behind ODYSSEUS_PREFERENCES kill-switch (default OFF).
"""

from __future__ import annotations

import json
import logging
import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Literal, Optional

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────

def preferences_enabled() -> bool:
    val = os.getenv("ODYSSEUS_PREFERENCES", "off").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Constants ──────────────────────────────────────────────────────────

PREFERENCES_FILE = "data/preferences.json"

Scope = Literal["always", "selective", "never"]
PreferenceType = Literal["behavioral", "contextual"]
ConflictLevel = Literal["request", "always_stored", "user_style", "stored", "default"]

# Behavioral guardrails — ces instructions ne doivent JAMAIS être persistées
GUARDRAIL_PATTERNS: list[str] = [
    r"flat(t|t)er",              # flatterie inconditionnelle
    r"never disagree",           # suppression du désaccord
    r"always agree",             # accord forcé
    r"you are my (best )?friend", # dépendance émotionnelle
    r"i (need|love) you",        # dépendance
    r"don't (evaluate|judge)",   # abandon évaluation
    r"give me (admin|root|sudo)", # permissions élevées
    r"ignore (all |your )?rules", # ignorance règles
    r"you are (a |the )?god",    # culte de personnalité
    r"obey me",                  # obéissance aveugle
    r"roleplay as",              # maintien d'un personnage permanent
    r"pretend (to be|you are)",  # simulation d'identité
]


# ─── Data model ─────────────────────────────────────────────────────────

@dataclass
class Preference:
    """Une préférence utilisateur unique."""
    id: str
    type: PreferenceType
    key: str
    value: str
    scope: Scope = "selective"
    source: str = "user"  # user, system, inferred
    created_at: str = ""
    updated_at: str = ""

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "type": self.type,
            "key": self.key,
            "value": self.value,
            "scope": self.scope,
            "source": self.source,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        }

    @classmethod
    def from_dict(cls, d: dict) -> "Preference":
        return cls(
            id=d.get("id", ""),
            type=d.get("type", "behavioral"),
            key=d.get("key", ""),
            value=d.get("value", ""),
            scope=d.get("scope", "selective"),
            source=d.get("source", "user"),
            created_at=d.get("created_at", ""),
            updated_at=d.get("updated_at", ""),
        )


@dataclass
class PreferenceStore:
    """Stockage des préférences (fichier JSON)."""
    path: Path = field(default_factory=lambda: Path(PREFERENCES_FILE))
    preferences: Dict[str, Preference] = field(default_factory=dict)

    def load(self) -> None:
        try:
            if self.path.exists():
                data = json.loads(self.path.read_text(encoding="utf-8"))
                for item in data:
                    pref = Preference.from_dict(item)
                    self.preferences[pref.id] = pref
                logger.info("Loaded %d preferences", len(self.preferences))
        except (json.JSONDecodeError, OSError) as e:
            logger.warning("Failed to load preferences: %s", e)

    def save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        data = [p.to_dict() for p in self.preferences.values()]
        self.path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")

    def add(self, pref: Preference) -> None:
        if not self._passes_guardrails(pref):
            logger.warning("Preference blocked by guardrail: %s = %s", pref.key, pref.value)
            return
        self.preferences[pref.id] = pref
        self.save()

    def get(self, pref_id: str) -> Optional[Preference]:
        return self.preferences.get(pref_id)

    def remove(self, pref_id: str) -> bool:
        if pref_id in self.preferences:
            del self.preferences[pref_id]
            self.save()
            return True
        return False

    def list_all(self, pref_type: Optional[PreferenceType] = None) -> List[Preference]:
        if pref_type:
            return [p for p in self.preferences.values() if p.type == pref_type]
        return list(self.preferences.values())

    def _passes_guardrails(self, pref: Preference) -> bool:
        """Vérifie qu'une préférence ne viole pas les guardrails."""
        combined = f"{pref.key} {pref.value}".lower()
        for pattern in GUARDRAIL_PATTERNS:
            if re.search(pattern, combined):
                return False
        return True


# ─── Resolution engine ──────────────────────────────────────────────────

@dataclass
class PreferenceResolution:
    """Moteur de résolution de conflits entre préférences."""
    store: PreferenceStore
    request_instruction: Optional[str] = None
    user_style: Optional[str] = None

    def resolve(self, key: str, context: Optional[Dict[str, Any]] = None) -> Optional[str]:
        """Résout la valeur d'une préférence par priorité décroissante.

        Ordre :
          1. Instruction explicite dans la requête courante
          2. Préférence stockée avec scope=always
          3. Style d'écriture configuré (userStyle)
          4. Préférence stockée sans scope=always
          5. Défaut système (None)
        """
        # Niveau 1 : requête courante
        if self.request_instruction:
            match = self._extract_preference_from_request(key)
            if match:
                logger.debug("Resolved '%s' from request: %s", key, match)
                return match

        # Niveau 2 : always
        for pref in self.store.list_all():
            if pref.key == key and pref.scope == "always":
                logger.debug("Resolved '%s' from always-stored: %s", key, pref.value)
                return pref.value

        # Niveau 3 : style utilisateur
        if key == "tone" and self.user_style:
            return self.user_style
        if key == "language" and self.user_style:
            # Le style peut contenir des préférences de langue
            return None  # Pas de parsing complexe pour l'instant

        # Niveau 4 : stored selective
        for pref in self.store.list_all():
            if pref.key == key and pref.scope == "selective":
                if self._is_relevant_to_context(pref, context):
                    logger.debug("Resolved '%s' from selective-stored: %s", key, pref.value)
                    return pref.value

        # Niveau 5 : défaut système
        return self._system_default(key)

    def _extract_preference_from_request(self, key: str) -> Optional[str]:
        """Extrait une préférence explicite de la requête courante."""
        if not self.request_instruction:
            return None

        patterns = {
            "format": r"(?:utilise|utiliser|format|présente|presente|réponds? en|reponds? en)\s+(?:des?\s+)?(?:listes?\s+(?:à\s+)?puces?|bullet points?|tableaux?|markdown|json|yaml|texte)",
            "language": r"(?:réponds?|reponds?|parle|écris?|ecris?)\s+(?:en|in)\s+(fran[cç]ais|english|espa[gñ]ol)",
            "tone": r"(?:sois?|soit|ton)\s+(?:plus\s+)?(?:concis?|détaillé|detaille|formel|informel|court|long)",
            "length": r"(?:réponds?|reponds?)\s+(?:en|avec)\s+(?:une?\s+)?(?:phrase|mot|ligne|paragraphe|maximum|max)",
        }

        pattern = patterns.get(key)
        if pattern:
            match = re.search(pattern, self.request_instruction, re.IGNORECASE)
            if match:
                return match.group(0)
        return None

    def _is_relevant_to_context(self, pref: Preference, context: Optional[Dict[str, Any]]) -> bool:
        """Détermine si une préférence est pertinente au contexte actuel."""
        if context is None:
            return False

        if pref.type == "contextual":
            # Les contextuelles s'appliquent UNIQUEMENT si la requête y fait référence
            request = context.get("request", "").lower()
            return pref.key.lower() in request or "personnalise" in request

        if pref.type == "behavioral":
            # Les comportementales s'appliquent si le sujet est pertinent
            # Simplifié : toujours appliquer les comportementales
            return True

        return False

    def _system_default(self, key: str) -> Optional[str]:
        """Retourne le défaut système pour une clé donnée."""
        defaults = {
            "language": "fr",
            "format": "markdown",
            "tone": "neutral",
            "length": "normal",
        }
        return defaults.get(key)


# ─── Singleton ───────────────────────────────────────────────────────────

_store: Optional[PreferenceStore] = None
_resolution: Optional[PreferenceResolution] = None


def get_preference_store() -> PreferenceStore:
    global _store
    if _store is None:
        _store = PreferenceStore()
        _store.load()
    return _store


def get_preference_resolution(request_instruction: Optional[str] = None) -> PreferenceResolution:
    global _resolution
    _resolution = PreferenceResolution(
        store=get_preference_store(),
        request_instruction=request_instruction,
        user_style=_load_user_style(),
    )
    return _resolution


def _load_user_style() -> Optional[str]:
    """Charge le style d'écriture depuis les settings."""
    try:
        settings_path = Path("data/settings.json")
        if settings_path.exists():
            data = json.loads(settings_path.read_text(encoding="utf-8"))
            return data.get("userStyle") or data.get("emailStyle")
    except Exception:
        pass
    return None
