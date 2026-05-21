import { play } from "../multiplayer";
import type { ServerMessage } from "./protocol";
import type { MatchCredentials } from "./token";

export type ConnectionState =
  | { status: "connecting" }
  | { status: "open" }
  | { status: "reconnecting"; attempt: number; nextRetryIn: number }
  | { status: "error"; message: string }
  | { status: "closed"; reason: string };

export interface ConnectionCallbacks {
  onMessage: (message: ServerMessage) => void;
  onStateChange: (state: ConnectionState) => void;
}

const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 16000];
const MAX_RECONNECT_DELAY = 30000;
const GRACE_PERIOD_MS = 120000;

export class MatchConnection {
  private matchId: string;
  private credentials: MatchCredentials;
  private socket: WebSocket | null = null;
  private callbacks: ConnectionCallbacks;
  private reconnectAttempt = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private gracePeriodTimer: NodeJS.Timeout | null = null;
  private intentionallyClosed = false;
  private messagesQueue: string[] = [];
  private authenticated = false;

  constructor(matchId: string, credentials: MatchCredentials, callbacks: ConnectionCallbacks) {
    this.matchId = matchId;
    this.credentials = credentials;
    this.callbacks = callbacks;
  }

  connect(): void {
    this.intentionallyClosed = false;
    this.authenticated = false;
    this.callbacks.onStateChange({ status: "connecting" });

    try {
      this.socket = play(this.matchId);

      this.socket.onopen = () => {
        this.reconnectAttempt = 0;

        this.socket!.send(
          JSON.stringify({
            type: "authenticate",
            secret: this.credentials.token,
            name: this.credentials.playerName,
          }),
        );
      };

      this.socket.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as ServerMessage;
          console.debug("[WS] received:", JSON.stringify(message));
          this.callbacks.onMessage(message);
        } catch (err) {
          console.error("Failed to parse WebSocket message:", err, event.data);
        }
      };

      this.socket.onerror = () => {
        this.callbacks.onStateChange({
          status: "error",
          message: "Connection error occurred",
        });
      };

      this.socket.onclose = () => {
        if (this.intentionallyClosed) {
          this.callbacks.onStateChange({
            status: "closed",
            reason: "Connection closed",
          });
          return;
        }

        this.startGracePeriod();
        this.scheduleReconnect();
      };
    } catch (err) {
      this.callbacks.onStateChange({
        status: "error",
        message: `Failed to connect: ${err}`,
      });
    }
  }

  setAuthenticated(): void {
    this.authenticated = true;
    this.callbacks.onStateChange({ status: "open" });
    this.flushMessageQueue();
    this.clearGracePeriod();
  }

  private scheduleReconnect(): void {
    const delay =
      this.reconnectAttempt < RECONNECT_DELAYS.length
        ? RECONNECT_DELAYS[this.reconnectAttempt]
        : MAX_RECONNECT_DELAY;

    this.reconnectAttempt++;

    this.callbacks.onStateChange({
      status: "reconnecting",
      attempt: this.reconnectAttempt,
      nextRetryIn: Math.ceil(delay / 1000),
    });

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private startGracePeriod(): void {
    this.gracePeriodTimer = setTimeout(() => {
      this.cleanup();
    }, GRACE_PERIOD_MS);
  }

  private clearGracePeriod(): void {
    if (this.gracePeriodTimer) {
      clearTimeout(this.gracePeriodTimer);
      this.gracePeriodTimer = null;
    }
  }

  send(message: object): void {
    const json = JSON.stringify({ type: "command", payload: message });
    console.debug("[WS] sending:", json);
    if (this.socket?.readyState === WebSocket.OPEN && this.authenticated) {
      this.socket.send(json);
    } else {
      this.messagesQueue.push(json);
    }
  }

  private flushMessageQueue(): void {
    while (
      this.messagesQueue.length > 0 &&
      this.socket?.readyState === WebSocket.OPEN
    ) {
      const json = this.messagesQueue.shift();
      if (json) {
        this.socket.send(json);
      }
    }
  }

  close(): void {
    this.intentionallyClosed = true;
    this.cleanup();
  }

  private cleanup(): void {
    this.clearGracePeriod();

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }

    this.messagesQueue = [];
  }

  getReconnectAttempt(): number {
    return this.reconnectAttempt;
  }
}
