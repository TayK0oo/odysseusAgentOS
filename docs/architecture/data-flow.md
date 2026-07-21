# Architecture — Flux de données

## Création de projet (chat → LLM → outils)

```mermaid
sequenceDiagram
    participant U as Utilisateur
    participant UI as Navigateur
    participant API as /api/chat
    participant Loop as stream_agent_loop
    participant LLM as llm_core
    participant Tool as tool_execution
    participant Mem0 as Mem0Provider
    participant Obs as Observer

    U->>UI: "Crée un script Python..."
    UI->>API: POST /api/chat (SSE)
    API->>Loop: stream_agent_loop(session, message)
    Loop->>Loop: _classify_agent_request()
    Loop->>Loop: _build_system_prompt()
    Loop->>LLM: stream_llm_with_fallback()
    LLM-->>Loop: Token streaming
    Loop->>Tool: execute_tool_block("bash", "echo...")
    Tool-->>Loop: Résultat stdout
    Loop->>Mem0: add_conversation(session, messages)
    Mem0-->>Loop: Faits extraits
    Loop->>Obs: ingest_metrics()
    Loop-->>API: SSE events
    API-->>UI: Streaming response
    UI-->>U: "PONG"
```

## Recherche RAG

```mermaid
sequenceDiagram
    participant U as Utilisateur
    participant API as /api/chat
    participant RAG as VectorRAG
    participant CH as ChromaDB
    participant EM as FastEmbed

    U->>API: "Que dit la doc sur X?"
    API->>RAG: vector_search(query)
    RAG->>EM: embed(query)
    EM-->>RAG: vecteur [384]
    RAG->>CH: search(collection, vector, limit=10)
    CH-->>RAG: [doc1, doc2, doc3]
    RAG->>RAG: RRF fusion (si activé)
    RAG-->>API: Contexte RAG
    API-->>U: Réponse augmentée
```

## Observation et traces

```mermaid
sequenceDiagram
    participant Loop as agent_loop
    participant Tool as tool_execution
    participant Obs as Observer
    participant TW as trace_writer

    Loop->>Tool: execute_tool()
    Tool->>TW: write_trace(tool_call)
    Tool-->>Loop: Résultat + cost_tokens
    Loop->>Obs: ingest_metrics(round_data)
    Obs->>Obs: compute_drift_score()
    Loop->>TW: record_run_tokens(total)
    Loop-->>UI: SSE run_status (phase, drift, iters)
```

## Kill-switch → Code

```mermaid
graph LR
    ENV[".env\nODYSSEUS_X=on"]-->KS[killswitch_registry.py]
    KS-->AGENT[agent_loop.py]
    AGENT-->MODULE[Module spécifique]
    
    KS-->|"PhaseTracker"|PT[phase_tracker.py]
    KS-->|"LangGraph"|LG[langgraph_loop.py]
    KS-->|"Autoeval"|AE[autoeval.py]
    KS-->|"Mem0"|M0[mem0_provider.py]
    KS-->|"Meilisearch"|MS[meilisearch_client.py]
```

---

→ Voir aussi : [Vue d'ensemble](overview.md) · [Boucle agent](agent-loop.md) · [Observabilité](../operations/observability.md)
