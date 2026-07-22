"""
Output content filter (SFD §5.20.2).

Prohibited output categories: graphic violence, gore, eating disorder
content, self-harm, sexual/suggestive, copyrighted characters,
identifiable real people, artwork reproductions, factual disinformation.
"""

from __future__ import annotations

import re
from typing import Optional

PROHIBITED_CATEGORIES = {
    "violence": ["graphic violence", "gore", "torture", "mutilation"],
    "self_harm": ["eating disorder", "self-harm", "suicide method"],
    "sexual": ["sexual content", "suggestive content", "explicit"],
    "copyright": ["copyrighted character", "trademarked character"],
    "real_people": ["identifiable person", "real person depiction"],
    "art_theft": ["artwork reproduction", "copy of existing art"],
    "disinformation": ["factual disinformation", "false claim"],
}


class OutputFilter:
    """Filters generated output for prohibited content.

    The system must NEVER generate content in these categories.
    This is a hard constraint, not a suggestion.
    """

    @staticmethod
    def check(text: str) -> Optional[str]:
        """Check text for prohibited content.

        Returns the category if found, None if safe.
        """
        text_lower = text.lower()

        # Violence / gore
        if any(w in text_lower for w in ["graphic violence", "gore", "torture", "mutilation"]):
            return "violence"

        # Self-harm / eating disorders
        if any(w in text_lower for w in ["eating disorder", "anorexia", "bulimia", "self-harm"]):
            return "self_harm"

        # Sexual content
        if any(w in text_lower for w in ["sexual content", "pornography", "explicit sexual"]):
            return "sexual"

        # Copyrighted characters
        if re.search(r"(?i)(mickey mouse|superman|spider-man|batman|pikachu|mario|sonic)\b", text):
            return "copyright"

        # Real identifiable people
        if re.search(r"(?i)(president|celebrity|politician).*(portrait|photo|depiction)", text):
            return "real_people"

        return None

    @staticmethod
    def is_safe(text: str) -> bool:
        """True if the text contains no prohibited content."""
        return OutputFilter.check(text) is None

    @staticmethod
    def filter_block(response: str) -> tuple[str, bool]:
        """Check and potentially block a response.

        Returns (response_or_empty, was_blocked).
        """
        violation = OutputFilter.check(response)
        if violation:
            return "", True
        return response, False
