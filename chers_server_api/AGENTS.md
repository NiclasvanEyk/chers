# Chers Server API - Agent Documentation

## Overview

This crate defines the WebSocket message protocol for multiplayer chess. It provides:

- **Client messages** - Actions sent by clients (make move, authenticate, etc.)
- **Server messages** - Events sent by the server (game state updates, notifications)
- **TypeScript generation** - Automatic TS type generation via `ts-rs`

This is a shared contract between the server and frontend.

## Key Technologies

- **serde** - Serialization/deserialization
- **ts-rs** - TypeScript type generation (with serde compatibility)
- **chers** - Core chess types (re-exported)

## Project Structure

```
src/
└── lib.rs              # All message types and protocol definitions
```

## Architecture

### Message Flow

```
Client                    Server
  |                          |
  |-- ClientMessage --------->|
  |                          |
  |<-------- ServerMessage --|
  |                          |
```

### Message Categories

1. **Client Messages** - Sent from client to server
   - `Authenticate` - Claim a player slot
   - `MakeMove` - Submit a chess move
   - `RequestSync` - Get current game state
   - `OfferDraw` / `AcceptDraw` / `DeclineDraw` - Draw handling
   - `Resign` - Concede the game
   - `Heartbeat` - Keep connection alive

2. **Server Messages** - Sent from server to clients
   - `PublicEvent` - Broadcast to all (players + spectators)
   - `PrivateEvent` - Sent only to specific player

See [../chers_server/AGENTS.md](../chers_server/AGENTS.md) for how the server handles these messages.

## TypeScript Integration

### WASM Types

Types from the `chers` crate (compiled to WASM) don't implement `ts_rs::TS`. We reference them by name:

```rust
#[ts(type = "Coordinate")]
from: Coordinate,
```

This generates TypeScript that references `Coordinate` from the WASM package rather than inlining the definition.

### Generated Files

Types are exported to `chers_web/generated/chers_server_api/`:

**Recommended way (from repo root):**

```bash
just chers-server-ts
```

**Manual way (from chers_server_api directory):**

```bash
# From chers_server_api directory
cargo test  # Generates TypeScript files
```

The `just chers-server-ts` command will:

1. Clean the old generated files
2. Create the output directory
3. Run `cargo test` with the correct export directory configured
4. Generate TypeScript bindings to `chers_web/generated/chers_server_api/`

```typescript
import { ServerMessage, ClientMessage } from "@/generated/chers_server_api";
```

See [../chers_web/AGENTS.md](../chers_web/AGENTS.md) for frontend usage.

## Key Types

### ClientMessage

All possible client actions:

```rust
pub enum ClientMessage {
    Authenticate { token: String, name: String },
    MakeMove { from: Coordinate, to: Coordinate, promotion: Option<PromotionPiece> },
    RequestSync,
    OfferDraw,
    AcceptDraw,
    DeclineDraw,
    Resign,
    Heartbeat,
}
```

### ServerMessage

Top-level server message wrapper:

```rust
pub enum ServerMessage {
    Public(PublicEvent),
    Private(PrivateEvent),
}
```

### PublicEvent

Broadcast to all viewers:

- `GameStarted` - Game begins with initial state
- `MoveMade` - A move was played
- `GameOver` - Game ended (checkmate, draw, etc.)
- `PlayerStatusChanged` - Connection status update
- `DrawOffered` / `DrawDeclined` - Draw interactions

### PrivateEvent

Sent to specific player only:

- `Authenticated` - Login success with assigned color
- `AuthenticationFailed` - Login rejected
- `MoveRejected` - Invalid move submission
- `StateSync` - Full game state for reconnection
- `DrawOffered` / `DrawDeclined` - Opponent draw interactions
- `HeartbeatAck` - Ping response

## Type Safety

### Serde Tagging

All enums use `#[serde(tag = "kind")]` for discriminant-based deserialization:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ClientMessage {
    Authenticate { ... },  // Serialized as {"kind": "Authenticate", ...}
    MakeMove { ... },    // Serialized as {"kind": "MakeMove", ...}
}
```

### ServerMessage Visibility Tagging

Uses `#[serde(tag = "visibility", content = "event")]`:

```rust
#[serde(tag = "visibility", content = "event")]
pub enum ServerMessage {
    Public(PublicEvent),    // {"visibility": "Public", "event": {...}}
    Private(PrivateEvent),  // {"visibility": "Private", "event": {...}}
}
```

## Game End Reasons

Machine-readable codes for game termination:

```rust
pub enum GameEndReason {
    #[serde(rename = "checkmate")]
    Checkmate,
    #[serde(rename = "stalemate")]
    Stalemate,
    #[serde(rename = "resignation")]
    Resignation,
    #[serde(rename = "draw_agreement")]
    DrawAgreement,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "abandoned")]
    Abandoned,
    // ... (FIDE rules)
}
```

Frontend should map these to human-readable messages.

## Move Rejection Reasons

Why a move might be rejected:

```rust
pub enum MoveRejectionReason {
    InvalidNotation,    // Couldn't parse move
    IllegalMove,      // Not legal for current position
    NotYourTurn,      // Wrong player's turn
    GameOver,         // Game already ended
}
```

## Usage Examples

### Making a Move

```rust
// Client sends
let message = ClientMessage::MakeMove {
    from: Coordinate::new(File::E, Rank::_2),
    to: Coordinate::new(File::E, Rank::_4),
    promotion: None,
};
```

### Handling Server Response

```rust
// Server broadcasts to all
let event = PublicEvent::MoveMade {
    author: Color::White,
    from: Coordinate::new(File::E, Rank::_2),
    to: Coordinate::new(File::E, Rank::_4),
    new_state: game_state,
    is_check: false,
    is_checkmate: false,
};
```

## Build Commands

| Command                | Description                                |
| ---------------------- | ------------------------------------------ |
| `just chers-server-ts` | Generate TypeScript bindings (recommended) |
| `cargo build`          | Build the crate                            |
| `cargo test`           | Run tests and generate TypeScript          |
| `cargo doc --open`     | View documentation                         |

## Dependencies

```toml
[dependencies]
chers = { path = "../chers" }  # Core chess engine
serde = { version = "1", features = ["derive"] }
ts-rs = { version = "10", features = ["serde-compat"] }
```

## Related Documentation

- [Server Implementation](../chers_server/AGENTS.md) - WebSocket server handling these messages
- [Frontend Usage](../chers_web/AGENTS.md) - Client-side protocol handling
- [Core Engine](../chers/AGENTS.md) - Chess rules and types

## Protocol Versioning

Currently no explicit versioning. Changes are coordinated between:

1. Update this crate
2. Regenerate TypeScript
3. Update server to handle new messages
4. Update frontend to send/receive new messages

For breaking changes, consider adding a version field to `Authenticate`.
