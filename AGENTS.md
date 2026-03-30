# Chers - Agent Documentation

This is a multiplayer chess application with a Rust/WASM engine, Next.js frontend, and Axum server.

## Project Structure

```
chers/                      # Core chess engine (Rust/WASM)
├── src/                    # Chess rules, move validation, game state
└── AGENTS.md               # See chers/AGENTS.md

chers_web/                  # Next.js web frontend
├── app/                    # App router pages and components
├── components/             # React components
├── lib/                    # Utilities, multiplayer logic, WASM bindings
└── AGENTS.md               # See chers_web/AGENTS.md

chers_server/               # WebSocket multiplayer server (Axum/Shuttle)
├── src/
│   ├── handlers/           # HTTP and WebSocket handlers
│   └── matches/            # Match state management
└── AGENTS.md               # See chers_server/AGENTS.md

chers_server_api/           # Shared message protocol
├── src/lib.rs              # WebSocket message definitions
└── AGENTS.md               # See chers_server_api/AGENTS.md

chers_cli/                  # Terminal chess client
└── src/main.rs
```

## Quick Links

- **[Frontend Development](chers_web/AGENTS.md)** - Next.js, React, WebSocket client, WASM integration
- **[Server Development](chers_server/AGENTS.md)** - Axum, WebSocket handlers, match state machine, deployment
- **[Server API](chers_server_api/AGENTS.md)** - Message protocol, TypeScript generation, API types
- **[Chess Engine](chers/AGENTS.md)** - Core chess logic (if/when documented)

## Architecture Overview

### Single Player (Local)
```
User -> Next.js (React) -> WASM Engine -> Browser
```

### Multiplayer
```
Player A (Web)                Player B (Web)
    |                             |
    v                             v
Next.js + WASM              Next.js + WASM
    |                             |
    +------------->+--------------+
                   |
            WebSocket Connection
                   |
             Axum Server (Shuttle)
                   |
            Match State (scc::HashMap)
```

## Development Workflow

### Running Locally

**Terminal 1** - Start the server:
```bash
cd chers_server
cargo shuttle run
```

**Terminal 2** - Start the frontend:
```bash
cd chers_web
bun dev
```

**Terminal 3** - (Optional) CLI client:
```bash
cd chers_cli
cargo run
```

### Regenerate TypeScript Types

When you modify message types in `chers_server_api`:

```bash
cd chers_server_api
cargo test  # Generates TypeScript to chers_web/generated/
```

### Build for Production

**Frontend:**
```bash
cd chers_web
bun run build
```

**Server:**
```bash
cd chers_server
cargo shuttle deploy
```

## Technology Stack

| Component | Technology |
|-----------|-----------|
| Chess Engine | Rust → WASM |
| Frontend | Next.js 16, React 19, TypeScript 5, Tailwind 4 |
| Server | Axum 0.8, Tokio, Shuttle |
| Protocol | WebSocket + JSON |
| Package Manager | Bun (frontend), Cargo (Rust) |
| Deployment | Vercel (frontend), Shuttle (server) |

## Key Features

- **Local Chess** - Play against yourself in the browser using WASM engine
- **Multiplayer** - Real-time WebSocket games with turn enforcement
- **Reconnection** - 2-minute grace period for network issues
- **Spectators** - Watch games in progress (future feature)
- **Accessibility** - Full keyboard navigation and ARIA labels

## Development Commands Reference

| Task | Command | Location |
|------|---------|----------|
| Dev frontend | `bun dev` | `chers_web/` |
| Dev server | `cargo shuttle run` | `chers_server/` |
| Test frontend | `bun test` | `chers_web/` |
| Test server | `cargo test` | `chers_server/` |
| Lint frontend | `bun run lint` | `chers_web/` |
| Generate types | `cargo test` | `chers_server_api/` |
| Deploy server | `cargo shuttle deploy` | `chers_server/` |

## Common Tasks

### Adding a New API Message

1. Define in `chers_server_api/src/lib.rs`
2. Run `cargo test` in `chers_server_api/` to generate TypeScript
3. Handle in `chers_server/src/handlers/matches/play.rs`
4. Handle in `chers_web/lib/multiplayer/useMatch.ts`
5. Update UI in `chers_web/app/multiplayer/[id]/components/`

### Debugging WebSocket Issues

1. Check browser Network tab → WS filter
2. Server logs: `RUST_LOG=debug cargo shuttle run`
3. See [Server Development](chers_server/AGENTS.md) for common issues

### Testing Multiplayer Locally

1. Open `http://localhost:3000` in two different browsers
2. In Browser 1: Settings → "Start New Multiplayer Game"
3. Copy URL, paste into Browser 2
4. Both should connect and see the game start

## Documentation by Component

Each crate/package has detailed documentation:

- **[chers_web/AGENTS.md](chers_web/AGENTS.md)** - Frontend architecture, multiplayer hook, WebSocket connection, WASM initialization
- **[chers_server/AGENTS.md](chers_server/AGENTS.md)** - Server routes, match lifecycle, color assignment, reconnection logic, deployment
- **[chers_server_api/AGENTS.md](chers_server_api/AGENTS.md)** - Message protocol, TypeScript generation, type safety

## Environment Setup

### Prerequisites

- [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [cargo-shuttle](https://docs.shuttle.rs/getting-started/installation) (for server deployment)

### Environment Variables

**Frontend** (`chers_web/.env.local`):
```env
NEXT_PUBLIC_SERVER_URL=http://localhost:3001  # Local dev
```

**Server** - None required locally (Shuttle handles production config)

## Repository

- GitHub: https://github.com/NiclasvanEyk/chers
- Live: https://chers.niclasve.me

---

*This is a living document. Update when adding new components or major features.*
