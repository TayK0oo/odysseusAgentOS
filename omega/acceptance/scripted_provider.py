#!/usr/bin/env python3
"""Faux LLM OpenAI-compatible, réponses déterministes.

Adaptation honnête vs énoncé P0-4 : le plan prévoyait un faux Ollama natif
(`/api/chat`), mais le provider réel (`omega-providers/src/ollama.rs:108`)
appelle l'endpoint **OpenAI-compatible** `POST {base}/v1/chat/completions`
et le catalogue `GET {base}/api/tags`. On scripte le contrat réel, sinon
le faux serveur ne serait jamais consulté (leçon L2 : vérifier le flow).

Le wire ne transporte PAS de tool_calls (`WireMessage {role, content}`) —
aucun scénario scripté ne peut donc faire passer l'agent pour un expert outils.
"""
from __future__ import annotations

import json
import threading
import time
import urllib.request
from http.server import BaseHTTPRequestHandler, HTTPServer


class _Handler(BaseHTTPRequestHandler):
    server_version = "ScriptedLLM/0.1"

    def log_message(self, *a):  # silence
        pass

    def do_GET(self):
        if self.path == "/api/tags":
            self._json({"models": [{"name": "scripted:latest"},
                                    {"name": "scripted-tools:latest"}]})
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path != "/v1/chat/completions":
            self.send_error(404)
            return
        n = int(self.headers.get("Content-Length", "0"))
        req = json.loads(self.rfile.read(n) or b"{}")
        self.server.requests.append(req)  # journal des requêtes reçues (preuve)
        idx = self.server.calls
        self.server.calls += 1
        if idx >= len(self.server.sequence):
            self._json({"error": f"scripted: séquence épuisée (appel {idx})"})
            return
        self._json(self.server.sequence[idx](req))

    def _json(self, obj):
        data = json.dumps(obj).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


class ScriptedProvider:
    """Serveur LLM factice. Séquence = liste de callables (req → dict wire)."""

    def __init__(self, sequence: list):
        self.sequence = sequence
        self.requests: list[dict] = []  # journal des requêtes reçues
        self._srv: HTTPServer | None = None
        self._thread: threading.Thread | None = None
        self.port = 0

    def start(self) -> int:
        self._srv = HTTPServer(("127.0.0.1", 0), _Handler)
        self._srv.calls = 0
        self._srv.sequence = self.sequence
        # journal des requêtes : le handler écrit dans server.requests
        self._server_requests = self._srv.requests = []
        self.port = self._srv.server_address[1]
        self._thread = threading.Thread(target=self._srv.serve_forever, daemon=True)
        self._thread.start()
        for _ in range(50):
            try:
                urllib.request.urlopen(
                    f"http://127.0.0.1:{self.port}/api/tags", timeout=0.2)
                return self.port
            except Exception:
                time.sleep(0.05)
        raise RuntimeError("ScriptedProvider n'a pas démarré")

    def stop(self):
        if self._srv:
            self._srv.shutdown()
        if self._thread:
            self._thread.join(timeout=2)

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self.port}"


# ---------------------------------------------------------------------------
# Fabriques de séquences (format wire OPENAI-COMPATIBLE, cf. ollama.rs)
# ---------------------------------------------------------------------------

def _reply(content: str) -> dict:
    return {"choices": [{"message": {"role": "assistant", "content": content}}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 5}}


def seq_text_reply(text: str):
    """Un appel ; le modèle répond directement, sans margin d'outils."""
    return [lambda req: _reply(text)]


def seq_echoreq_reply(prefix: str = ""):
    """Répond avec le contenu de la dernière requête — utile au contrôle négatif."""
    def one(req):
        msgs = req.get("messages", [])
        last = msgs[-1].get("content", "") if msgs else ""
        return _reply(f"{prefix}{last[-200:]}")
    return [one]


def seq_react_toolcall(tool: str, tours: int = 12):
    """Le modèle propose tours fois de suite un appel d'outil, en forme texte
    ReAct canonique (ADR-0006 : `{"tool": "...", "arguments": {...}}`).
    Base du scénario S2 : mesurer qu'un budget de steps/fini par arrêt.
    Aucun marqueur de budget dans la prose — un arrêt doit venir du MOTEUR.
    """
    rep = _reply(json.dumps({"tool": tool, "arguments": {}}, sort_keys=True))
    return [lambda req: rep for _ in range(tours)]


def seq_tool_then_echo(tool: str, arguments: dict | None = None):
    """Le modèle demande UN outil, puis repond en reprenant l'observation.

    Base du scénario S1 (outil reel, canary) : le tour 1 propose l'appel
    d'outil (forme ReAct d'ADR-0006 — le wire ne transporte pas encore
    `tool_calls`), le tour 2 reprend **integralement** le dernier message
    (l'observation reinjectee par la boucle). Les noms de canary sont
    aleatoires : le modele ne peut pas les connaitre autrement qu'en lisant
    l'observation — c'est la preuve que l'outil a reellement ete execute.

    La reprise est INTEGRALE (pas tronquee comme `seq_echoreq_reply`) : la
    liste des canaries doit tenir dans la reponse.
    """
    call = _reply(json.dumps({"tool": tool, "arguments": arguments or {}}, sort_keys=True))

    def echo_observation(req):
        msgs = req.get("messages", [])
        last = msgs[-1].get("content", "") if msgs else ""
        return _reply(last)

    return [lambda req: call, echo_observation]


def seq_memory_store_then_recall(canary):
    """4 tours : session A stocke, session B rappelle (memory_recall) puis echo.

    Le tour 3 (`memory_recall`) est la preuve de persistance : le fait stocke
    en A est retrouve en B via l'outil, pas via la prose scriptee.
    """
    canary = str(canary)
    store_call = _reply(json.dumps(
        {"tool": "memory_store",
         "arguments": {"content": f"Le code couleur du projet est {canary}",
                       "provenance": "[stated]"}},
        sort_keys=True))
    recall_call = _reply(json.dumps(
        {"tool": "memory_recall", "arguments": {"query": "code couleur du projet"}},
        sort_keys=True))

    def confirm(req):
        return _reply(f"Memorise : {canary}")

    def echo_observation(req):
        msgs = req.get("messages", [])
        last = msgs[-1].get("content", "") if msgs else ""
        return _reply(last)

    return [lambda req: store_call, confirm, lambda req: recall_call, echo_observation]


def seq_long_context_compaction(facts):
    """8 a 15 tours de remplissage, puis compaction declenchee.

    `facts` peut etre une chaine ou une liste de chaines : les faits sont
    plantes aux tours 0 et au milieu. Chaque tour renvoie un message long
    pour forcer le depassement de budget et la compaction.
    """
    if isinstance(facts, list | tuple):
        fact_list = [str(f) for f in facts]
    else:
        fact_list = [str(facts)]
    if not fact_list:
        fact_list = [""]

    n = 12
    turns = []
    # Un seul appel : le faux LLM renvoie le contexte qu'il a recu (compresse
    # par le moteur). La preuve S5 porte sur la requete reçue : les faits
    # epingles doivent y figurer ; le moteur seul les preserve sous budget.
    def echo_context(req):
        return _reply(json.dumps(req.get("messages", []), ensure_ascii=False))
    return [echo_context]
