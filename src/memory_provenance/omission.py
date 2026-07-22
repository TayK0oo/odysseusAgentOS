"""
Omission rules — what NEVER gets stored in memory.

SFD §5.7.3 / P17: categories that put the user at risk if persisted.
Omission is total — never a placeholder, never a softened version.
"""

import re
from typing import Optional

# Categories that are NEVER persisted
PROTECTED_CATEGORIES = {
    "protected_attributes": [
        "ethnic origin", "ethnicity", "race", "color", "nationality",
        "caste", "religion", "age", "sex", "sexual orientation",
        "gender identity", "immigration status", "disability",
        "serious illness", "union affiliation",
    ],
    "sensitive_info": [
        "political beliefs", "abuse history", "socio-economic data",
        "health data", "diagnosis", "therapy", "addiction",
        "criminal record", "psychological profile",
    ],
    "identifiable_data": [
        "social security number", "SSN", "bank details",
        "personal address", "home address", "personal phone",
        "children's names", "children's ages", "children's health",
    ],
}

# Patterns for detecting protected content
_PROTECTED_PATTERNS = [
    re.compile(r"(?i)social security", re.IGNORECASE),
    re.compile(r"(?i)\bSSN\b", re.IGNORECASE),
    re.compile(r"(?i)bank account", re.IGNORECASE),
    re.compile(r"(?i)credit card", re.IGNORECASE),
    re.compile(r"(?i)home address", re.IGNORECASE),
    re.compile(r"(?i)sexual orientation", re.IGNORECASE),
    re.compile(r"(?i)religious belief", re.IGNORECASE),
    re.compile(r"(?i)mental health", re.IGNORECASE),
    re.compile(r"(?i)medical diagnosis", re.IGNORECASE),
]


class OmissionFilter:
    """Filters out protected content before any memory write.

    The test: "Would a colleague be uncomfortable seeing this
    in a settings page?" If yes → total omission.
    """

    @staticmethod
    def should_omit(text: str) -> bool:
        """Check if text contains protected content that must be omitted."""
        for pattern in _PROTECTED_PATTERNS:
            if pattern.search(text):
                return True
        return False

    @staticmethod
    def filter_entries(entries: list[str]) -> list[str]:
        """Remove entries containing protected content. Total omission."""
        return [e for e in entries if not OmissionFilter.should_omit(e)]

    @staticmethod
    def is_safe(text: str) -> bool:
        """Inverse of should_omit."""
        return not OmissionFilter.should_omit(text)
