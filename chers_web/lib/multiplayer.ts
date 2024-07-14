function parseBoolean(value: unknown) {
  if (value === "true") return true;
  if (value === "false") return false;
  return undefined;
}

const SERVER_HOST =
  process.env.NEXT_PUBLIC_CHERS_SERVER_HOST ?? "server.chers.niclasve.me";
const USE_SSL = parseBoolean(process.env.NEXT_PUBLIC_CHERS_USE_SSL) ?? true;
const SERVER_URL = `${USE_SSL ? "https" : "http"}://${SERVER_HOST}`;
const WEBSOCKET_URL = `${USE_SSL ? "wss" : "ws"}://${SERVER_HOST}`;

/**
 * Requests the server to start a new game.
 */
export async function startNewMatch(): Promise<number> {
  const response = await fetch(`${SERVER_URL}/matches/new`, { method: "POST" });
  const body = await response.json();
  const id = body.id;

  if (!Number.isInteger(id)) {
    throw new Error(
      `Server did not respond with a valid match id! Got: ${id} `,
    );
  }

  return id;
}

/**
 * Builds up the websocket object for a given match.
 */
export function play(matchId: number) {
  return new WebSocket(`${WEBSOCKET_URL}/matches/${matchId}/play`);
}
