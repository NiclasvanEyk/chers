# Chers Web Frontend - Agent Documentation

## Overview

This is the Next.js web frontend for the Chers chess application. It provides:

- **Local single-player chess** - The default mode, played against yourself
- **Multiplayer chess** - Real-time WebSocket-based multiplayer games
- **WASM-powered engine** - Uses the Rust chess engine compiled to WebAssembly

## Key Technologies

- **Next.js 16** with App Router
- **React 19** and React Server Components
- **TypeScript 5**
- **Tailwind CSS 4** for styling
- **pnpm** for package management
- **Bun** for testing (via `bun test`)
- **chers** (WASM) - The Rust chess engine

## Project Structure

```
app/
├── page.tsx                 # Home page with local chess game
├── layout.tsx               # Root layout
├── Home.tsx                 # Home page client component
├── multiplayer/
│   ├── page.tsx             # Redirects to new match
│   └── [id]/
│       ├── page.tsx         # Main multiplayer match page
│       └── components/      # Match-specific UI components
│           ├── Lobby.tsx           # Waiting for opponent
│           ├── MultiplayerGame.tsx # Active game
│           ├── GameOver.tsx        # Game finished screen
│           ├── PlayerBar.tsx       # Player info bar
│           └── ReconnectingOverlay.tsx
components/
├── Chers.tsx                # Main chess board component
├── Board.tsx                # Chess board grid
├── Cell.tsx                 # Individual board cell
├── Piece.tsx                # Chess piece SVG rendering
├── Promotion.tsx            # Pawn promotion dialog
├── Settings.tsx             # Settings menu with multiplayer button
└── ...
lib/
├── chers.ts                 # Chess engine WASM bindings
├── ui/
│   ├── state.ts             # UI state management (selection, moves)
│   ├── accessibility.ts     # ARIA labels and keyboard navigation
│   └── useFocusManagement.ts # Focus management for accessibility
└── multiplayer/
    ├── token.ts             # Player credential management (sessionStorage)
    ├── connection.ts        # WebSocket connection with reconnection
    ├── useMatch.ts          # Main multiplayer state hook
    └── index.ts             # Public exports
```

## Multiplayer Architecture

The multiplayer feature uses a WebSocket connection to the server:

1. **Match Creation**: Click "Start New Multiplayer Game" in Settings → creates match via REST API
2. **Match Joining**: Second player visits the same match URL
3. **WebSocket Connection**: Both players connect to `/matches/{id}/play`
4. **Authentication**: Players send `Authenticate` message with token
5. **Gameplay**: Real-time move exchange via WebSocket

See [../chers_server/AGENTS.md](../chers_server/AGENTS.md) for the server architecture.

## Key Multiplayer Components

### useMatch Hook (`lib/multiplayer/useMatch.ts`)

The main state management for multiplayer games. Handles:

- Connection lifecycle (connecting → connected → reconnecting)
- Message processing from server
- Game state synchronization
- Move sending

```typescript
const { matchState, sendMove, isConnected } = useMatch(matchId);
```

### MatchConnection (`lib/multiplayer/connection.ts`)

WebSocket wrapper with exponential backoff reconnection:

- 1s → 2s → 4s → 8s → 16s → 30s backoff
- Max reconnection window: 2 minutes (conservative approach)
- Heartbeat/ping-pong every 30 seconds

### Token Management (`lib/multiplayer/token.ts`)

Stores player credentials in sessionStorage:

- `token` - Unique player identifier
- `matchId` - Match the player is in
- `playerName` - Generated player name

## Game Board Integration

The `Chers` component supports both local and multiplayer modes:

```typescript
// Local mode (default)
<Chers />

// Multiplayer mode
<Chers
  multiplayerState={matchState.game}
  onMove={sendMove}
  disabled={matchState.phase !== 'playing'}
/>
```

## WASM Initialization

The chess engine WASM must be initialized before use. In multiplayer:

```typescript
// app/multiplayer/[id]/page.tsx
const [wasmReady, setWasmReady] = useState(false);
useEffect(() => {
  init().then(() => setWasmReady(true));
}, []);
```

## Testing

Run tests with Bun (still required for testing):

```bash
bun test
```

Or from the repo root:

```bash
pnpm web test
```

Test files follow the pattern `*.test.ts` (e.g., `lib/ui/state.test.ts`).

## Development

From the `chers_web/` directory:

```bash
# Install dependencies
pnpm install

# Run dev server
pnpm dev

# Build for production
pnpm run build

# Run linter
pnpm run lint
```

Or from the repo root using pnpm workspace filtering:

```bash
# Run dev server
pnpm web dev

# Build for production
pnpm web build

# Run linter
pnpm web lint
```

The dev server runs on http://localhost:3000.

## Environment Configuration

Create `.env.local`:

```env
# API server for multiplayer (default: localhost:3001)
NEXT_PUBLIC_SERVER_URL=http://localhost:3001
```

## Type Generation

The multiplayer API types are generated from the Rust server API. See [../chers_server_api/AGENTS.md](../chers_server_api/AGENTS.md) for how this works.

## Common Issues

### React Strict Mode Double Mount

In development, React Strict Mode causes components to mount/unmount/remount. This can close WebSocket connections prematurely. We add a 500ms delay in cleanup functions to handle this.

### WASM Not Initialized

Always initialize WASM before rendering the chess board:

```typescript
import { init } from "chers";
```

### Next.js 15 Params

In Next.js 15, `params` is a Promise and must be unwrapped:

```typescript
const { id } = React.use(params); // Not: const { id } = params;
```

## Build Commands

| Command          | Description                      |
| ---------------- | -------------------------------- |
| `pnpm dev`       | Start dev server with hot reload |
| `pnpm run build` | Production build                 |
| `pnpm run lint`  | Run ESLint                       |
| `bun test`       | Run test suite (requires Bun)    |

From repo root using `pnpm web <command>`:

| Command          | Description                      |
| ---------------- | -------------------------------- |
| `pnpm web dev`   | Start dev server with hot reload |
| `pnpm web build` | Production build                 |
| `pnpm web lint`  | Run ESLint                       |
| `pnpm web test`  | Run test suite                   |

## Related Documentation

- [Server Architecture](../chers_server/AGENTS.md) - WebSocket server and game logic
- [Server API](../chers_server_api/AGENTS.md) - Message protocol and type definitions
- [Core Engine](../chers/AGENTS.md) - Chess rules and move validation
