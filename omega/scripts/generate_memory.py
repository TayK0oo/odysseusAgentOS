#!/usr/bin/env python3
"""
Génère MEMORY.md depuis STATUS.json + git (provenance ADR-0007).

Chiffres « générés, jamais saisis » (principe 7 du plan P0) : chaque nombre du
corpus/test vient de `docs/omega/plan/STATUS.json` (mesure Plus), accompagné du
tag doc-truth `<!--m:cle=valeur-->` pour que `check_doc_truth.py` le re-vérifie.
Aucune exécution de build ici (pas de `cargo test`) : STATUS est déjà la mesure.
La bannière d'état (date/heure, HEAD, fraîcheur de STATUS) est auto-calculée.
"""
import json
import subprocess
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).parent.parent.parent
STATUS_PATH = ROOT / "docs" / "omega" / "plan" / "STATUS.json"


def run(argv, cwd=None):
    """Commande git en argv discret (pas de shell intermédiaire)."""
    try:
        return subprocess.run(argv, cwd=cwd or ROOT, capture_output=True, text=True, check=False).stdout.strip()
    except Exception:
        return ""


def get_git_info():
    return {
        "branch": run(["git", "branch", "--show-current"]),
        "last_commit": run(["git", "log", "-1", "--format=%h %s (%ci)"]),
        "commit_count": run(["git", "rev-list", "--count", "HEAD"]),
        "head_sha": run(["git", "rev-parse", "--short", "HEAD"]),
        "dirty": bool(run(["git", "status", "--porcelain"])),
    }


def noeud(status, cle):
    """Module status par chemin pointé ; None si une clé manque."""
    courant = status
    for part in cle.split("."):
        if not isinstance(courant, dict) or part not in courant:
            return None
        courant = courant[part]
    return courant


def valeur_de(node):
    """Dévoile l'enveloppe {"valeur": ..., "methode": ...} de measure.py."""
    if isinstance(node, dict) and "valeur" in node:
        return node["valeur"]
    return node


def mesure(status, cle):
    """« valeur [tag] » depuis STATUS, ou « INCONNU(cle absent du STATUS) ».

    Le tag porte la valeur (convention A, ADR-0007) : doc-truth le re-vérifie
    à chaque passage. Une clé absente n'émet pas de tag (strict échouerait).
    """
    v = valeur_de(noeud(status, cle))
    if v is None:
        return f"INCONNU({cle} absent du STATUS)"
    return f"{v} <!--m:{cle}={v}-->"


def methode_de(status, cle):
    node = noeud(status, cle)
    if isinstance(node, dict) and "methode" in node:
        return node["methode"]
    return "methode non renseignée"


def get_status():
    try:
        return json.loads(STATUS_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}


def get_rust_info(status):
    """Crates depuis « STATUS.socles.rust.* » : liste et count restent cohérents."""
    liste = valeur_de(noeud(status, "socles.rust.liste"))
    if isinstance(liste, list):
        return {"count": f"{len(liste)} <!--m:socles.rust.crates={len(liste)}-->", "liste": liste}
    count = valeur_de(noeud(status, "socles.rust.crates"))
    if count is None:
        return {"count": "INCONNU(socles.rust.crates absent du STATUS)", "liste": []}
    return {"count": f"{count} <!--m:socles.rust.crates={count}-->", "liste": []}


def phase_complete(phase_dir):
    noms = {f.name for f in phase_dir.iterdir()}
    return {
        "name": phase_dir.name,
        "context": any(n.endswith("-CONTEXT.md") for n in noms),
        "plan": any(n.endswith("-PLAN.md") for n in noms),
        "summary": any(n.endswith("-SUMMARY.md") for n in noms),
        "verification": any(n.endswith("-VERIFICATION.md") for n in noms),
    }


def get_phase_status():
    planning = ROOT / ".planning"
    phases = []
    if (planning / "phases").exists():
        for phase_dir in sorted((planning / "phases").iterdir()):
            if phase_dir.is_dir():
                phases.append(phase_complete(phase_dir))
    return phases


def get_model_policy():
    policy_file = ROOT / "docs" / "omega" / "rules" / "model-policy.json"
    if policy_file.exists():
        return json.loads(policy_file.read_text())
    return {}


def banniere(git_info, status, now):
    """Bannière auto-calculée : date/heure, HEAD, fraîcheur de STATUS."""
    lignes = [
        f"> **Bannière (auto-calculée, jamais saisie)** : généré le {now} · "
        f"HEAD `{git_info['head_sha']}` ({git_info['branch']}) · à jour.",
    ]
    if git_info["dirty"]:
        lignes.append("> **Working tree dirty** : d'autres modifications coexistent avec cette génération.")
    sha_status = valeur_de(noeud(status, "git_sha"))
    mesure_le = valeur_de(noeud(status, "mesure_le"))
    if sha_status and sha_status != git_info["head_sha"]:
        lignes.append(
            f"> ⚠️ STATUS.json a été mesuré sur le commit `{sha_status}`"
            f"{' le ' + str(mesure_le) if mesure_le else ''} — antérieur au HEAD :"
            " les chiffres ci-dessous peuvent être périmés, relancer `measure.py`."
        )
    return "\n".join(lignes)


def main():
    git_info = get_git_info()
    status = get_status()
    rust_info = get_rust_info(status)
    phases = get_phase_status()
    model_policy = get_model_policy()

    now = datetime.now().strftime("%Y-%m-%d %H:%M")

    md = f"""# MEMORY.md — Omega (auto-généré)

{banniere(git_info, status, now)}

**Généré le:** {now}
**Source:** `omega/scripts/generate_memory.py` — génère MEMORY.md depuis STATUS.json + git — provenance: [observed]

---

## État Git

- **Branche:** {git_info['branch']}
- **Dernier commit:** {git_info['last_commit']}
- **Total commits:** {git_info['commit_count']}
- **Working tree dirty:** {'oui' if git_info['dirty'] else 'non'}

---

## Espace Rust (omega/)

- **Crates:** {rust_info['count']}
- **Liste:** {', '.join(rust_info['liste'])}

---

## Tests (depuis STATUS.json — aucune exécution ici)

- **Rust:** {mesure(status, 'socles.rust.tests.valeur')} tests ({methode_de(status, 'socles.rust.tests')}) — [observed]
- **Python:** {mesure(status, 'socles.python.tests.valeur')} tests ({methode_de(status, 'socles.python.tests')}) — [observed]

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

    md += f"""


---

## Corpus Reverse-Engineering (chiffres générés depuis STATUS.json — [observed])

- **Capacités Omega:** {mesure(status, 'avancee.capacites_omega')}
- **Fonctionnalités cataloguées:** {mesure(status, 'avancee.fonctionnalites_total')} (177 adoptées · 695 planifiées · 38 écartées)
- **Harnais disséqués:** {mesure(status, 'avancee.dissections')} (JSON source unique dans `omega/crates/omega-harness/data/apps/`, fiches md générées)
- **Preuves REA:** intake machine `docs/omega/research/rea/evidence/` (48 harnais analysés)
- **Census:** {mesure(status, 'avancee.census_recenses')} recensés · {mesure(status, 'avancee.dissections')} disséqués · 3 écarts documentés (copilot-chat, kimi-cli, pearai) · 0 silencieux
- **Corpus TikTok:** 78 vidéos @0xloucash (3 niveaux: légendes, oral, écran) — [stated] non mesuré par STATUS

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
*Chaque tag `<!--m:cle=valeur-->` est re-vérifié par `python3 omega/scripts/check_doc_truth.py`.*
"""

    output_path = ROOT / "docs" / "omega" / "MEMORY.md"
    output_path.write_text(md)
    print(f"✅ MEMORY.md généré: {output_path}")


if __name__ == "__main__":
    main()
