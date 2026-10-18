#!/usr/bin/env python3
"""OMEGA-COMPLET — le combine unifie (deterministe, zero LLM).

Fusionne TOUT ce qui a ete reverse-engineere en UN document :

1. chaque capacite du systeme, avec **combien de harnais la reclament** et
   **lesquels** (pondere par la popularite mesuree — cf. `harnesses_census.json`) ;
2. l'**exemplaire de reference** par capacite : le harnais dont la dissection est
   la plus detaillee (le plus de `refs` verifiables), avec ses `file:line` ;
3. la **traduction Omega** : capacite -> port -> adaptateur -> cout -> phase ;
4. la **feuille de route** par phase (S = socle, M = structure, L = lourd) ;
5. les **non-objectifs** (fonctionnalites ecartees, avec raison) ;
6. la **provenance** (dissections + preuves REA).

Sortie : `docs/omega/reverse-eng/OMEGA-COMPLET.md`

Usage : python3 omega/scripts/omega_complet.py
"""
from __future__ import annotations

import glob
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APPS = sorted(glob.glob(str(ROOT / "crates/omega-harness/data/apps/*.json")))
CENSUS = ROOT / "crates/omega-harness/data/harnesses_census.json"
REA_EVIDENCE = ROOT.parent / "docs" / "omega" / "research" / "rea" / "evidence"
OUT = ROOT.parent / "docs/omega/reverse-eng/OMEGA-COMPLET.md"

COST_ORDER = {"S": 0, "M": 1, "L": 2}
PHASE = {"S": "Phase 1 — socle", "M": "Phase 2 — structure", "L": "Phase 3 — lourd"}

# Familles : capacite -> famille (pour la lecture)
FAMILIES = {
    "context-engine": "Contexte & memoire de travail",
    "memory-store": "Contexte & memoire de travail",
    "memory-search": "Contexte & memoire de travail",
    "skill-store": "Contexte & memoire de travail",
    "compaction-model": "Contexte & memoire de travail",
    "prompt-cache-policy": "Contexte & memoire de travail",
    "tool": "Outils & surface d'action",
    "tool-surface": "Outils & surface d'action",
    "mcp-bridge": "Outils & surface d'action",
    "sandbox-backend": "Execution & securite",
    "exec-policy": "Execution & securite",
    "permission-policy": "Execution & securite",
    "subagent-runner": "Orchestration",
    "scheduler-job": "Orchestration",
    "hook": "Orchestration",
    "channel": "Orchestration",
    "config-source": "Configuration & extension",
    "llm-provider": "Routage & modeles",
    "catalog-source": "Routage & modeles",
    "model-registry": "Routage & modeles",
    "task-classifier": "Routage & modeles",
    "router-policy": "Routage & modeles",
    "router-feedback": "Routage & modeles",
    "telemetry-sink": "Observabilite & tracabilite",
    "event-store": "Observabilite & tracabilite",
    "provenance-sink": "Observabilite & tracabilite",
    "ui-panel": "Observabilite & tracabilite",
    "harness-catalog": "Reverse-engineering",
}


def family_of(cap: str) -> str:
    """Famille d'une capacite : table explicite, puis regles de prefixe."""
    if cap in FAMILIES:
        return FAMILIES[cap]
    c = cap.strip("()").lower()
    rules = [
        (("context", "memory", "skill", "compaction", "prompt-cache", "conversation"),
         "Contexte & memoire de travail"),
        (("tool", "mcp", "mention", "harness-emulation"), "Outils & surface d'action"),
        (("sandbox", "exec", "permission", "worktree", "checkpoint", "approval"),
         "Execution & securite"),
        (("subagent", "agent", "scheduler", "channel", "hook", "lifecycle", "headless"),
         "Orchestration"),
        (("config", "registry", "extension", "plugin", "marketplace", "store"),
         "Configuration & extension"),
        (("llm", "catalog", "model", "task-classifier", "router", "provider"),
         "Routage & modeles"),
        (("telemetry", "event", "provenance", "ui", "trace", "observ"),
         "Observabilite & tracabilite"),
        (("harness", "dissection", "reverse"), "Reverse-engineering"),
    ]
    for prefixes, family in rules:
        if any(c.startswith(p) for p in prefixes):
            return family
    return "Autres"


def load_census() -> tuple[dict[str, int], bool]:
    """id -> etoiles, + drapeau « mesure reelle »."""
    if not CENSUS.is_file():
        return {}, False
    d = json.loads(CENSUS.read_text())
    stars = {c["id"]: c.get("stars", 0) for c in d.get("candidates", [])}
    return stars, bool(d.get("measured"))


def main() -> int:
    apps = [json.loads(Path(p).read_text()) for p in APPS]
    stars, measured = load_census()
    rea_done = set()
    if REA_EVIDENCE.is_dir():
        for pat in ("intake-*.summary.json", "harness-*.summary.json"):
            for p_ in Path(REA_EVIDENCE).glob(pat):
                rea_done.add(p_.name.split("-", 1)[-1].removesuffix(".summary.json"))

    groups: dict[str, dict] = defaultdict(
        lambda: {"apps": set(), "exemplars": [], "costs": [], "adapters": set(), "domains": set()}
    )
    rejected: list[tuple[str, str, str]] = []
    total = adopted = planned = 0

    for app in apps:
        for f in app["functionalities"]:
            total += 1
            st = f["status"]
            if st == "adopted":
                adopted += 1
                continue
            if st == "rejected":
                rejected.append((app["id"], f["name"], f.get("how", "")))
                continue
            planned += 1
            key = f.get("omega_capability") or f"({f['domain']})"
            g = groups[key]
            g["apps"].add(app["id"])
            g["costs"].append(f.get("rust_cost", "M"))
            g["domains"].add(f["domain"])
            if f.get("omega_adapter"):
                g["adapters"].add(f["omega_adapter"])
            g["exemplars"].append(
                {
                    "app": app["id"],
                    "name": f["name"],
                    "how": f.get("how", ""),
                    "port": f.get("omega_port"),
                    "adapter": f.get("omega_adapter"),
                    "cost": f.get("rust_cost", "M"),
                    "refs": f.get("refs", []),
                    "integration": f.get("integration"),
                }
            )

    rows = []
    for cap, g in groups.items():
        popularity = sum(stars.get(a, 0) for a in g["apps"])
        best = max(g["exemplars"], key=lambda e: (len(e["refs"]), len(e["how"])))
        rows.append(
            {
                "cap": cap,
                "family": family_of(cap),
                "apps": sorted(g["apps"]),
                "popularity": popularity,
                "min_cost": min(g["costs"], key=lambda c: COST_ORDER.get(c, 1)),
                "adapters": sorted(g["adapters"]),
                "best": best,
                "domains": sorted(g["domains"]),
            }
        )
    rows.sort(key=lambda r: (-len(r["apps"]), -r["popularity"], r["cap"]))

    # --- rapport -----------------------------------------------------------
    L: list[str] = []
    L.append("# OMEGA-COMPLET — le combine unifie")
    L.append("")
    L.append(
        "> Genere par `omega/scripts/omega_complet.py` (deterministe, regenerable). "
        "Fusionne **toutes** les dissections de harnais en un seul plan d'action."
    )
    L.append("")
    L.append("## 1. Synthese")
    L.append("")
    L.append(f"- harnais disseques : **{len(apps)}**")
    L.append(f"- fonctionnalites inventoriees : **{total}** (adoptees {adopted} · planifiees {planned} · ecartees {len(rejected)})")
    L.append(f"- capacites visees (regroupees) : **{len(rows)}**")
    L.append(
        "- popularite : "
        + ("**mesuree** (API GitHub, somme des etoiles des harnais demandeurs)"
           if measured else "non mesuree (recensement hors ligne)")
    )
    L.append(f"- preuves REA (intake machine) disponibles pour : {len(rea_done)} harnais")
    L.append("")
    L.append("### Ce que le consensus designe (>= 4 harnais demandeurs)")
    L.append("")
    L.append("| Capacite | Harnais | Popularite (etoiles) | Cout min | Famille |")
    L.append("|---|---|---|---|---|")
    for r in rows:
        if len(r["apps"]) >= 4:
            L.append(
                f"| `{r['cap']}` | {len(r['apps'])} | {r['popularity']:,} | {r['min_cost']} | {r['family']} |"
            )
    L.append("")
    L.append("## 2. Le combine, capacite par capacite")
    L.append("")
    by_family: dict[str, list[dict]] = defaultdict(list)
    for r in rows:
        by_family[r["family"]].append(r)
    for family in sorted(by_family):
        L.append(f"### {family}")
        L.append("")
        for r in by_family[family]:
            L.append(f"#### `{r['cap']}` — {len(r['apps'])} harnais, cout {r['min_cost']}")
            L.append("")
            L.append(f"- **harnais demandeurs** : {', '.join(r['apps'])}")
            if r["popularity"]:
                L.append(f"- **popularite cumulee** : {r['popularity']:,} etoiles")
            if r["adapters"]:
                L.append(
                    "- **adaptateurs proposes** : " + ", ".join(f"`{a}`" for a in r["adapters"])
                )
            b = r["best"]
            L.append(
                f"- **exemplaire de reference** : `{b['app']}` — *{b['name']}* "
                f"(port `{b['port'] or '—'}`"
                + (f", adaptateur `{b['adapter']}`" if b["adapter"] else "")
                + f", cout {b['cost']})"
            )
            L.append(f"  - comment il le fait : {b['how'][:400]}")
            if b["refs"]:
                L.append("  - preuves : " + ", ".join(f"`{x}`" for x in b["refs"][:4]))
            L.append("")
    # --- 2bis. Le noyau : architecture de plugins (ce que le corpus impose) ---
    CORE_PATTERNS = (
        "registry", "plugin", "seam", "config-source", "hook", "extension", "couture",
        "event-store", "provenance", "sandbox-backend", "exec-policy", "permission-policy",
        "subagent-runner", "tool-surface", "tool", "lifecycle", "agent-runtime", "agent-store",
    )
    core = [r for r in rows if any(pat in r["cap"].lower() for pat in CORE_PATTERNS)]
    core.sort(key=lambda r: (-len(r["apps"]), -r["popularity"]))
    L.append("## 2bis. Le noyau — ce que le corpus impose a l'architecture de plugins")
    L.append("")
    L.append(
        "Capacites qui touchent au **squelette** (registre, coutures, couches, effets, "
        "politiques, sous-agents). C'est la partie ou une erreur de conception se paie "
        "dans tout le systeme : a traiter **en premier**, avant les fonctionnalites."
    )
    L.append("")
    L.append("| Capacite | Harnais | Cout | Exemplaire de reference | Detail a copier |")
    L.append("|---|---|---|---|---|")
    for r in core:
        b = r["best"]
        how = b["how"].replace("|", "/")[:150]
        L.append(
            f"| `{r['cap']}` | {len(r['apps'])} | {r['min_cost']} | `{b['app']}` — *{b['name']}` | {how} |"
        )
    L.append("")
    L.append("### Regles de conception deduites (appliquees a toutes les capacites ci-dessus)")
    L.append("")
    for rule in [
        "**Effets reversibles** : toute inscription (registre, hook, timer, watcher) retourne un disposer ; rien ne fuit au unload.",
        "**Couches nommees** : toute valeur de configuration vient d'une couche identifiable et est tracable jusqu'a sa source.",
        "**Reload atomique** : un plugin se recharge entier ; les ecritures de config sont serialisees avec les reloads.",
        "**Guard monotone** : les politiques ne peuvent que restreindre, jamais elargir (pas de variante permissive).",
        "**Fail-closed** : a chaque couture, l'absence de canal, de provider ou de decision = refus, jamais execution.",
        "**Couture explicite** : chaque sous-systeme interchangeable est relie a une cle stable ; echanger le provider deplace tout le sous-systeme d'un coup.",
        "**Matrice d'evenements verifiee** : producteurs et consommateurs declares et controles automatiquement (pas d'evenement orphelin).",
    ]:
        L.append(f"- {rule}")
    L.append("")
    L.append("## 3. Feuille de route par phase")
    L.append("")
    for phase_cost in ("S", "M", "L"):
        caps = [r for r in rows if r["min_cost"] == phase_cost]
        if not caps:
            continue
        L.append(f"### {PHASE[phase_cost]} ({len(caps)} capacites)")
        L.append("")
        for r in caps:
            L.append(f"- `{r['cap']}` — {len(r['apps'])} harnais" + (f" · {r['popularity']:,} etoiles" if r["popularity"] else ""))
        L.append("")
    L.append("## 4. Non-objectifs (ecartes, avec raison)")
    L.append("")
    L.append("| Harnais | Fonctionnalite | Pourquoi ecartee |")
    L.append("|---|---|---|")
    for app, name, how in sorted(rejected):
        L.append(f"| `{app}` | {name} | {how[:200]} |")
    L.append("")
    L.append("## 5. Provenance")
    L.append("")
    L.append("- dissections : `omega/crates/omega-harness/data/apps/*.json` (schema `DISSECTION-SCHEMA.md`)")
    L.append("- popularite : `omega/crates/omega-harness/data/harnesses_census.json` (API GitHub)")
    L.append("- preuves machine : `docs/omega/research/rea/evidence/` (REA, avec limites declarees)")
    L.append("- ce document ne declare « adopte » que ce qui existe reellement dans le code")
    L.append("")

    OUT.write_text("\n".join(L) + "\n", encoding="utf-8")
    print(
        f"{OUT} : {len(rows)} capacites, {len(apps)} harnais, {total} fonctionnalites "
        f"({planned} planifiees), {len(rejected)} ecartees"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
