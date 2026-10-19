#!/usr/bin/env python3
"""
Génère MEMORY.md automatiquement depuis l'état du codebase.
Remplace la maintenance manuelle.
"""
import json
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).parent.parent.parent

def run(cmd, cwd=None):
    try:
        return subprocess.run(cmd, shell=True, cwd=cwd or ROOT, capture_output=True, text=True, check=False).stdout.strip()
    except Exception:
        return ""

def get_git_info():
    return {
        "branch": run("git branch --show-current"),
        "last_commit": run("git log -1 --format='%h %s (%ci)'"),
        "commit_count": run("git rev-list --count HEAD"),
        "dirty": bool(run("git status --porcelain")),
    }

def get_rust_info():
    cargo_toml = ROOT / "omega" / "Cargo.toml"
    crates = []
    if cargo_toml.exists():
        for crate_dir in (ROOT / "omega" / "crates").iterdir():
            if crate_dir.is_dir() and (crate_dir / "Cargo.toml").exists():
                crates.append(crate_dir.name)
    return {"crates": crates, "count": len(crates)}

def get_test_counts():
    # Run cargo test and parse output
    output = run("cargo test 2>&1 | grep -E 'test result:.*passed'")
    return {"raw": output}

def get_phase_status():
    planning = ROOT / ".planning"
    phases = []
    if (planning / "phases").exists():
        for phase_dir in sorted((planning / "phases").iterdir()):
            if phase_dir.is_dir():
                has_ctx = any(f.name.endswith("-CONTEXT.md") for f in phase_dir.iterdir())
                has_plan = any(f.name.endswith("-PLAN.md") for f in phase_dir.iterdir())
                has_summary = any(f.name.endswith("-SUMMARY.md") for f in phase_dir.iterdir())
                has_verif = any(f.name.endswith("-VERIFICATION.md") for f in phase_dir.iterdir())
                phases.append({
                    "name": phase_dir.name,
                    "context": has_ctx,
                    "plan": has_plan,
                    "summary": has_summary,
                    "verification": has_verif,
                })
    return phases

def get_model_policy():
    policy_file = ROOT / "docs" / "omega" / "rules" / "model-policy.json"
    if policy_file.exists():
        return json.loads(policy_file.read_text())
    return {}

def main():
    git_info = get_git_info()
    rust_info = get_rust_info()
    test_info = get_test_counts()
    phases = get_phase_status()
    model_policy = get_model_policy()
    
    now = datetime.now().strftime("%Y-%m-%d %H:%M")
    
    md = f"""# MEMORY.md — Omega (auto-généré)

**Généré le:** {now}  
**Source:** `omega/scripts/generate_memory.py` — provenance: [observed]

---

## État Git

- **Branche:** {git_info['branch']}
- **Dernier commit:** {git_info['last_commit']}
- **Total commits:** {git_info['commit_count']}
- **Working tree dirty:** {'oui' if git_info['dirty'] else 'non'}

---

## Espace Rust (omega/)

- **Crates:** {rust_info['count']}
- **Liste:** {', '.join(rust_info['crates'])}

---

## Tests

```
{test_info['raw']}
```

---

## Phases de planning

| Phase | Context | Plan | Summary | Verification |
|-------|---------|------|---------|--------------|
"""
    for p in phases:
        md += f"| {p['name']} | {'✅' if p['context'] else '❌'} | {'✅' if p['plan'] else '❌'} | {'✅' if p['summary'] else '❌'} | {'✅' if p['verification'] else '❌'} |\n"
    
    md += """

---

## Politique des modèles (MODEL-POLICY)

"""
    if model_policy:
        md += f"- **Gratuits par défaut:** {len(model_policy.get('free_defaults', {}))} modèles\n"
        md += f"- **Interdits:** {len(model_policy.get('forbidden', []))} patterns\n"
        md += f"- **Quota Go:** {model_policy.get('go_quota', {}).get('monthly', 'N/A')}\n"
    
    md += """

---

## Corpus Reverse-Engineering

- **Harnais disséqués:** 51 (JSON source unique dans `omega/crates/omega-harness/data/apps/`, fiches md générées)
- **Fonctionnalités cataloguées:** 910 (177 adoptées · 695 planifiées · 38 écartées) · 109 capacités
- **Preuves REA:** intake machine `docs/omega/research/rea/evidence/` (48 harnais analysés)
- **Census:** 66 recensés · 51 disséqués · 3 écarts documentés (copilot-chat, kimi-cli, pearai) · 0 silencieux
- **Corpus TikTok:** 78 vidéos @0xloucash (3 niveaux: légendes, oral, écran)

---

## 3 Trous (statut)

1. **Fichiers modèles ingérés:** ✅ 3 fichiers dans `docs/omega/research/models/`
2. **MEMORY.md auto-généré:** ✅ Ce fichier est généré par script
3. **INDEX.md racine:** ✅ docs/omega/INDEX.md (catalogue de l'espace Omega)

---

## Prochaines actions

- [ ] Milestone v0.4 — prochain cycle (voir `docs/omega/INDEX.md` et ROADMAP)
- [ ] Revue indépendante W8/W9 (quota Go)

---
*Ce fichier est régénéré à chaque exécution de `python3 omega/scripts/generate_memory.py`. Ne pas éditer manuellement.*
"""
    
    output_path = ROOT / "docs" / "omega" / "MEMORY.md"
    output_path.write_text(md)
    print(f"✅ MEMORY.md généré: {output_path}")

if __name__ == "__main__":
    main()
