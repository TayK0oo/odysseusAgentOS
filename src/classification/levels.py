"""
Retention levels and data classification (SFD §5.19).

5 levels:
    PUBLIC    — non-sensitive, shareable, unlimited retention
    INTERNAL  — work data, workspace lifetime + 90 days
    PERSONAL  — non-sensitive personal, unlimited (right to forget)
    SENSITIVE — moderate risk, session only
    PROTECTED — high risk, NEVER persisted
"""

from __future__ import annotations

from datetime import datetime, timezone, timedelta
from enum import Enum
from typing import Optional


class RetentionLevel(str, Enum):
    PUBLIC = "public"
    INTERNAL = "internal"
    PERSONAL = "personal"
    SENSITIVE = "sensitive"
    PROTECTED = "protected"

    @property
    def retention_days(self) -> int:
        """Days before data is eligible for deletion. -1 = unlimited."""
        return {
            RetentionLevel.PUBLIC: -1,
            RetentionLevel.INTERNAL: 90,
            RetentionLevel.PERSONAL: -1,
            RetentionLevel.SENSITIVE: 1,
            RetentionLevel.PROTECTED: 0,
        }[self]

    @property
    def can_persist(self) -> bool:
        """Whether data at this level can be persisted at all."""
        return self != RetentionLevel.PROTECTED

    @property
    def session_only(self) -> bool:
        """Whether data lives only for the current session."""
        return self == RetentionLevel.SENSITIVE


CLASSIFICATION_RULES = {
    RetentionLevel.PUBLIC: [
        "format preference", "public skill", "language setting",
        "tool configuration", "workspace name",
    ],
    RetentionLevel.INTERNAL: [
        "project plan", "execution history", "build log",
        "test result", "code artifact", "deployment config",
    ],
    RetentionLevel.PERSONAL: [
        "first name", "role", "company", "food preference",
        "schedule", "hobby", "expertise level", "interest",
    ],
    RetentionLevel.SENSITIVE: [
        "approximate location", "current project status",
        "temporary auth token",
    ],
    RetentionLevel.PROTECTED: [
        "health data", "religion", "sexual orientation",
        "bank details", "SSN", "personal address",
        "children information", "criminal record",
    ],
}


def classify_data(subject: str, content: str = "") -> RetentionLevel:
    """Classify a piece of data into a retention level.

    Checks PROTECTED first (highest priority), then SENSITIVE, etc.
    """
    combined = (subject + " " + content).lower()

    # Check PROTECTED first — anything matching here is NEVER persisted
    for indicator in CLASSIFICATION_RULES[RetentionLevel.PROTECTED]:
        if indicator in combined:
            return RetentionLevel.PROTECTED

    for indicator in CLASSIFICATION_RULES[RetentionLevel.SENSITIVE]:
        if indicator in combined:
            return RetentionLevel.SENSITIVE

    for indicator in CLASSIFICATION_RULES[RetentionLevel.PERSONAL]:
        if indicator in combined:
            return RetentionLevel.PERSONAL

    for indicator in CLASSIFICATION_RULES[RetentionLevel.INTERNAL]:
        if indicator in combined:
            return RetentionLevel.INTERNAL

    return RetentionLevel.PUBLIC
