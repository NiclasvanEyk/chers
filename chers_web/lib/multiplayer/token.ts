import { generateToken, generatePlayerName } from "../multiplayer";

const STORAGE_KEY_PREFIX = "chers_match_";

export interface MatchCredentials {
  token: string;
  playerName: string;
}

/**
 * Gets the storage key for a match.
 */
function getStorageKey(matchId: string): string {
  return `${STORAGE_KEY_PREFIX}${matchId}`;
}

/**
 * Retrieves or creates credentials for a match.
 * If credentials exist in sessionStorage, returns them (reconnecting player).
 * Otherwise, generates new credentials (new player).
 */
export function getOrCreateCredentials(matchId: string): MatchCredentials {
  const key = getStorageKey(matchId);
  const stored = sessionStorage.getItem(key);

  if (stored) {
    try {
      return JSON.parse(stored) as MatchCredentials;
    } catch {
      // Invalid stored data, generate new
    }
  }

  const credentials: MatchCredentials = {
    token: generateToken(),
    playerName: generatePlayerName(),
  };

  sessionStorage.setItem(key, JSON.stringify(credentials));
  return credentials;
}

/**
 * Clears stored credentials for a match.
 * Call this when leaving a match permanently.
 */
export function clearCredentials(matchId: string): void {
  const key = getStorageKey(matchId);
  sessionStorage.removeItem(key);
}

/**
 * Updates the player name for a match.
 * Call this when the player changes their name in the lobby.
 */
export function updatePlayerName(matchId: string, newName: string): void {
  const key = getStorageKey(matchId);
  const stored = sessionStorage.getItem(key);

  if (stored) {
    try {
      const credentials = JSON.parse(stored) as MatchCredentials;
      credentials.playerName = newName;
      sessionStorage.setItem(key, JSON.stringify(credentials));
    } catch {
      // Invalid stored data, ignore
    }
  }
}
