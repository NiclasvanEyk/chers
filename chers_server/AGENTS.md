# Chers Server - Agent Documentation

## Overview

The Chers server provides WebSocket-based multiplayer chess functionality. It's built with Rust using Axum.

## Key Technologies

- **Axum 0.8** - Web framework with WebSocket support
- **Tokio** - Async runtime
- **scc** - Scalable concurrent containers (for match state storage)
- **UUID** - Match ID generation
- **Tracing** - Structured logging

## Project Structure

```
src/
├── main.rs                    # Server entry point, routes
├── lib.rs                     # Library exports (for tests)
├── handlers/
│   ├── health.rs              # Health check endpoint
│   ├── mod.rs
│   └── matches/
│       ├── mod.rs
│       ├── new.rs             # POST /matches/new - Create match
│       └── play.rs            # WS /matches/{id}/play - WebSocket handler
└── matches/
    ├── mod.rs                 # MatchId type and utilities
    ├── channels.rs            # Channel management for player communication
    ├── player.rs              # Player struct and connection handling
    ├── repository.rs          # In-memory match storage (scc::HashMap)
    └── state.rs               # Match state machine and game logic

tests/
├── e2e_tests.rs               # End-to-end WebSocket tests
└── common/
    └── mod.rs                 # Test utilities
```

## Architecture

### Match Lifecycle

1. **Created** (`new.rs`)
   - POST `/matches/new` creates a new match
   - Generates unique MatchId (UUID)
   - Initializes empty match state

2. **Waiting for Players** (`play.rs`)
   - WebSocket connection to `/matches/{id}/play`
   - First player: assigned randomly as White or Black, gets token
   - Second player: fills remaining slot
   - Match transitions to `InProgress` when 2 players connected

3. **In Progress**
   - Players exchange moves via WebSocket
   - Server validates all moves
   - Game ends on checkmate, resignation, or timeout

4. **Ended**
   - Final state persisted briefly
   - Match eventually cleaned up

### State Management

Matches are stored in a concurrent HashMap (`scc::HashMap`):

```rust
// repository.rs
pub type MatchRepository = scc::HashMap<MatchId, Arc<Mutex<Match>>>;
```

Each match is wrapped in an `Arc<Mutex<>>` for thread-safe access.

### Color Assignment

Colors are assigned randomly when the game starts (not before):

```rust
// In state.rs, start_game() method
let (white_token, black_token) = if rand::random() {
    (token1, token2)
} else {
    (token2, token1)
};
```

This ensures both players learn their color at the same time when the game begins.

## WebSocket Protocol

The server uses a message-based protocol defined in [../chers_server_api/AGENTS.md](../chers_server_api/AGENTS.md).

### Connection Flow

1. Client opens WebSocket connection
2. Server sends `ConnectionEstablished` (not used in current impl)
3. Client sends `Authenticate { token, name }`
4. Server responds:
   - `Authenticated { player: Color }` on success
   - `AuthenticationFailed { reason }` on failure
5. If game in progress, server sends `StateSync` with full state

### Message Handling

See `src/handlers/matches/play.rs` - the `handle_socket` function:

- Receives `ClientMessage` from WebSocket
- Updates match state
- Broadcasts `ServerMessage` to appropriate recipients
- Uses `tokio::select!` to handle concurrent message streams

### Channels

Each match has broadcast channels for events:

```rust
// channels.rs
pub struct MatchChannels {
    public: broadcast::Sender<PublicEvent>,
    white: mpsc::Sender<PrivateEvent>,
    black: mpsc::Sender<PrivateEvent>,
}
```

## Reconnection

The server handles disconnections with a grace period:

1. Player disconnects → Status changed to `Disconnected`
2. 2-minute grace period starts
3. Player reconnects with same token → `Authenticated` + `StateSync`
4. Grace period expires → Game ends by `Abandoned`

The grace period gives players time to reconnect after network issues.

## Disconnection Handling

When a player disconnects:

```rust
// play.rs
let disconnect_result = timeout(Duration::from_secs(120), async {
    // Wait for reconnection or timeout
}).await;

if disconnect_result.is_err() {
    // Grace period expired - end game
    // Send GameOver with Abandoned reason
}
```

Important: We delay closing the connection slightly after GameOver to ensure the player who made the winning move receives the message.

## Testing

Run tests with cargo:

```bash
# Unit tests
cargo test

# E2E tests (requires server running)
cargo test --test e2e_tests
```

## Logging

Structured logging with tracing:

```rust
tracing::info!(match_id = %id, player = ?color, "Game started");
tracing::debug!("Move received: {:?}", message);
```

Log levels are controlled via the `RUST_LOG` environment variable. By default, the server logs at `info` level. For development, you can enable debug logging:

```bash
# Debug logging for the server
RUST_LOG=chers_server=debug cargo run

# Debug logging for everything
RUST_LOG=debug cargo run

# Trace level (very verbose)
RUST_LOG=trace cargo run
```

## Common Issues

### Axum 0.8 Path Syntax

In Axum 0.8, use `{id}` not `:id`:

```rust
// Correct
.route("/matches/{id}/play", get(...))

// Wrong (older Axum versions)
.route("/matches/:id/play", get(...))
```

### WASM Types in API

Some types (Coordinate, Color, Game) come from the chers WASM crate and don't implement `ts_rs::TS`. Use `#[ts(type = "TypeName")]` in the API crate.

## Build Commands

| Command                       | Description                    |
| ----------------------------- | ------------------------------ |
| `cargo run`                   | Run server locally (port 3000) |
| `PORT=8000 cargo run`         | Run server on custom port      |
| `cargo test`                  | Run unit tests                 |
| `cargo test --test e2e_tests` | Run E2E tests                  |

## Static Frontend Bundle (Single Binary)

The server supports embedding the frontend directly into the binary for easy deployment. This is enabled via the `bundle-frontend` feature flag.

### How it works

When the `bundle-frontend` feature is enabled:

1. **Build-time embedding**: The Rust compiler embeds all files from `chers_web/dist/client/` directly into the binary using `rust-embed`
2. **Runtime serving**: The server serves these embedded files via the `/*path` catch-all route
3. **SPA routing**: All non-API routes serve `_shell.html` (TanStack Start's SPA shell), allowing client-side routing to work

### Building a static binary

From the repo root:

```bash
just chers-static
```

This will:
1. Build the WASM chess engine
2. Generate TypeScript types from the server API
3. Build the frontend (Vite + TanStack Start)
4. Compile the server with frontend files embedded

### Verification at startup

When `bundle-frontend` is enabled, the server verifies at startup that the embedded files exist. If the frontend wasn't built, you'll see:

```
Static frontend files not found!
The 'bundle-frontend' feature is enabled but chers_web/dist/client/ appears to be empty.
Please build the frontend first: just chers-static
```

### Deployment

With a static binary, deployment is simple - just the single executable:

```bash
# Build
just chers-static

# Deploy (example)
scp target/release/chers_server user@server:/opt/chers/
ssh user@server "cd /opt/chers && ./chers_server"
```

No separate frontend server or CDN required - everything is self-contained.

## Environment Variables

- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Log level filter (default: info)

Example:

```bash
PORT=8000 RUST_LOG=debug cargo run
```

## API Documentation

For message protocol details, see:

- [Server API Types](../chers_server_api/AGENTS.md) - Message definitions and protocol
- [Frontend](../chers_web/AGENTS.md) - Client-side WebSocket handling

## Match State Machine

```
WaitingForPlayers
       |
       v (2 players connected)
  InProgress
       |
       +----> Ended (checkmate/resignation/draw/abandoned)
```

See `src/matches/state.rs` for the full implementation.
