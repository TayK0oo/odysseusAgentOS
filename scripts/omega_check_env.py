#!/usr/bin/env python3
"""Valide la presence des credentials Omega dans .env — sans jamais afficher les valeurs.

Usage: python3 scripts/omega_check_env.py
Sortie: tableau (variable, requis?, present?, longueur) + code retour 0 si tout requis present.
"""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

REQUIRED = {
    "OPENCODE_API_KEY": "Routeur ZenRouter (moteur)",
    "GITHUB_TOKEN": "Clones / reverse-engineering",
    "HF_TOKEN": "Benchmarks / modeles",
}
OPTIONAL = {
    "TIKTOK_SESSIONID": "Scraping profond TikTok",
    "MODAL_TOKEN_ID": "Compute elastique",
    "MODAL_TOKEN_SECRET": "Compute elastique",
    "OPENAI_API_KEY": "Fournisseur LLM",
    "ANTHROPIC_API_KEY": "Fournisseur LLM",
    "GEMINI_API_KEY": "Fournisseur LLM",
    "GROQ_API_KEY": "Fournisseur LLM",
    "XAI_API_KEY": "Fournisseur LLM",
    "OPENROUTER_API_KEY": "Fournisseur LLM",
    "DEEPSEEK_API_KEY": "Fournisseur LLM",
    "OLLAMA_BASE_URL": "Self-hosting local",
    "LM_STUDIO_URL": "Self-hosting local",
}


def parse_env(path: Path) -> dict[str, str]:
    env: dict[str, str] = {}
    if not path.exists():
        return env
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        env[k.strip()] = v.strip().strip("'\"")
    return env


def main() -> int:
    env_path = ROOT / ".env"
    env = parse_env(env_path)
    print(f".env : {env_path}  ({'trouve' if env_path.exists() else 'ABSENT'})\n")

    missing_required = []
    print(f"{'VARIABLE':<22} {'REQ':<4} {'PRESENT':<8} {'LONG':<5} USAGE")
    print("-" * 70)
    for key, usage in {**REQUIRED, **OPTIONAL}.items():
        req = "oui" if key in REQUIRED else "-"
        val = env.get(key, "")
        present = "oui" if val else "non"
        print(f"{key:<22} {req:<4} {present:<8} {len(val):<5} {usage}")
        if key in REQUIRED and not val:
            missing_required.append(key)

    print()
    if missing_required:
        print(f"MANQUANT (requis) : {', '.join(missing_required)}")
        return 1
    print("OK : tous les credentials requis sont presents.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
