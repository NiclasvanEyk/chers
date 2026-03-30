import { generateToken, generatePlayerName } from "../multiplayer";

const STORAGE_KEY_PREFIX = "chers_match_";
const CREDENTIALS_TTL_MS = 24 * 60 * 60 * 1000; // 24 hours

export interface MatchCredentials {
  token: string;
  playerName: string;
  createdAt: number;
  lastAccessedAt: number;
}

/**
 * Gets the storage key for a match.
 */
function getStorageKey(matchId: string): string {
  return `${STORAGE_KEY_PREFIX}${matchId}`;
}

/**
 * Retrieves or creates credentials for a match.
 * If credentials exist in localStorage, returns them (reconnecting player).
 * Otherwise, generates new credentials (new player).
 */
export function getOrCreateCredentials(matchId: string): MatchCredentials {
  const key = getStorageKey(matchId);
  const stored = localStorage.getItem(key);

  if (stored) {
    try {
      const credentials = JSON.parse(stored) as MatchCredentials;
      // Update last accessed time
      credentials.lastAccessedAt = Date.now();
      localStorage.setItem(key, JSON.stringify(credentials));
      return credentials;
    } catch {
      // Invalid stored data, generate new
    }
  }

  const now = Date.now();
  const credentials: MatchCredentials = {
    token: generateToken(),
    playerName: generatePlayerName(),
    createdAt: now,
    lastAccessedAt: now,
  };

  localStorage.setItem(key, JSON.stringify(credentials));
  return credentials;
}

/**
 * Clears stored credentials for a match.
 * Call this when leaving a match permanently.
 */
export function clearCredentials(matchId: string): void {
  const key = getStorageKey(matchId);
  localStorage.removeItem(key);
}

/**
 * Updates the player name for a match.
 * Call this when the player changes their name in the lobby.
 */
export function updatePlayerName(matchId: string, newName: string): void {
  const key = getStorageKey(matchId);
  const stored = localStorage.getItem(key);

  if (stored) {
    try {
      const credentials = JSON.parse(stored) as MatchCredentials;
      credentials.playerName = newName;
      credentials.lastAccessedAt = Date.now();
      localStorage.setItem(key, JSON.stringify(credentials));
    } catch {
      // Invalid stored data, ignore
    }
  }
}

/**
 * Checks if credentials exist for a match.
 * Useful for determining if we're reconnecting to an existing match.
 */
export function hasExistingCredentials(matchId: string): boolean {
  const key = getStorageKey(matchId);
  const stored = localStorage.getItem(key);
  return stored !== null;
}

/**
 * Cleans up expired credentials from localStorage.
 * Removes all match credentials older than 24 hours.
 */
function cleanupExpiredCredentials(): void {
  const now = Date.now();
  const keysToRemove: string[] = [];

  // Scan all localStorage keys
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith(STORAGE_KEY_PREFIX)) {
      try {
        const stored = localStorage.getItem(key);
        if (stored) {
          const credentials = JSON.parse(stored) as MatchCredentials;
          // Use lastAccessedAt if available, fallback to createdAt
          const lastActive = credentials.lastAccessedAt ?? credentials.createdAt;
          if (now - lastActive > CREDENTIALS_TTL_MS) {
            keysToRemove.push(key);
          }
        }
      } catch {
        // Invalid data, mark for removal
        keysToRemove.push(key);
      }
    }
  }

  // Remove expired credentials
  keysToRemove.forEach(key => {
    localStorage.removeItem(key);
    console.log(`🧹 Cleaned up expired match credentials: ${key}`);
  });

  if (keysToRemove.length > 0) {
    console.log(`🧹 Cleaned up ${keysToRemove.length} expired match credential(s)`);
  }
}

/**
 * Initializes credential cleanup on app startup.
 * Defers execution to avoid blocking initial render.
 */
export function initializeCredentialCleanup(): void {
  const doCleanup = () => {
    try {
      cleanupExpiredCredentials();
    } catch (error) {
      console.error("Failed to cleanup expired credentials:", error);
    }
  };

  // Use requestIdleCallback if available (Chrome), otherwise fallback to setTimeout
  if (typeof window !== "undefined") {
    if ("requestIdleCallback" in window) {
      window.requestIdleCallback(doCleanup, { timeout: 2000 });
    } else {
      setTimeout(doCleanup, 0);
    }
  }
}
