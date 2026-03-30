import { useCallback, useEffect, useReducer, useRef } from "react";
import type { State as GameState, Coordinate, PromotedFigure } from "@/generated/chers/chers";
import type { Color } from "@/generated/chers/chers";
import type { ServerMessage } from "@/generated/chers_server_api/ServerMessage";
import type { PublicEvent } from "@/generated/chers_server_api/PublicEvent";
import type { PrivateEvent } from "@/generated/chers_server_api/PrivateEvent";
import type { GameResult } from "@/generated/chers_server_api/GameResult";
import type { GameEndReason } from "@/generated/chers_server_api/GameEndReason";
import type { PlayerInfo } from "@/generated/chers_server_api/PlayerInfo";
import type { AuthFailureReason } from "@/generated/chers_server_api/AuthFailureReason";
import type { ConnectionState } from "./connection";
import { MatchConnection } from "./connection";
import { getOrCreateCredentials, clearCredentials, updatePlayerName } from "./token";
import type { ClientMessage } from "@/generated/chers_server_api/ClientMessage";
import type { PromotionPiece } from "@/generated/chers_server_api/PromotionPiece";

// ============================================================================
// Types
// ============================================================================

export type MatchPhase =
  | { kind: "loading" }
  | { kind: "connecting" }
  | { kind: "authenticating" }
  | { kind: "auth_error"; reason: AuthFailureReason }
  | { kind: "match_not_found" }
  | {
      kind: "waiting";
      inviteUrl: string;
      isReady: boolean;
      opponentName: string | null;
      opponentReady: boolean;
      countdown: number | null;
    }
  | {
      kind: "game_starting";  // NEW: Game started but waiting for color assignment
      game: GameState;
      white_player: PlayerInfo;
      black_player: PlayerInfo;
    }
  | {
      kind: "playing";
      game: GameState;
      myTurn: boolean;
      opponent: PlayerInfo;
    }
  | {
      kind: "reconnecting";
      attempt: number;
      secondsRemaining: number;
    }
  | {
      kind: "game_over";
      result: GameResult;
      reason: GameEndReason;
    }
  | { kind: "error"; message: string };

export interface MatchState {
  phase: MatchPhase;
  matchId: string;
  myColor: Color | null;
  myName: string | null;
  mySlot: number | null; // 1 or 2 - use this to identify self in lobby events
  connectionStatus: ConnectionState["status"];
  gameOver: boolean; // Track if game has ended to prevent reconnection
}

export type MatchAction =
  | { type: "CONNECTION_STATE_CHANGE"; state: ConnectionState }
  | { type: "SERVER_MESSAGE"; message: ServerMessage }
  | { type: "SET_MY_COLOR"; color: Color }
  | { type: "GAME_OVER" }
  | { type: "DISCONNECT" };

// ============================================================================
// Reducer
// ============================================================================

function matchReducer(state: MatchState, action: MatchAction): MatchState {
  switch (action.type) {
    case "CONNECTION_STATE_CHANGE": {
      const { state: connState } = action;
      
      if (connState.status === "reconnecting") {
        // Don't reconnect if game is already over
        if (state.gameOver) {
          return state;
        }
        
        // Calculate seconds remaining in grace period
        const secondsRemaining = Math.max(0, 120 - connState.attempt * 15);
        return {
          ...state,
          connectionStatus: connState.status,
          phase: { kind: "reconnecting", attempt: connState.attempt, secondsRemaining },
        };
      }

      return {
        ...state,
        connectionStatus: connState.status,
      };
    }

    case "SERVER_MESSAGE": {
      const { message } = action;
      console.log("📨 SERVER_MESSAGE reducer processing:", message.visibility, message.event.kind);

      if (message.visibility === "Private") {
        return handlePrivateEvent(state, message.event);
      } else {
        return handlePublicEvent(state, message.event);
      }
    }

    case "SET_MY_COLOR": {
      return {
        ...state,
        myColor: action.color,
      };
    }

    case "DISCONNECT": {
      return {
        ...state,
        connectionStatus: "closed",
        phase: { kind: "error", message: "Disconnected from server" },
      };
    }

    default:
      return state;
  }
}

function handlePrivateEvent(state: MatchState, event: PrivateEvent): MatchState {
  console.log("🔐 handlePrivateEvent received:", event.kind, "Current phase:", state.phase.kind);
  
  switch (event.kind) {
    case "LobbyJoined": {
      console.log("🚪 LobbyJoined event received! Slot:", event.slot);
      console.log("👥 Players in lobby:", event.player1, event.player2);
      
      // Determine opponent info based on our slot
      let opponentName = null;
      let opponentReady = false;
      let isReady = false;
      
      if (event.slot === 1) {
        // We are player 1, opponent is player 2
        opponentName = event.player2?.name || null;
        opponentReady = event.player2_ready;
        isReady = event.player1_ready;
      } else {
        // We are player 2, opponent is player 1
        opponentName = event.player1?.name || null;
        opponentReady = event.player1_ready;
        isReady = event.player2_ready;
      }
      
      // Player is in lobby waiting for opponent
      return {
        ...state,
        mySlot: event.slot, // Store our slot number (1 or 2)
        myColor: null, // Color not assigned yet
        phase: {
          kind: "waiting",
          inviteUrl: `${window.location.origin}/multiplayer/${state.matchId}`,
          isReady,
          opponentName,
          opponentReady,
          countdown: null,
        },
      };
    }

    case "ColorsAssigned": {
      console.log("🎨 ColorsAssigned event received! Player color:", event.player, "Current phase:", state.phase.kind);
      
      const isReconnecting = state.phase.kind === "reconnecting";
      
      if (isReconnecting) {
        // Will receive StateSync next to restore game state
        return {
          ...state,
          myColor: event.player,
          phase: { kind: "connecting" },
        };
      }

      // If we receive ColorsAssigned but already in game_starting phase, transition to playing
      if (state.phase.kind === "game_starting") {
        console.log("🎮 Transitioning from game_starting to playing with color:", event.player);
        const gamePhase = state.phase as Extract<MatchPhase, { kind: "game_starting" }>;
        const myColor = event.player;
        const isMyTurn = gamePhase.game.player === myColor;
        const opponent = myColor === "White" ? gamePhase.black_player : gamePhase.white_player;

        return {
          ...state,
          myColor: myColor,
          phase: {
            kind: "playing",
            game: gamePhase.game,
            myTurn: isMyTurn,
            opponent,
          },
        };
      }

      // If we're waiting or in any other phase, store the color
      // The StateSync effect will detect we need game state and request it
      console.log("🎨 Got color", event.player, "while in phase:", state.phase.kind, "- waiting for game state");
      return {
        ...state,
        myColor: event.player,
      };
    }

    case "AuthenticationFailed": {
      if (event.reason === "MatchNotFound") {
        return {
          ...state,
          phase: { kind: "match_not_found" },
        };
      }

      return {
        ...state,
        phase: { kind: "auth_error", reason: event.reason },
      };
    }

    case "MoveRejected": {
      console.error("Move rejected:", event.reason);
      return state;
    }

    case "StateSync": {
      // Restore full state on reconnect
      // Use your_color from the event to know which color we are
      const myColor = event.your_color;
      
      // Check if game is already over
      if (event.game_result && event.game_end_reason) {
        console.log("🏁 StateSync received for finished game!");
        return {
          ...state,
          myColor: myColor,
          phase: {
            kind: "game_over",
            result: event.game_result,
            reason: event.game_end_reason,
          },
          gameOver: true,
        };
      }
      
      const isMyTurn = event.current_turn === myColor;
      const opponent = myColor === "White" ? event.black_player : event.white_player;

      return {
        ...state,
        myColor: myColor,  // Set color from StateSync
        phase: {
          kind: "playing",
          game: event.game_state,
          myTurn: isMyTurn,
          opponent,
        },
      };
    }

    default:
      return state;
  }
}

function handlePublicEvent(state: MatchState, event: PublicEvent): MatchState {
  console.log("📢 handlePublicEvent received:", event.kind, "Current phase:", state.phase.kind);
  
  switch (event.kind) {
    case "GameStarted": {
      console.log("🎮 GameStarted received! Current phase:", state.phase.kind, "myColor:", state.myColor);
      
      // Check if we already have a color assigned (ColorsAssigned arrived first)
      if (state.myColor) {
        console.log("🎮 ColorsAssigned already received! Transitioning directly to playing");
        const myColor = state.myColor;
        const isMyTurn = event.game_state.player === myColor;
        const opponent = myColor === "White" ? event.black_player : event.white_player;

        return {
          ...state,
          phase: {
            kind: "playing",
            game: event.game_state,
            myTurn: isMyTurn,
            opponent,
          },
        };
      }
      
      // No color yet - wait for ColorsAssigned
      console.log("⏳ Waiting for ColorsAssigned...");
      return {
        ...state,
        phase: {
          kind: "game_starting",
          game: event.game_state,
          white_player: event.white_player,
          black_player: event.black_player,
        },
      };
    }

    case "MoveMade": {
      if (state.phase.kind !== "playing") return state;

      const myColor = state.myColor;
      if (!myColor) return state;

      const currentPhase = state.phase as Extract<MatchPhase, { kind: "playing" }>;
      const isMyTurn = event.new_state.player === myColor;

      return {
        ...state,
        phase: {
          ...currentPhase,
          game: event.new_state,
          myTurn: isMyTurn,
        },
      };
    }

    case "GameOver": {
      return {
        ...state,
        phase: {
          kind: "game_over",
          result: event.result,
          reason: event.reason,
        },
        gameOver: true, // Mark game as over
      };
    }

    case "PlayerStatusChanged": {
      if (state.phase.kind !== "playing") return state;

      const myColor = state.myColor;
      if (!myColor) return state;

      const currentPhase = state.phase as Extract<MatchPhase, { kind: "playing" }>;
      
      // Update opponent connection status
      const opponentColor = myColor === "White" ? "Black" : "White";
      if (event.player === opponentColor) {
        return {
          ...state,
          phase: {
            ...currentPhase,
            opponent: {
              ...currentPhase.opponent,
              connected: event.status === "Connected",
            },
          },
        };
      }

      return state;
    }

    case "NameChanged": {
      console.log("✏️ NameChanged received:", event.player, "slot:", event.slot, "name:", event.name);
      
      // Ignore our own name changes - only update opponent name
      if (event.slot === state.mySlot) {
        console.log("🙉 Ignoring own name change");
        return state;
      }
      
      // Update opponent name if in waiting phase
      if (state.phase.kind === "waiting") {
        const currentPhase = state.phase as Extract<MatchPhase, { kind: "waiting" }>;
        console.log("📝 Updating opponent name to:", event.name);
        return {
          ...state,
          phase: {
            ...currentPhase,
            opponentName: event.name,
          },
        };
      }
      
      // Update opponent name if in playing phase
      if (state.phase.kind === "playing" && state.myColor) {
        const opponentColor = state.myColor === "White" ? "Black" : "White";
        if (event.player === opponentColor) {
          const currentPhase = state.phase as Extract<MatchPhase, { kind: "playing" }>;
          return {
            ...state,
            phase: {
              ...currentPhase,
              opponent: {
                ...currentPhase.opponent,
                name: event.name,
              },
            },
          };
        }
      }
      
      return state;
    }

    case "PlayerReady": {
      console.log("✅ PlayerReady received:", event.player, "slot:", event.slot, "ready:", event.ready);
      
      if (state.phase.kind === "waiting") {
        const currentPhase = state.phase as Extract<MatchPhase, { kind: "waiting" }>;
        
        // Use slot to determine if it's our ready status or opponent's
        const isOurReady = event.slot === state.mySlot;
        
        return {
          ...state,
          phase: {
            ...currentPhase,
            isReady: isOurReady ? event.ready : currentPhase.isReady,
            opponentReady: !isOurReady ? event.ready : currentPhase.opponentReady,
          },
        };
      }
      
      return state;
    }

    case "Countdown": {
      console.log("⏱️ Countdown received:", event.seconds);
      
      if (state.phase.kind === "waiting") {
        const currentPhase = state.phase as Extract<MatchPhase, { kind: "waiting" }>;
        return {
          ...state,
          phase: {
            ...currentPhase,
            countdown: event.seconds,
          },
        };
      }
      
      return state;
    }

    case "PlayerLeftLobby": {
      console.log("👋 PlayerLeftLobby received: slot", event.slot);
      
      // Only handle if we're in the waiting phase and it's our opponent who left
      if (state.phase.kind === "waiting" && event.slot !== state.mySlot) {
        const currentPhase = state.phase as Extract<MatchPhase, { kind: "waiting" }>;
        console.log("📝 Opponent left lobby, clearing opponent info");
        return {
          ...state,
          phase: {
            ...currentPhase,
            opponentName: null,
            opponentReady: false,
            countdown: null,
          },
        };
      }
      
      return state;
    }

    case "GameStarting": {
      console.log("🚀 GameStarting received!");
      // Transition to game_starting phase (already handled by ColorsAssigned)
      return state;
    }

    default:
      return state;
  }
}

// ============================================================================
// Hook
// ============================================================================

const INITIAL_STATE = (matchId: string, myName: string): MatchState => ({
  phase: { kind: "loading" },
  matchId,
  myColor: null,
  myName,
  mySlot: null,
  connectionStatus: "connecting",
  gameOver: false,
});

export function useMatch(matchId: string) {
  const credentials = getOrCreateCredentials(matchId);
  const myName = credentials.playerName;
  
  const [state, dispatch] = useReducer(
    matchReducer,
    matchId,
    (id) => INITIAL_STATE(id, myName)
  );

  const connectionRef = useRef<MatchConnection | null>(null);
  const credentialsRef = useRef(credentials);

  // Initialize connection
  useEffect(() => {
    // Prevent multiple connection attempts
    if (connectionRef.current) {
      return;
    }
    
    dispatch({ type: "CONNECTION_STATE_CHANGE", state: { status: "connecting" } });

    const connection = new MatchConnection(matchId, {
      onMessage: (message) => {
        console.log("📨 useMatch received message:", message);
        dispatch({ type: "SERVER_MESSAGE", message });
      },
      onStateChange: (state) => {
        console.log("📡 Connection state changed:", state);
        dispatch({ type: "CONNECTION_STATE_CHANGE", state });
      },
    });

    connectionRef.current = connection;
    connection.connect();

    return () => {
      // Delay cleanup to prevent React Strict Mode double-mount from closing connection
      setTimeout(() => {
        connection.close();
        connectionRef.current = null;
      }, 500);
    };
  }, [matchId]);

  // Send authentication when connection opens
  useEffect(() => {
    if (state.connectionStatus === "open") {
      const credentials = credentialsRef.current;
      const authMessage: ClientMessage = {
        kind: "Authenticate",
        token: credentials.token,
        name: credentials.playerName,
      };
      connectionRef.current?.send(authMessage);
    }
  }, [state.connectionStatus]);

  // Proactive state sync - request full state if we're in an inconsistent state
  useEffect(() => {
    if (state.connectionStatus !== "open") return;

    // If we have color but no game state, or game state but no color, request sync
    const hasColor = state.myColor !== null;
    const hasGameState = state.phase.kind === "playing" || state.phase.kind === "game_starting";
    
    if ((hasColor && !hasGameState) || (!hasColor && hasGameState)) {
      console.log("🔄 Inconsistent state detected - requesting StateSync");
      const syncMessage: ClientMessage = { kind: "RequestSync" };
      connectionRef.current?.send(syncMessage);
    }
  }, [state.myColor, state.phase.kind, state.connectionStatus]);

  // Handle move sending
  const sendMove = useCallback((from: Coordinate, to: Coordinate, promotion: PromotedFigure | null) => {
    if (state.phase.kind !== "playing" || !state.phase.myTurn) {
      console.error("Cannot move: not your turn");
      return;
    }

    const promotionPiece: PromotionPiece | null = promotion === null ? null : 
      promotion === "Queen" ? "Q" :
      promotion === "Rook" ? "R" :
      promotion === "Bishop" ? "B" :
      promotion === "Knight" ? "N" : null;

    const moveMessage: ClientMessage = {
      kind: "MakeMove",
      from,
      to,
      promotion: promotionPiece,
    };

    connectionRef.current?.send(moveMessage);
  }, [state.phase]);

  // Handle name update (only in lobby)
  const sendUpdateName = useCallback((newName: string) => {
    if (state.phase.kind !== "waiting") {
      console.error("Cannot update name: not in lobby");
      return;
    }

    // Validate name length
    if (newName.length === 0) {
      console.error("Name cannot be empty");
      return;
    }
    if (newName.length > 25) {
      console.error("Name must be 25 characters or less");
      return;
    }

    const updateMessage: ClientMessage = {
      kind: "UpdateName",
      name: newName,
    };

    console.log("✏️ Sending name update:", newName);
    connectionRef.current?.send(updateMessage);
    
    // Update name in sessionStorage for persistence
    updatePlayerName(state.matchId, newName);
  }, [state.phase.kind, state.matchId]);

  // Handle ready toggle (only in lobby)
  const sendReady = useCallback((ready: boolean) => {
    if (state.phase.kind !== "waiting") {
      console.error("Cannot toggle ready: not in lobby");
      return;
    }

    const readyMessage: ClientMessage = {
      kind: "Ready",
      ready,
    };

    console.log("✅ Sending ready status:", ready);
    connectionRef.current?.send(readyMessage);
  }, [state.phase.kind]);

  // Handle disconnect
  const disconnect = useCallback(() => {
    connectionRef.current?.close();
    clearCredentials(matchId);
    dispatch({ type: "DISCONNECT" });
  }, [matchId]);

  return {
    state,
    myName,
    sendMove,
    sendUpdateName,
    sendReady,
    disconnect,
  };
}
