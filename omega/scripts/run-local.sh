#!/usr/bin/env bash
# Lance Omega en local : une seule origine (API + cockpit React servis par omega-server).
#
# Usage :
#   omega/scripts/run-local.sh            # port 8787 (defaut)
#   omega/scripts/run-local.sh 9000       # port explicite
#
# Prerequis : `ollama serve` en fond et le modele tire
#   ollama pull qwen2.5-coder:7b
# Pour un contexte assemble > 4096 tokens, demarrer Ollama avec
#   OLLAMA_NUM_CTX=32768 ollama serve
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
omega="$(cd "$here/.." && pwd)"
cd "$omega"

port="${1:-8787}"

echo "[1/3] UI : build (Vite)"
if [ ! -d ui/node_modules ]; then
  ( cd ui && npm install --no-audit --no-fund )
fi
( cd ui && npm run build )

echo "[2/3] Serveur : build (cargo)"
cargo build -p omega-server

echo "[3/3] Lancement sur http://127.0.0.1:${port}"
echo "      (Ctrl-C pour arreter)"
export OMEGA_PORT="$port"
export OMEGA_UI_DIR="$omega/ui/dist"
exec ./target/debug/omega-server
