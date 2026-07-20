# Agent A14: Traefik — API Gateway & Reverse Proxy

## TASK
Add Traefik (MIT) as a reverse proxy with automatic service discovery, Let's Encrypt SSL, and rate limiting — replacing manual port binding.

## CONTEXT
- Current: No reverse proxy. FastAPI binds directly to ports. SECURITY.md recommends Cloudflare/Tailscale/VPN.
- Traefik: Docker-native, auto-discovers services via labels, auto-SSL via Let's Encrypt, middleware for rate limiting/IP whitelisting.

## REQUIREMENTS

### 1. Docker Compose
Add Traefik to `docker-compose.yml`:
```yaml
traefik:
  image: traefik:latest
  command:
    - "--api.insecure=false"
    - "--providers.docker=true"
    - "--providers.docker.exposedbydefault=false"
    - "--entrypoints.web.address=:80"
    - "--entrypoints.websecure.address=:443"
    - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
    - "--certificatesresolvers.letsencrypt.acme.email=${ACME_EMAIL}"
    - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
  ports:
    - "80:80"
    - "443:443"
  volumes:
    - /var/run/docker.sock:/var/run/docker.sock:ro
    - letsencrypt-data:/letsencrypt
  profiles: ["gateway"]
```

### 2. Service Labels
Add Traefik labels to Odysseus and other services:
```yaml
odysseus:
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.odysseus.rule=Host(`${DOMAIN:-localhost}`)"
    - "traefik.http.routers.odysseus.entrypoints=websecure"
    - "traefik.http.routers.odysseus.tls.certresolver=letsencrypt"
    - "traefik.http.middlewares.ratelimit.ratelimit.average=100"
    - "traefik.http.middlewares.ratelimit.ratelimit.burst=50"
```

### 3. Env Config
In `.env.example`:
```bash
DOMAIN=localhost
ACME_EMAIL=admin@example.com
ODYSSEUS_TRAEFIK=off
```

### 4. Documentation
Update `docs/setup.md` with Traefik setup for HTTPS.

### 5. Kill-Switch
`ODYSSEUS_TRAEFIK=off` → bind directly (current behavior)

## VERIFICATION
- `docker compose --profile gateway up` starts Traefik
- Dashboard on `:8080` (internal only)
- HTTP→HTTPS redirect works
- Rate limiting returns 429 after threshold

## OUTPUT
Files modified, setup docs update
