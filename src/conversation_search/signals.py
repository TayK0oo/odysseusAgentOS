"""
Linguistic signal detector for cross-session references.

SFD §5.16.2: detects implicit references to past conversations:
    - Possessive without context ("mon projet", "notre approche")
    - Definite article assuming shared reference ("le script", "cette stratégie")
    - Past-tense verb on prior exchange ("tu m'avais recommandé")
    - Direct request ("tu te souviens", "reprends où on en était")

Rule: never answer "I don't see a previous conversation" without searching.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
import re


class SignalType(str, Enum):
    POSSESSIVE = "possessive"
    DEFINITE_REFERENCE = "definite_reference"
    PAST_EXCHANGE = "past_exchange"
    DIRECT_REQUEST = "direct_request"
    NONE = "none"


@dataclass
class DetectedSignal:
    type: SignalType
    matched_text: str
    confidence: float  # 0.0 - 1.0


# French patterns
_FR_POSSESSIVE = re.compile(
    r"\b(mon|ma|mes|notre|nos|votre|vos)\s+(projet|approche|stratégie|script|code|solution|idée|plan|décision)\b",
    re.IGNORECASE,
)
_FR_DEFINITE = re.compile(
    r"\b(le|la|les|l'|cette|ce|cet|ces)\s+(script|stratégie|approche|solution|méthode|configuration|erreur|bug)\b",
    re.IGNORECASE,
)
_FR_PAST_EXCHANGE = re.compile(
    r"\b(tu\s+m['e]\w*\s+(recommandé|conseillé|dit|montré|expliqué|parlé|proposé)|on\s+(?:avait|a)\s+(décidé|choisi|fait|commencé|parlé|vu))\b",
    re.IGNORECASE,
)
_FR_DIRECT = re.compile(
    r"\b(tu\s+(te\s+)?souviens|rappelles?-toi|reprends?\s+où\s+on\s+en\s+était|la\s+dernière\s+fois|précédemment|tout\s+à\s+l'heure)\b",
    re.IGNORECASE,
)

# English patterns
_EN_POSSESSIVE = re.compile(
    r"\b(my|our|your)\s+(project|approach|strategy|script|code|solution|idea|plan|decision)\b",
    re.IGNORECASE,
)
_EN_DEFINITE = re.compile(
    r"\b(the|this|that)\s+(script|strategy|approach|solution|method|config(?:uration)?|error|bug)\b",
    re.IGNORECASE,
)
_EN_PAST_EXCHANGE = re.compile(
    r"\b(you\s+(?:had\s+)?(?:recommended|suggested|told|showed|explained|mentioned)|we\s+(?:had\s+)?(?:decided|chosen|discussed|started|talked))\b",
    re.IGNORECASE,
)
_EN_DIRECT = re.compile(
    r"\b(do\s+you\s+remember|recall|pick\s+up\s+where\s+we\s+left|last\s+time|previously|earlier)\b",
    re.IGNORECASE,
)


class LinguisticSignalDetector:
    """Detects implicit references to past conversations.

    Runs BEFORE answering to check if the user is referencing
    something from a previous session.
    """

    @staticmethod
    def detect(text: str) -> list[DetectedSignal]:
        """Detect all linguistic signals in the user's message."""
        signals: list[DetectedSignal] = []

        # French patterns
        for pattern, stype in [
            (_FR_POSSESSIVE, SignalType.POSSESSIVE),
            (_FR_DEFINITE, SignalType.DEFINITE_REFERENCE),
            (_FR_PAST_EXCHANGE, SignalType.PAST_EXCHANGE),
            (_FR_DIRECT, SignalType.DIRECT_REQUEST),
        ]:
            for match in pattern.finditer(text):
                signals.append(DetectedSignal(
                    type=stype,
                    matched_text=match.group(0),
                    confidence=0.9,
                ))

        # English patterns
        for pattern, stype in [
            (_EN_POSSESSIVE, SignalType.POSSESSIVE),
            (_EN_DEFINITE, SignalType.DEFINITE_REFERENCE),
            (_EN_PAST_EXCHANGE, SignalType.PAST_EXCHANGE),
            (_EN_DIRECT, SignalType.DIRECT_REQUEST),
        ]:
            for match in pattern.finditer(text):
                signals.append(DetectedSignal(
                    type=stype,
                    matched_text=match.group(0),
                    confidence=0.85,
                ))

        return signals

    @staticmethod
    def should_search(text: str) -> bool:
        """True if the text contains signals warranting a conversation search."""
        signals = LinguisticSignalDetector.detect(text)
        return len(signals) > 0

    @staticmethod
    def has_direct_request(text: str) -> bool:
        """True if user explicitly asks about past conversations."""
        signals = LinguisticSignalDetector.detect(text)
        return any(s.type == SignalType.DIRECT_REQUEST for s in signals)

    @staticmethod
    def extract_query(text: str) -> str:
        """Extract the most relevant search query from the user's message.

        Removes meta-words like "discuté", "hier", "remember" and keeps
        content words (nouns, proper names).
        """
        meta_words = [
            "discuté", "parlé", "hier", "avant", "précédent", "souviens",
            "discussed", "talked", "yesterday", "before", "remember", "recall",
        ]
        words = text.split()
        content_words = [w for w in words if w.lower().strip(",.?!") not in meta_words]
        return " ".join(content_words).strip()
