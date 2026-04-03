# Deployment Guide

This guide covers how to deploy the Chers chess server in different configurations.

## Deployment Scenarios

### 1. Single Binary Deployment (Static Build)

The simplest deployment option - a single binary that includes both the server and frontend.

**When to use:**
- Self-hosting on a VPS or dedicated server
- Docker containers with single artifact
- Easy distribution (just one file)

**Build:**
```bash
just chers-static
```

This creates `target/release/chers_server` with the frontend embedded. The frontend automatically uses relative URLs to connect to the embedded backend on the same origin.

**Deploy:**
```bash
# Copy the binary to your server
scp target/release/chers_server user@your-server:/opt/chers/

# Run on the server
ssh user@your-server "cd /opt/chers && ./chers_server"
```

**Access:**
- Local: `http://localhost:3000/`
- Network: `http://0.0.0.0:3000/` (accessible from any device on the network)

**How it works:**
- The build process sets `VITE_CHERS_SERVER_HOST=""` (empty)
- This tells the frontend to use relative URLs (`/matches/new` for REST, `ws://current-host/...` for WebSocket)
- Both frontend and backend run on the same origin, so no CORS issues

---

### 2. Split Deployment (Frontend + Backend Separately)

Deploy the frontend and backend on different platforms for scalability and cost optimization.

**When to use:**
- Hosting frontend on CDN/edge (Vercel, Cloudflare, AWS CloudFront)
- Backend on specialized platform (Fly.io, Railway, Render, AWS, etc.)
- Taking advantage of free tiers on multiple platforms

**Architecture:**
```
┌─────────────┐         ┌─────────────────┐
│   Vercel    │ ──────► │   Chers Server  │
│  (Frontend) │  WebSocket │   (Backend)   │
└─────────────┘         └─────────────────┘
```

**Build Frontend:**
```bash
just wasm-release server-ts
pnpm install
# Set your backend URL for the frontend to connect to
VITE_CHERS_SERVER_HOST=api.chers.example.com \
  VITE_CHERS_USE_SSL=true \
  pnpm --filter chers_web run build
```

Deploy the `chers_web/dist/client/` contents to your frontend platform.

**Build & Deploy Backend:**
```bash
cargo build --package=chers_server --release
```

Deploy the binary to your backend platform with CORS configured to allow your frontend domain.

**Configure CORS:**
The server uses `CorsLayer::new().allow_origin(Any)` by default, which allows any origin. For production, you should restrict this to your specific frontend domain.

---

## Environment Variables

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | The port the server listens on |
| `CHERS_LISTEN_ADDRESS` | `0.0.0.0` | IP address to bind the server to. Use `127.0.0.1` for localhost-only, `0.0.0.0` for all interfaces |
| `CHERS_HOST` | `localhost` | Hostname displayed in startup logs (for clickable URLs). Set to your public domain for clarity |

### Frontend Configuration (Build-time)

These environment variables are used **at build time** to configure the frontend's backend connection. In the frontend code, access them via `import.meta.env.VITE_*` (Vite's convention):

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_CHERS_SERVER_HOST` | `chers-server.fly.dev` | The backend server hostname for API/WebSocket calls. **Leave empty for single-binary deployment** (uses relative URLs) |
| `VITE_CHERS_USE_SSL` | `true` | Whether to use HTTPS/WSS (true) or HTTP/WS (false). Ignored when SERVER_HOST is empty |

**Behavior:**

- **Single Binary Deployment:** Set `VITE_CHERS_SERVER_HOST=""` (empty). The frontend uses relative URLs (`/matches/new`, `ws://current-host/...`), which automatically connect to the embedded backend on the same origin.

- **Split Deployment:** Set `VITE_CHERS_SERVER_HOST` to your backend domain (e.g., `api.chers.example.com`). The frontend uses absolute URLs with the specified host.

The `just chers-static` recipe automatically sets `VITE_CHERS_SERVER_HOST=""` to enable same-origin mode.

### Logging & Debugging

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level filter. Defaults to `info` so startup messages are visible. Use `debug` or `trace` for verbose output, or `error` to suppress non-error messages |

### Telemetry (Optional)

| Variable | Default | Description |
|----------|---------|-------------|
| `SENTRY_DSN` | - | Sentry error tracking DSN |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | OpenTelemetry collector endpoint |

---

## Startup Messages

When starting the server, you'll see one of these formats:

**Default (network accessible):**
```
🚀 Chers server ready
   Local:   http://localhost:3000/
   Network: http://0.0.0.0:3000/ (all interfaces)
```

**Localhost only:**
```bash
CHERS_LISTEN_ADDRESS=127.0.0.1 ./chers_server
```
```
🚀 Chers server ready
   Local:   http://localhost:3000/
```

**Custom display hostname:**
```bash
CHERS_HOST=chess.example.com ./chers_server
```
```
🚀 Chers server ready
   Local:   http://chess.example.com:3000/
   Network: http://0.0.0.0:3000/ (all interfaces)
```

**Specific interface:**
```bash
CHERS_LISTEN_ADDRESS=192.168.1.100 CHERS_HOST=chess.lan ./chers_server
```
```
🚀 Chers server ready
   Local:   http://chess.lan:3000/
```

---

## Security Considerations

### CORS in Production

The default CORS configuration allows any origin (`Any`). For production, consider restricting this:

```rust
// In main.rs, replace:
.layer(CorsLayer::new().allow_origin(Any))

// With specific origins:
.layer(CorsLayer::new().allow_origin([
    "https://your-frontend-domain.com".parse().unwrap(),
]))
```

### Binding to Specific Interfaces

For better security, bind only to necessary interfaces:

- **Development:** `CHERS_LISTEN_ADDRESS=127.0.0.1` (local only)
- **Behind reverse proxy (nginx, Caddy):** `CHERS_LISTEN_ADDRESS=127.0.0.1` and let the proxy handle external traffic
- **Direct exposure:** `CHERS_LISTEN_ADDRESS=0.0.0.0` (binds to all interfaces)

---

## WebSocket Considerations

The multiplayer functionality uses WebSocket connections to `/matches/{id}/play`.

- Most modern hosting platforms support WebSocket connections
- Ensure your reverse proxy (if any) is configured to upgrade WebSocket connections
- The connection includes a 2-minute reconnection grace period for network issues

---

## Health Checks

The server provides a health check endpoint at `GET /health` which returns:
```json
{"status": "ok"}
```

Use this for container health checks and load balancer health probes.

---

## Static File Serving (bundle-frontend feature)

When using the `bundle-frontend` feature:

- Frontend files are embedded at compile time using `rust-embed`
- The server verifies files exist at startup
- If files are missing, you'll see:
  ```
  Static frontend files not found!
  The 'bundle-frontend' feature is enabled but chers_web/dist/client/ appears to be empty.
  Please build the frontend first: just chers-static
  ```
- The server serves `_shell.html` (TanStack Start's SPA shell) for all non-API routes
- Static assets (JS, CSS, WASM) are served with proper MIME types
