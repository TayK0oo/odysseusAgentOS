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

# Install local SFD packages
RUN cd /home/agentos/packages/sfd-eventbus && npm install && npm link && \
    cd /home/agentos/packages/sfd-phase && npm install && npm link && \
    cd /home/agentos/packages/sfd-memory && npm install && npm link && \
    cd /home/agentos/packages/sfd-durable && npm install && npm link && \
    cd /home/agentos/packages/sfd-prefs && npm install && npm link && \
    cd /home/agentos/packages/sfd-visual && npm install && npm link && \
    cd /home/agentos/packages/sfd-classify && npm install && npm link && \
    cd /home/agentos/packages/sfd-security && npm install && npm link && \
    cd /home/agentos/packages/sfd-discovery && npm install && npm link && \
    cd /home/agentos/packages/sfd-heartbeat && npm install && npm link && \
    cd /home/agentos && npm link @agentos/sfd-eventbus @agentos/sfd-phase @agentos/sfd-memory \
    @agentos/sfd-durable @agentos/sfd-prefs @agentos/sfd-visual @agentos/sfd-classify \
    @agentos/sfd-security @agentos/sfd-discovery @agentos/sfd-heartbeat

# Copy bridge script
COPY --chown=agentos:agentos src/opencode_bridge.py /home/agentos/bridge.py
COPY --chown=agentos:agentos src/opencode_engine.py /home/agentos/engine.py

# Workspace for project execution (INTERNAL only, no host mount)
RUN mkdir -p /home/agentos/workspace && chown agentos:agentos /home/agentos/workspace

# Data directory (traces, memory, etc.)
RUN mkdir -p /home/agentos/data/traces /home/agentos/obsidian-vault
RUN chown -R agentos:agentos /home/agentos/data /home/agentos/obsidian-vault

USER agentos
ENV HOME=/home/agentos
ENV OPENCODE_HOME=/home/agentos/.opencode

# The engine API key (separate from user's key)
# Set via docker-compose environment
ENV OPENCODE_API_KEY=""

EXPOSE 7001
CMD ["sh", "-c", "echo 'AgentOS Engine ready' && tail -f /dev/null"]
