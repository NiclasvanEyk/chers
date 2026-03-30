function parseBoolean(value: unknown) {
  if (value === "true") return true;
  if (value === "false") return false;
  return undefined;
}

const SERVER_HOST =
  process.env.NEXT_PUBLIC_CHERS_SERVER_HOST ?? "chers-server.fly.dev";
const USE_SSL = parseBoolean(process.env.NEXT_PUBLIC_CHERS_USE_SSL) ?? true;
const SERVER_URL = `${USE_SSL ? "https" : "http"}://${SERVER_HOST}`;
const WEBSOCKET_URL = `${USE_SSL ? "wss" : "ws"}://${SERVER_HOST}`;

/**
 * Requests the server to start a new game.
 * Returns a UUID string that identifies the match.
 */
export async function startNewMatch(): Promise<string> {
  const response = await fetch(`${SERVER_URL}/matches/new`, { method: "POST" });
  const body = await response.json();
  const id = body.id;

  if (typeof id !== "string") {
    throw new Error(
      `Server did not respond with a valid match id! Got: ${id}`,
    );
  }

  return id;
}

/**
 * Builds up the websocket object for a given match.
 * The matchId should be a UUID string.
 */
export function play(matchId: string): WebSocket {
  return new WebSocket(`${WEBSOCKET_URL}/matches/${matchId}/play`);
}

/**
 * Generates a random token for player authentication.
 * Format: 8 random alphanumeric characters.
 */
export function generateToken(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  for (let i = 0; i < 8; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

/**
 * Generates a random player name.
 * Format: "Player_XXXX" where XXXX is 4 random characters.
 */
export function generatePlayerName(): string {
  const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
  let result = "Player_";
  for (let i = 0; i < 4; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

// Re-exports from the multiplayer module
export {
  getOrCreateCredentials,
  clearCredentials,
  hasExistingCredentials,
  initializeCredentialCleanup,
} from "./multiplayer/token";
export type { MatchCredentials } from "./multiplayer/token";

export { MatchConnection } from "./multiplayer/connection";
export type { ConnectionState, ConnectionCallbacks } from "./multiplayer/connection";

export { useMatch } from "./multiplayer/useMatch";
export type { MatchState, MatchPhase, MatchAction } from "./multiplayer/useMatch";
