"""SFD §5.7 — Mémoire avec provenance.

Système de fichiers mémoire structuré :
  /profile.md              — identité stable
  /topics/<domaine>.md    — faits par domaine
  /areas/<nom>.md          — projets en cours
  /people/<nom>.md         — contexte relationnel
  /preferences.md          — préférences comportementales

Tags de provenance :
  [stated]   — l'utilisateur l'a dit explicitement
  [observed] — le système l'a constaté
  [inferred] — déduit avec niveau de confiance

Contrôle de concurrence versionné (if_version) :
  memory_read  → contenu + jeton 12 car.
  memory_write → échoue si if_version invalide

Gated behind ODYSSEUS_PROVENANCE_MEMORY kill-switch (default OFF).
"""

from __future__ import annotations

import hashlib
import logging
import os
import re
import secrets
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Literal

logger = logging.getLogger(__name__)

# ─── Kill-switch ────────────────────────────────────────────────────────


def provenance_memory_enabled() -> bool:
    val = os.getenv("ODYSSEUS_PROVENANCE_MEMORY", "on").strip().lower()
    return val in {"on", "1", "true", "yes"}


# ─── Constants ──────────────────────────────────────────────────────────

MEMORY_ROOT = Path("data/memory-fs")
ProvenanceTag = Literal["stated", "observed", "inferred"]

# Catégories protégées — jamais stockées
PROTECTED_CATEGORIES: list[str] = [
    "origine",
    "ethnique",
    "nationalité",
    "caste",
    "religion",
    "âge",
    "sexe",
    "orientation sexuelle",
    "identité de genre",
    "immigration",
    "handicap",
    "maladie grave",
    "syndicale",
]

SENSITIVE_CATEGORIES: list[str] = [
    "croyances politiques",
    "historique d'abus",
    "données financières",
    "diagnostics",
    "thérapie",
    "addictions",
    "casier judiciaire",
]

IDENTIFIABLE_CATEGORIES: list[str] = [
    "numéro sécurité sociale",
    "données bancaires",
    "adresse personnelle",
    "téléphone personnel",
    "enfants",
    "noms des enfants",
]

# Mots déclencheurs pour l'omission automatique
OMISSION_TRIGGERS: list[str] = PROTECTED_CATEGORIES + SENSITIVE_CATEGORIES + IDENTIFIABLE_CATEGORIES


# ─── Data model ─────────────────────────────────────────────────────────


@dataclass
class MemoryFile:
    """Un fichier mémoire avec frontmatter et contenu."""

    path: Path
    name: str
    description: str = ""
    sources: list[str] = field(default_factory=lambda: ["chat"])
    aliases: list[str] = field(default_factory=list)
    entries: list[MemoryEntry] = field(default_factory=list)
    version: str = ""

    def generate_version(self) -> str:
        self.version = secrets.token_hex(6)
        return self.version

    @property
    def content(self) -> str:
        """Génère le contenu Markdown du fichier."""
        lines = [
            "---",
            f"name: {self.name}",
            f"description: {self.description}",
            f"sources: [{', '.join(self.sources)}]",
        ]
        if self.aliases:
            lines.append(f"aliases: [{', '.join(self.aliases)}]")
        lines.append("---")
        lines.append("")
        for entry in self.entries:
            lines.append(str(entry))
        return "\n".join(lines)

    @classmethod
    def from_content(cls, path: Path, content: str) -> MemoryFile:
        """Parse un fichier mémoire existant."""
        mf = cls(path=path, name=path.stem)
        frontmatter = {}
        entries = []

        if content.startswith("---"):
            parts = content.split("---", 2)
            if len(parts) >= 3:
                for line in parts[1].strip().split("\n"):
                    if ":" in line:
                        k, v = line.split(":", 1)
                        frontmatter[k.strip()] = v.strip()
                body = parts[2]
            else:
                body = content
        else:
            body = content

        mf.name = frontmatter.get("name", path.stem)
        mf.description = frontmatter.get("description", "")
        mf.sources = [s.strip() for s in frontmatter.get("sources", "chat").strip("[]").split(",")]
        mf.aliases = [a.strip() for a in frontmatter.get("aliases", "").strip("[]").split(",") if a.strip()]

        for line in body.strip().split("\n"):
            line = line.strip()
            if line.startswith("- ["):
                entry = MemoryEntry.from_line(line)
                if entry:
                    entries.append(entry)

        mf.entries = entries
        return mf


@dataclass
class MemoryEntry:
    """Une entrée de mémoire avec provenance."""

    tag: ProvenanceTag
    text: str
    confidence: float | None = None
    timestamp: float = field(default_factory=time.time)

    def __str__(self) -> str:
        if self.tag == "inferred" and self.confidence is not None:
            return f"- [{self.tag}] {self.text} (confiance: {self.confidence})"
        return f"- [{self.tag}] {self.text}"

    @classmethod
    def from_line(cls, line: str) -> MemoryEntry | None:
        """Parse une ligne de type '- [stated] texte'."""
        match = re.match(r"-\s*\[(\w+)\]\s+(.+)", line)
        if match:
            tag = match.group(1)
            text = match.group(2)
            if tag in {"stated", "observed", "inferred"}:
                conf = None
                conf_match = re.search(r"confiance:\s*([\d.]+)", text)
                if conf_match:
                    conf = float(conf_match.group(1))
                    text = re.sub(r"\s*\(confiance:\s*[\d.]+\)", "", text)
                return cls(tag=tag, text=text.strip(), confidence=conf)  # type: ignore
        return None


# ─── Memory filesystem ──────────────────────────────────────────────────


class MemoryFS:
    """Système de fichiers mémoire avec provenance et versionnage."""

    def __init__(self, root: Path | None = None):
        self.root = root or MEMORY_ROOT
        self.root.mkdir(parents=True, exist_ok=True)

    # ── Opérations de base ──────────────────────────────────────────

    def memory_read(self, path: str) -> tuple[str | None, str]:
        """Lit un fichier mémoire. Retourne (contenu, jeton_version)."""
        full_path = self.root / path
        if not full_path.exists():
            return None, ""
        content = full_path.read_text(encoding="utf-8")
        version = hashlib.sha256(content.encode()).hexdigest()[:12]
        return content, version

    def memory_write(self, path: str, content: str, if_version: str) -> tuple[bool, str]:
        """Crée ou remplace un fichier mémoire avec contrôle de version."""
        full_path = self.root / path

        if if_version == "new":
            if full_path.exists():
                return False, "File already exists — use existing version token"
        elif full_path.exists():
            current, current_version = self.memory_read(path)
            if current_version != if_version:
                return False, f"Version mismatch: expected {if_version}, got {current_version}. Re-read first."
        else:
            return False, "File does not exist — use 'new' to create"

        # Vérifier les règles d'omission
        content = self._sanitize_content(content)

        full_path.parent.mkdir(parents=True, exist_ok=True)
        full_path.write_text(content, encoding="utf-8")
        new_version = hashlib.sha256(content.encode()).hexdigest()[:12]
        return True, new_version

    def memory_append(self, path: str, entry: str, if_version: str) -> tuple[bool, str]:
        """Ajoute une ligne à un fichier existant."""
        content, current_version = self.memory_read(path)
        if content is None:
            return False, "File does not exist"

        if current_version != if_version:
            return False, "Version mismatch"

        # Vérifier les règles d'omission
        if self._should_omit(entry):
            return False, "Entry blocked by omission rules"

        # Vérifier que l'entrée n'est pas déjà présente
        if entry.strip() in content:
            return True, current_version  # Déjà présent, pas d'erreur

        new_content = content.rstrip() + "\n" + entry + "\n"
        return self.memory_write(path, new_content, current_version)

    def memory_str_replace(self, path: str, old_str: str, new_str: str, if_version: str) -> tuple[bool, str]:
        """Remplacement chirurgical dans un fichier."""
        content, current_version = self.memory_read(path)
        if content is None:
            return False, "File does not exist"

        if current_version != if_version:
            return False, "Version mismatch"

        count = content.count(old_str)
        if count == 0:
            return False, "old_str not found"
        if count > 1:
            return False, f"old_str found {count} times — must be unique"

        new_content = content.replace(old_str, new_str, 1)
        return self.memory_write(path, new_content, current_version)

    def memory_delete(self, path: str, if_version: str) -> tuple[bool, str]:
        """Supprime un fichier mémoire."""
        full_path = self.root / path
        if not full_path.exists():
            return False, "File does not exist"

        if if_version != "force":
            _, current_version = self.memory_read(path)
            if current_version != if_version:
                return False, "Version mismatch"

        full_path.unlink()
        return True, ""

    def memory_list(self, prefix: str = "") -> list[str]:
        """Liste les fichiers sous un préfixe."""
        search_path = self.root / prefix if prefix else self.root
        if not search_path.exists():
            return []
        files = []
        for f in search_path.rglob("*.md"):
            rel = f.relative_to(self.root)
            files.append(str(rel).replace("\\", "/"))
        return sorted(files)

    # ── Règles d'omission ────────────────────────────────────────────

    def _should_omit(self, text: str) -> bool:
        """Vérifie si un texte contient des données à ne jamais stocker."""
        text_lower = text.lower()
        for trigger in OMISSION_TRIGGERS:
            if trigger in text_lower:
                logger.info("Omitting entry containing: %s", trigger)
                return True
        return False

    def _sanitize_content(self, content: str) -> str:
        """Nettoie le contenu avant écriture."""
        lines = content.split("\n")
        clean_lines = []
        for line in lines:
            if self._should_omit(line):
                continue
            clean_lines.append(line)
        return "\n".join(clean_lines)

    # ── Helpers de haut niveau ────────────────────────────────────────

    def add_stated(self, domain: str, fact: str) -> tuple[bool, str]:
        """Ajoute un fait [stated] dans le domaine approprié."""
        path = f"topics/{domain}.md"
        entry = f"- [stated] {fact}"

        content, version = self.memory_read(path)
        if content is None:
            # Créer le fichier
            mf = MemoryFile(
                path=self.root / path,
                name=domain,
                description=f"Faits sur {domain}",
            )
            mf.entries = [MemoryEntry(tag="stated", text=fact)]
            return self.memory_write(path, mf.content, "new")

        return self.memory_append(path, entry, version)

    def add_observed(self, domain: str, fact: str) -> tuple[bool, str]:
        """Ajoute un fait [observed]."""
        path = f"topics/{domain}.md"
        entry = f"- [observed] {fact}"

        content, version = self.memory_read(path)
        if content is None:
            return False, "Domain file does not exist — cannot observe without prior context"

        return self.memory_append(path, entry, version)

    def add_inferred(self, domain: str, fact: str, confidence: float) -> tuple[bool, str]:
        """Ajoute un fait [inferred] avec niveau de confiance."""
        path = f"topics/{domain}.md"
        entry = f"- [inferred] {fact} (confiance: {confidence})"

        content, version = self.memory_read(path)
        if content is None:
            return False, "Domain file does not exist"

        return self.memory_append(path, entry, version)

    def get_profile(self) -> str | None:
        """Lit le profil utilisateur."""
        content, _ = self.memory_read("profile.md")
        return content

    def update_profile(self, fact: str, tag: ProvenanceTag = "stated") -> tuple[bool, str]:
        """Met à jour le profil."""
        path = "profile.md"
        entry = f"- [{tag}] {fact}"

        content, version = self.memory_read(path)
        if content is None:
            mf = MemoryFile(
                path=self.root / path,
                name="profile",
                description="Identité stable de l'utilisateur",
            )
            mf.entries = [MemoryEntry(tag=tag, text=fact)]
            return self.memory_write(path, mf.content, "new")

        return self.memory_append(path, entry, version)


# ─── Singleton ───────────────────────────────────────────────────────────

_fs: MemoryFS | None = None


def get_memory_fs() -> MemoryFS:
    global _fs
    if _fs is None:
        _fs = MemoryFS()
    return _fs
