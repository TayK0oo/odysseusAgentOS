# AgentOS Engine — Isolated OpenCode runtime
# This is NOT the user's OpenCode CLI. This is the SYSTEM's own engine.
FROM node:22-slim

LABEL com.agentos.role=engine
LABEL com.agentos.description="AgentOS OpenCode Engine — isolated from host"

# Install OpenCode CLI (the system's own copy)
RUN curl -fsSL https://opencode.ai/install | bash

# System dependencies for the engine
RUN apt-get update && apt-get install -y --no-install-recommends \
    git curl jq python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Create agentos user (not root for security)
RUN useradd -m -s /bin/bash agentos
WORKDIR /home/agentos

# Copy AgentOS config (OWN config, not user's)
COPY --chown=agentos:agentos opencode.json /home/agentos/opencode.json
COPY --chown=agentos:agentos .opencode/ /home/agentos/.opencode/
COPY --chown=agentos:agentos packages/ /home/agentos/packages/

# Install local SFD eventbus package (only one with real TS impl; sfd-phase is a custom tool loaded via .opencode/tools/)
RUN cd /home/agentos/packages/sfd-eventbus && npm install && npm link && \
    cd /home/agentos && npm link @agentos/sfd-eventbus

# Python deps for the bridge server
RUN pip install --no-cache-dir fastapi uvicorn httpx

# Copy bridge server
COPY docker/agentos-engine/engine_server.py /home/agentos/server.py

# Copy engine code
COPY src/opencode_engine.py /home/agentos/engine.py
COPY src/perf_profiler.py /home/agentos/perf_profiler.py

USER agentos
ENV HOME=/home/agentos

EXPOSE 7001
CMD ["python", "/home/agentos/server.py"]
