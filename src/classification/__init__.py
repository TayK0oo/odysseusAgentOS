"""
Data Classification & Retention.

SFD §5.19: 5 retention levels (public, internal, personal, sensitive, protected).
Right to be forgotten: unit, file, or total deletion — definitive, irreversible.
"""

from src.classification.levels import RetentionLevel, classify_data
from src.classification.forgetting import RightToForget

__all__ = ["RetentionLevel", "classify_data", "RightToForget"]
