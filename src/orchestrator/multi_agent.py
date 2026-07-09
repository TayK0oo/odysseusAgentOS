"""MultiAgentWorkflow — orchestre un vrai pipeline multi-agent depuis un prompt.

Chaque phase canonique exécute son (ses) agent(s) dédié(s) avec les outils
appropriés. Le résultat de chaque agent est injecté dans le contexte de la
phase suivante. Le workflow traverse les 7 phases et produit un résultat final.

Utilisation :
    workflow = MultiAgentWorkflow(session_id, objective, model)
    async for event in workflow.run():
        yield SSE event
"""
from __future__ import annotations

import asyncio
import json
import logging
import os
from dataclasses import dataclass, field
from typing import AsyncGenerator, Dict, List, Optional

from src.orchestrator.phases import Phase, CANONICAL_SEQUENCE

logger = logging.getLogger(__name__)

# ── Tool sets par rôle ────────────────────────────────────────────────
# Chaque agent reçoit UNIQUEMENT les outils utiles à son rôle.
# Pas de bash pour le planner, pas de web_search pour l'exécuteur, etc.

TOOLS_BY_ROLE: Dict[str, List[str]] = {
    # CLASSIFY : lecture seule — classifier le risque
    "constitution": ["read_file", "grep", "glob", "ls", "web_search", "web_fetch"],

    # PLAN : recherche + planification — écriture limitée à .planning/
    "planner": ["read_file", "grep", "glob", "ls", "write_file", "web_search", "web_fetch"],
    "researcher": ["web_search", "web_fetch", "read_file", "grep", "glob", "ls"],

    # BUILD : implémentation complète
    "executor": ["bash", "python", "write_file", "edit_file", "read_file",
                  "grep", "glob", "ls", "web_search", "web_fetch"],
    "debugger": ["bash", "python", "read_file", "grep", "glob", "ls"],

    # QUALITY : review + audit
    "debate": ["read_file", "grep", "glob", "ls", "web_search"],
    "security": ["read_file", "grep", "glob", "ls", "bash", "web_search"],

    # AUTOEVAL : tests + edge cases
    "verifier": ["bash", "python", "read_file", "grep", "glob", "ls"],
    "edgecase": ["read_file", "grep", "glob", "web_search"],

    # MEMORY_OBSERVE : persistance
    "roadmapper": ["write_file", "read_file", "grep", "glob", "manage_memory", "update_plan"],
}

# ── Phase → agent(s) ──────────────────────────────────────────────────
PHASE_AGENTS: Dict[Phase, List[str]] = {
    Phase.CLASSIFY:        ["constitution"],
    Phase.KNOW:            [],  # mémoire native (CBM, RAG) — pas d'agent
    Phase.PLAN:            ["planner", "researcher"],
    Phase.BUILD:           ["executor"],
    Phase.QUALITY:         ["debate", "security"],
    Phase.AUTOEVAL:        ["verifier", "edgecase"],
    Phase.MEMORY_OBSERVE:  ["roadmapper"],
}

# ── System prompts par rôle ────────────────────────────────────────────
ROLE_PROMPTS: Dict[str, str] = {
    "constitution": (
        "Tu es l'agent Constitution. Ta mission : classifier le risque de "
        "l'objectif utilisateur selon les 10 invariants AgentOS. "
        "Produis un rapport concis : niveau de risque (LOW/MEDIUM/HIGH/CRITICAL), "
        "invariants concernés, et recommandations. Ne modifie rien."
    ),
    "planner": (
        "Tu es l'agent Planner. Ta mission : créer un plan d'exécution détaillé "
        "pour l'objectif. Décompose en étapes atomiques. Pour chaque étape, "
        "indique les fichiers à créer/modifier et les outils nécessaires. "
        "Écris le plan dans .planning/ si nécessaire."
    ),
    "researcher": (
        "Tu es l'agent Researcher. Ta mission : rechercher les meilleures "
        "approches, bibliothèques, patterns pour l'objectif. "
        "Fournis des recommandations concrètes avec des exemples de code. "
        "Utilise web_search et web_fetch pour trouver des solutions à jour."
    ),
    "executor": (
        "Tu es l'agent Executor. Ta mission : implémenter l'objectif étape par "
        "étape en suivant le plan. Utilise bash pour exécuter des commandes, "
        "write_file pour créer des fichiers, edit_file pour les modifier. "
        "À chaque étape, vérifie que le code fonctionne avant de passer à la suite."
    ),
    "debugger": (
        "Tu es l'agent Debugger. Ta mission : diagnostiquer et corriger les "
        "erreurs rencontrées par l'executor. Utilise la méthode scientifique : "
        "isole le problème, formule une hypothèse, teste, corrige."
    ),
    "debate": (
        "Tu es l'agent Debate (5 personas). Ta mission : challenger le code "
        "produit sous 5 angles (Architecture, Sécurité, Performance, UX, Devil's advocate). "
        "Produis un verdict GO/CAUTION/STOP avec les raisons pour chaque angle."
    ),
    "security": (
        "Tu es l'agent Security Audit. Ta mission : auditer le code selon "
        "STRIDE + OWASP Top 10. Pour chaque vulnérabilité trouvée, indique "
        "la sévérité et propose un fix concret. Ne modifie pas le code toi-même."
    ),
    "verifier": (
        "Tu es l'agent Verifier. Ta mission : exécuter les tests et vérifier "
        "que l'objectif est atteint. Lance pytest, npm test, ou les commandes "
        "appropriées. Si des tests échouent, documente précisément lesquels et pourquoi."
    ),
    "edgecase": (
        "Tu es l'agent Edge-Case Generator. Ta mission : générer des cas limites "
        "pour le code produit (entrées vides, très longues, nulles, négatives, "
        "concurrentes...). Produis une liste de scénarios de test."
    ),
    "roadmapper": (
        "Tu es l'agent Roadmapper. Ta mission : mettre à jour la roadmap du projet "
        "(.planning/ROADMAP.md, STATE.md) avec les résultats du run. "
        "Documente ce qui a été fait, les décisions prises, et les prochaines étapes."
    ),
}


@dataclass
class AgentResult:
    """Résultat d'un agent exécuté dans une phase."""
    agent: str
    role: str
    phase: str
    output: str = ""
    success: bool = True
    error: str = ""


@dataclass
class PhaseReport:
    """Rapport d'une phase complète (peut contenir plusieurs agents)."""
    phase: str
    agents: List[AgentResult] = field(default_factory=list)
    summary: str = ""


@dataclass
class WorkflowReport:
    """Rapport final du workflow complet."""
    objective: str
    phases: List[PhaseReport] = field(default_factory=list)
    final_output: str = ""
    success: bool = False


class MultiAgentWorkflow:
    """Orchestre un pipeline multi-agent complet à travers les 7 phases canoniques.

    Usage :
        wf = MultiAgentWorkflow(session_id, "Créer un script Python qui...")
        async for event in wf.run():
            yield f"data: {json.dumps(event)}\\n\\n"
    """

    def __init__(self, session_id: str, objective: str,
                 model: Optional[str] = None,
                 registry=None):
        self.session_id = session_id
        self.objective = objective
        self.model = model
        self._registry = registry
        self._context: List[dict] = []  # messages accumulés entre phases
        self._phase_results: Dict[str, PhaseReport] = {}

    # ── API publique ───────────────────────────────────────────────────

    async def run(self) -> AsyncGenerator[dict, None]:
        """Exécute le workflow complet et yield les événements SSE."""
        yield {"type": "workflow_start", "objective": self.objective,
               "phases": [p.value for p in CANONICAL_SEQUENCE]}

        report = WorkflowReport(objective=self.objective)

        for phase in CANONICAL_SEQUENCE:
            agent_names = PHASE_AGENTS.get(phase, [])
            if not agent_names:
                yield {"type": "phase_skip", "phase": phase.value,
                       "reason": "no agents assigned"}
                continue

            yield {"type": "phase_start", "phase": phase.value,
                   "agents": agent_names}

            phase_report = PhaseReport(phase=phase.value)

            # Lancer tous les agents de cette phase EN PARALLÈLE
            tasks = []
            for agent_name in agent_names:
                tasks.append(self._run_agent(agent_name, phase))

            results: List[AgentResult] = await asyncio.gather(*tasks, return_exceptions=True)

            for i, result in enumerate(results):
                if isinstance(result, Exception):
                    result = AgentResult(
                        agent=agent_names[i], role=agent_names[i],
                        phase=phase.value, success=False, error=str(result),
                    )
                phase_report.agents.append(result)
                yield {
                    "type": "agent_result",
                    "phase": phase.value,
                    "agent": result.agent,
                    "success": result.success,
                    "output_preview": result.output[:200] if result.output else "",
                    "error": result.error,
                }

            # Résumé de phase injecté dans le contexte
            phase_summary = self._summarize_phase(phase_report)
            phase_report.summary = phase_summary
            self._phase_results[phase.value] = phase_report

            # Injecter le résumé dans le contexte pour la phase suivante
            self._context.append({
                "role": "system",
                "content": f"[PHASE {phase.value} COMPLÉTÉE]\n{phase_summary}",
            })

            yield {"type": "phase_complete", "phase": phase.value,
                   "summary": phase_summary[:500]}

            report.phases.append(phase_report)

        # Phase finale : synthèse
        final = await self._synthesize(report)
        report.final_output = final
        report.success = True

        yield {"type": "workflow_complete", "final_output": final,
               "phases_completed": len(report.phases)}

    # ── Interne ────────────────────────────────────────────────────────

    async def _run_agent(self, agent_name: str, phase: Phase) -> AgentResult:
        """Exécute un agent avec ses outils dédiés."""
        role = agent_name
        tools = TOOLS_BY_ROLE.get(role, ["read_file", "grep", "glob", "ls"])
        system_prompt = ROLE_PROMPTS.get(role, f"Tu es l'agent {agent_name}.")

        try:
            # Charger le prompt complet depuis le fichier .md si dispo
            full_prompt = self._load_agent_prompt(agent_name)
            if full_prompt:
                system_prompt = full_prompt

            # Construire les messages pour cet agent
            messages = [
                {"role": "system", "content": system_prompt},
                # Contexte accumulé des phases précédentes
                *self._context[-6:],  # garder les 6 derniers messages de contexte
                {"role": "user", "content": (
                    f"OBJECTIF : {self.objective}\n\n"
                    f"Phase actuelle : {phase.value}\n"
                    f"Outils disponibles : {', '.join(tools)}\n"
                    f"Exécute ta mission pour cette phase. Sois concis et actionnable."
                )},
            ]

            # Appeler le LLM avec les outils
            result_text = await self._call_llm_with_tools(messages, tools)

            return AgentResult(
                agent=agent_name, role=role, phase=phase.value,
                output=result_text, success=True,
            )

        except Exception as exc:
            logger.warning("[MultiAgent] %s failed: %s", agent_name, exc)
            return AgentResult(
                agent=agent_name, role=role, phase=phase.value,
                success=False, error=str(exc),
            )

    async def _call_llm_with_tools(self, messages: list, tool_names: list) -> str:
        """Appel LLM avec exécution réelle d'outils (jusqu'à 5 tours).

        L'agent peut utiliser ses outils (bash, write_file, read_file, etc.)
        comme dans le chat normal. Les résultats sont réinjectés dans la
        conversation pour les tours suivants.
        """
        from src.tool_schemas import FUNCTION_TOOL_SCHEMAS
        from src.task_endpoint import resolve_task_candidates
        from src.tool_execution import execute_tool_block
        from src.agent_tools import ToolBlock

        candidates = resolve_task_candidates(owner=self.session_id)
        if not candidates:
            return "[Erreur: aucun endpoint LLM disponible]"

        # Filtrer les schémas d'outils pour n'inclure que ceux autorisés
        allowed_tools = set(tool_names)
        tool_schemas = [
            s for s in FUNCTION_TOOL_SCHEMAS
            if (s.get("function", {}).get("name") or s.get("name", "")) in allowed_tools
        ]

        accumulated_output: list[str] = []
        max_turns = 5
        turn_messages = list(messages)  # copie modifiable

        for turn in range(max_turns):
            url, model, headers = candidates[0]

            # Appeler le LLM avec les outils
            try:
                raw = await self._call_llm_raw(
                    url, model, headers, turn_messages, tool_schemas
                )
            except Exception as exc:
                logger.warning("[MultiAgent] LLM call failed turn %d: %s", turn, exc)
                break

            if not raw:
                break

            # Extraire le contenu texte et les tool_calls
            content = ""
            tool_calls = []

            if isinstance(raw, dict):
                content = raw.get("content") or ""
                # Format OpenAI : choices[0].message.tool_calls
                choices = raw.get("choices", [])
                if choices:
                    msg = choices[0].get("message", {})
                    content = content or msg.get("content") or ""
                    tool_calls = msg.get("tool_calls") or []

            if isinstance(raw, str):
                content = raw

            # Si pas de tool calls, c'est la réponse finale
            if not tool_calls:
                if content:
                    accumulated_output.append(content)
                break

            # Ajouter la réponse de l'assistant aux messages
            assistant_msg = {"role": "assistant", "content": content or None}
            if tool_calls:
                assistant_msg["tool_calls"] = tool_calls
            turn_messages.append(assistant_msg)

            # Exécuter chaque outil
            for tc in tool_calls:
                fn = tc.get("function", {})
                tool_name = fn.get("name", "")
                try:
                    tool_args = json.loads(fn.get("arguments", "{}"))
                except json.JSONDecodeError:
                    tool_args = {}

                # Construire le contenu pour execute_tool_block
                # Format attendu par _MCP_ARG_PARSERS dans tool_execution.py :
                #   bash:        "command"
                #   write_file:  "path\ncontent"
                #   read_file:   "path"
                #   web_search:  "query"
                #   etc.
                if tool_name == "write_file":
                    path = tool_args.get("path", "")
                    content = tool_args.get("content", "")
                    tool_content = f"{path}\n{content}"
                elif tool_name == "edit_file":
                    path = tool_args.get("path", "")
                    old = tool_args.get("old_string") or tool_args.get("oldText") or ""
                    new = tool_args.get("new_string") or tool_args.get("newText") or ""
                    tool_content = json.dumps({"path": path, "old_string": old, "new_string": new})
                elif tool_name == "bash":
                    tool_content = tool_args.get("command", "")
                elif tool_name == "python":
                    tool_content = tool_args.get("code", "")
                elif tool_name in ("web_search", "web_fetch"):
                    tool_content = tool_args.get("query") or tool_args.get("url", "")
                elif tool_name in ("read_file", "grep", "glob", "ls"):
                    tool_content = tool_args.get("path") or tool_args.get("pattern") or tool_args.get("query") or ""
                elif tool_name == "manage_memory":
                    tool_content = json.dumps(tool_args)
                else:
                    tool_content = json.dumps(tool_args)

                logger.info(
                    "[MultiAgent] tool call: %s args=%.100s",
                    tool_name, str(tool_args)[:100],
                )

                try:
                    block = ToolBlock(tool_type=tool_name, content=tool_content)
                    desc, result = await execute_tool_block(
                        block,
                        session_id=self.session_id,
                        owner=self.session_id,
                    )
                    tool_output = json.dumps(result) if isinstance(result, dict) else str(result)
                    accumulated_output.append(
                        f"[OUTIL {tool_name}] {desc}\n{tool_output[:500]}"
                    )
                    logger.info(
                        "[MultiAgent] tool executed: %s → %.100s",
                        tool_name, desc,
                    )
                except Exception as tool_exc:
                    logger.warning(
                        "[MultiAgent] tool %s failed: %s", tool_name, tool_exc,
                    )
                    tool_output = f"Erreur: {tool_exc}"
                    accumulated_output.append(f"[OUTIL {tool_name}] ERREUR: {tool_exc}")

                # Ajouter le résultat au tour de messages
                turn_messages.append({
                    "role": "tool",
                    "tool_call_id": tc.get("id", f"call_{turn}_{tool_name}"),
                    "content": tool_output,
                })

        return "\n\n".join(accumulated_output) if accumulated_output else "[Aucune réponse]"

    async def _call_llm_raw(self, url: str, model: str, headers: dict,
                            messages: list, tools: list) -> dict:
        """Appel LLM brut avec outils. Retourne la réponse brute (dict ou str)."""
        import httpx

        payload = {
            "model": model,
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 4096,
        }
        if tools:
            payload["tools"] = tools
            payload["tool_choice"] = "auto"

        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                url,
                json=payload,
                headers={**headers, "Content-Type": "application/json"},
            )
            if resp.status_code != 200:
                logger.warning(
                    "[MultiAgent] LLM error %d: %.200s",
                    resp.status_code, resp.text[:200],
                )
                return None
            return resp.json()

    def _load_agent_prompt(self, agent_name: str) -> Optional[str]:
        """Charge le prompt complet depuis le fichier .opencode/agents/{name}.md."""
        try:
            from pathlib import Path
            agents_dir = Path(__file__).parent.parent.parent / ".opencode" / "agents"
            md_file = agents_dir / f"{agent_name}.md"
            if md_file.exists():
                from src.orchestrator.spec import parse_agent_spec
                spec = parse_agent_spec(md_file)
                return spec.prompt or spec.description
        except Exception:
            pass
        return None

    def _summarize_phase(self, report: PhaseReport) -> str:
        """Produit un résumé concis d'une phase."""
        parts = []
        for agent in report.agents:
            status = "✅" if agent.success else "❌"
            preview = agent.output[:300] if agent.output else agent.error
            parts.append(f"{status} {agent.agent}: {preview}")
        return "\n".join(parts)

    async def _synthesize(self, report: WorkflowReport) -> str:
        """Produit une synthèse finale du workflow complet."""
        parts = [f"# Workflow terminé : {self.objective}\n"]
        for pr in report.phases:
            parts.append(f"\n## Phase {pr.phase}")
            parts.append(pr.summary)
        return "\n".join(parts)
