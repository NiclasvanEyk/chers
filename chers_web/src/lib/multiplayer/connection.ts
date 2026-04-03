import { play } from "../multiplayer";
import type { ServerMessage } from "@/generated/chers_server_api/ServerMessage";
import type { ClientMessage } from "@/generated/chers_server_api/ClientMessage";

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

const RECONNECT_DELAYS = [1000, 2000, 4000, 8000, 16000]; // Conservative backoff: 1s, 2s, 4s, 8s, 16s
const MAX_RECONNECT_DELAY = 30000; // Then every 30s
const GRACE_PERIOD_MS = 120000; // 2 minutes

export class MatchConnection {
  private matchId: string;
  private socket: WebSocket | null = null;
  private callbacks: ConnectionCallbacks;
  private reconnectAttempt = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private gracePeriodTimer: NodeJS.Timeout | null = null;
  private intentionallyClosed = false;
  private messagesQueue: ClientMessage[] = [];

  constructor(matchId: string, callbacks: ConnectionCallbacks) {
    this.matchId = matchId;
    this.callbacks = callbacks;
  }

  connect(): void {
    this.intentionallyClosed = false;
    this.callbacks.onStateChange({ status: "connecting" });
    console.log("🔌 MatchConnection.connect() called for match:", this.matchId);

    try {
      this.socket = play(this.matchId);
      console.log("🔌 WebSocket object created, readyState:", this.socket.readyState);

      this.socket.onopen = () => {
        console.log("✅ WebSocket opened successfully");
        this.reconnectAttempt = 0;
        this.callbacks.onStateChange({ status: "open" });
        this.flushMessageQueue();
        this.startHeartbeat();
        this.clearGracePeriod();
      };

      this.socket.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data) as ServerMessage;
          console.log("📨 WebSocket message received:", message);
          this.callbacks.onMessage(message);
        } catch (err) {
          console.error("❌ Failed to parse WebSocket message:", err, event.data);
        }
      };

      this.socket.onerror = (error) => {
        console.error("WebSocket error:", error);
        this.callbacks.onStateChange({
          status: "error",
          message: "Connection error occurred",
        });
      };

      this.socket.onclose = (event) => {
        this.stopHeartbeat();

        if (this.intentionallyClosed) {
          this.callbacks.onStateChange({
            status: "closed",
            reason: "Connection closed",
          });
          return;
        }

        // Start grace period countdown
        this.startGracePeriod();

        // Attempt reconnection with backoff
        this.scheduleReconnect();
      };
    } catch (err) {
      this.callbacks.onStateChange({
        status: "error",
        message: `Failed to connect: ${err}`,
      });
    }
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
    // After 2 minutes without successful reconnect, game will end
    this.gracePeriodTimer = setTimeout(() => {
      // Don't try to reconnect anymore, game has ended
      this.cleanup();
    }, GRACE_PERIOD_MS);
  }

  private clearGracePeriod(): void {
    if (this.gracePeriodTimer) {
      clearTimeout(this.gracePeriodTimer);
      this.gracePeriodTimer = null;
    }
  }

  private startHeartbeat(): void {
    // Send heartbeat every 30 seconds to keep connection alive
    this.heartbeatTimer = setInterval(() => {
      this.send({ kind: "Heartbeat" });
    }, 30000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  send(message: ClientMessage): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(message));
    } else {
      // Queue message for when connection is restored
      this.messagesQueue.push(message);
    }
  }

  private flushMessageQueue(): void {
    while (this.messagesQueue.length > 0 && this.socket?.readyState === WebSocket.OPEN) {
      const message = this.messagesQueue.shift();
      if (message) {
        this.socket.send(JSON.stringify(message));
      }
    }
  }

  close(): void {
    this.intentionallyClosed = true;
    this.cleanup();
  }

  private cleanup(): void {
    this.stopHeartbeat();
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
