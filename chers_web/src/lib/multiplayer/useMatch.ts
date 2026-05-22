import { useCallback, useEffect, useMemo, useReducer, useRef } from "react";
import type { State as GameState, Coordinate, PromotedFigure, Color } from "@/generated/chers/chers";
import type { ConnectionState } from "./connection";
import { MatchConnection } from "./connection";
import { getOrCreateCredentials, clearCredentials, updatePlayerName } from "./token";
import type {
  ServerMessage,
  Event,
  RoomStateMirror,
  User,
  GameEndReason,
  GameEvent,
  LobbyEvent,
} from "./protocol";

export type MatchPhase =
  | { kind: "loading" }
  | { kind: "connecting" }
  | {
      kind: "waiting";
      inviteUrl: string;
      isReady: boolean;
      opponent: User | null;
      opponentIsReady: boolean;
    }
  | {
      kind: "playing";
      game: GameState;
      myTurn: boolean;
      opponent: User;
    }
  | {
      kind: "post_game";
      game: GameState;
      opponent: User;
      youWon: boolean;
      reason: GameEndReason;
    }
  | { kind: "reconnecting"; attempt: number; secondsRemaining: number }
  | { kind: "error"; message: string };

export interface MatchState {
  phase: MatchPhase;
  matchId: string;
  myColor: Color | null;
  myName: string | null;
  myUser: User | null;
  connectionStatus: ConnectionState["status"];
  gameOver: boolean;
}

export type MatchAction =
  | { type: "CONNECTION_STATE_CHANGE"; state: ConnectionState }
  | { type: "SERVER_MESSAGE"; message: ServerMessage }
  | { type: "SET_USER"; user: User }
  | { type: "GAME_OVER" }
  | { type: "DISCONNECT" };

function matchReducer(state: MatchState, action: MatchAction): MatchState {
  switch (action.type) {
    case "CONNECTION_STATE_CHANGE": {
      const connState = action.state;

      if (connState.status === "reconnecting") {
        if (state.gameOver) return state;
        const secondsRemaining = Math.max(0, 120 - connState.attempt * 15);
        return {
          ...state,
          connectionStatus: connState.status,
          phase: { kind: "reconnecting" as const, attempt: connState.attempt, secondsRemaining },
        };
      }

      return { ...state, connectionStatus: connState.status };
    }

    case "SERVER_MESSAGE": {
      const msg = action.message;
      if (msg.type === "event") return handleEvent(state, msg.payload);
      if (msg.type === "state") return handleState(state, msg.payload);
      if (msg.type === "error") {
        return { ...state, phase: { kind: "error" as const, message: msg.reason } };
      }
      return state;
    }

    case "SET_USER": {
      return { ...state, myUser: action.user, myName: action.user.name };
    }

    case "DISCONNECT": {
      return {
        ...state,
        connectionStatus: "closed",
        phase: { kind: "error" as const, message: "Disconnected from server" },
      };
    }

    default:
      return state;
  }
}

function handleEvent(state: MatchState, event: Event): MatchState {
  if ("Lobby" in event) return handleLobbyEvent(state, event.Lobby);
  if ("Game" in event) return handleGameEvent(state, event.Game);
  if ("System" in event && "CommandRejected" in event.System) {
    return { ...state, phase: { kind: "error" as const, message: event.System.CommandRejected.reason } };
  }
  return state;
}

function handleLobbyEvent(state: MatchState, event: LobbyEvent): MatchState {
  if ("PlayerJoined" in event) {
    const user = event.PlayerJoined.user;
    const isOwn = state.myUser?.id === user.id;

    if (isOwn) {
      return { ...state, connectionStatus: "open" };
    }

    return {
      ...state,
      connectionStatus: "open",
      phase: {
        kind: "waiting" as const,
        inviteUrl: `${window.location.origin}/multiplayer/${state.matchId}`,
        isReady: state.phase.kind === "waiting" ? state.phase.isReady : false,
        opponent: user,
        opponentIsReady: false,
      },
    };
  }

  if ("PlayerReadyChanged" in event && state.phase.kind === "waiting") {
    const isOwn = state.myUser?.id === event.PlayerReadyChanged.user.id;
    if (isOwn) {
      return { ...state, phase: { ...state.phase, isReady: event.PlayerReadyChanged.is_ready } };
    }
    return { ...state, phase: { ...state.phase, opponentIsReady: event.PlayerReadyChanged.is_ready } };
  }

  if ("PlayerNameChanged" in event && state.phase.kind === "waiting" && state.phase.opponent) {
    const isOpponent = state.phase.opponent.id === event.PlayerNameChanged.user.id;
    if (isOpponent) {
      return {
        ...state,
        phase: {
          ...state.phase,
          opponent: { ...state.phase.opponent, name: event.PlayerNameChanged.user.name },
        },
      };
    }
    return state;
  }

  if ("PlayerLeft" in event && state.phase.kind === "waiting") {
    return {
      ...state,
      phase: { ...state.phase, opponent: null, opponentIsReady: false },
    };
  }

  return state;
}

function handleGameEvent(state: MatchState, _event: GameEvent): MatchState {
  return state;
}

function handleState(state: MatchState, mirror: RoomStateMirror): MatchState {
  if ("Lobby" in mirror) {
    const { you, opponent, you_are_ready, opponent_is_ready } = mirror.Lobby;
    return {
      ...state,
      myUser: you,
      myName: you.name,
      connectionStatus: "open",
      phase: {
        kind: "waiting" as const,
        inviteUrl: `${window.location.origin}/multiplayer/${state.matchId}`,
        isReady: you_are_ready,
        opponent: opponent || null,
        opponentIsReady: opponent_is_ready,
      },
    };
  }

  if ("Game" in mirror) {
    const { you, your_color, opponent, board_state, is_your_turn } = mirror.Game;
    return {
      ...state,
      myUser: you,
      myName: you.name,
      myColor: your_color,
      gameOver: false,
      connectionStatus: "open",
      phase: {
        kind: "playing" as const,
        game: board_state,
        myTurn: is_your_turn,
        opponent,
      },
    };
  }

  if ("PostGame" in mirror) {
    const { you, your_color, opponent, you_won, reason, final_board } = mirror.PostGame;
    return {
      ...state,
      myUser: you,
      myName: you.name,
      myColor: your_color,
      gameOver: true,
      connectionStatus: "open",
      phase: {
        kind: "post_game" as const,
        game: final_board,
        opponent,
        youWon: you_won,
        reason,
      },
    };
  }

  return state;
}

const INITIAL_STATE = (matchId: string): MatchState => ({
  phase: { kind: "loading" },
  matchId,
  myColor: null,
  myName: null,
  myUser: null,
  connectionStatus: "connecting",
  gameOver: false,
});

function needsStateSync(payload: Event): boolean {
  if ("Game" in payload) {
    const game = payload.Game;
    return (
      "GameStarted" in game ||
      "Turn" in game ||
      "GameEnded" in game ||
      "PlayerReconnected" in game
    );
  }
  if ("PostGame" in payload) return true;
  return false;
}

export function useMatch(matchId: string) {
  const credentials = useMemo(() => getOrCreateCredentials(matchId), [matchId]);
  const [state, dispatch] = useReducer(matchReducer, matchId, INITIAL_STATE);

  const connectionRef = useRef<MatchConnection | null>(null);
  const syncPendingRef = useRef(false);
  const authConfirmedRef = useRef(false);
  const stateRef = useRef(state);
  stateRef.current = state;

  const handleServerMessage = useCallback((message: ServerMessage) => {
    dispatch({ type: "SERVER_MESSAGE", message });

    if (message.type === "event" && !authConfirmedRef.current) {
      authConfirmedRef.current = true;
      connectionRef.current?.setAuthenticated();

      const needsInitialSync =
        ("Lobby" in message.payload && "PlayerJoined" in message.payload.Lobby) ||
        ("Game" in message.payload && "PlayerReconnected" in message.payload.Game);

      if (needsInitialSync) {
        const user =
          "Lobby" in message.payload && "PlayerJoined" in message.payload.Lobby
            ? message.payload.Lobby.PlayerJoined.user
            : ("Game" in message.payload && "PlayerReconnected" in message.payload.Game
                ? message.payload.Game.PlayerReconnected.user
                : null);

        if (user) {
          syncPendingRef.current = true;
          connectionRef.current?.send({ RequestState: { user } });
        }
      }

      return;
    }

    if (message.type === "event" && needsStateSync(message.payload)) {
      if (!syncPendingRef.current) {
        syncPendingRef.current = true;
        const user = stateRef.current.myUser;
        if (user) {
          connectionRef.current?.send({ RequestState: { user } });
        }
      }
    }

    if (message.type === "state") {
      syncPendingRef.current = false;
    }
  }, []);

  const handleStateChange = useCallback((connState: ConnectionState) => {
    dispatch({ type: "CONNECTION_STATE_CHANGE", state: connState });
  }, []);

  useEffect(() => {
    if (state.connectionStatus === "reconnecting") {
      authConfirmedRef.current = false;
      syncPendingRef.current = false;
    }
  }, [state.connectionStatus]);

  useEffect(() => {
    // Close any stale connection before creating a new one.
    // This handles both Strict Mode (where the effect runs twice) and
    // navigation between different matches.
    if (connectionRef.current) {
      connectionRef.current.close();
      connectionRef.current = null;
    }

    dispatch({ type: "CONNECTION_STATE_CHANGE", state: { status: "connecting" } });

    const connection = new MatchConnection(matchId, credentials, {
      onMessage: handleServerMessage,
      onStateChange: handleStateChange,
    });

    connectionRef.current = connection;
    connection.connect();

    return () => {
      const oldConnection = connection;
      const oldRef = connectionRef;
      setTimeout(() => {
        oldConnection.close();
        if (oldRef.current === oldConnection) {
          oldRef.current = null;
        }
      }, 10000);
    };
  }, [matchId, credentials, handleServerMessage, handleStateChange]);

  const sendMove = useCallback(
    (from: Coordinate, to: Coordinate, promotion: PromotedFigure | null) => {
      if (state.phase.kind !== "playing" || !state.phase.myTurn) {
        console.error("Cannot move: not your turn");
        return;
      }
      connectionRef.current?.send({
        Game: { MakeMove: { move_: { from, to, promotion } } },
      });
    },
    [state.phase],
  );

  const sendUpdateName = useCallback(
    (newName: string) => {
      if (state.phase.kind !== "waiting") {
        console.error("Cannot update name: not in lobby");
        return;
      }
      if (newName.length === 0 || newName.length > 25) {
        console.error("Name must be 1-25 characters");
        return;
      }
      connectionRef.current?.send({
        Lobby: { ChangeName: { new_name: newName } },
      });
      updatePlayerName(state.matchId, newName);
    },
    [state.phase.kind, state.matchId],
  );

  const sendReady = useCallback(
    (ready: boolean) => {
      if (state.phase.kind !== "waiting") {
        console.error("Cannot toggle ready: not in lobby");
        return;
      }
      connectionRef.current?.send({
        Lobby: { ChangeReady: { is_ready: ready } },
      });
    },
    [state.phase.kind],
  );

  const disconnect = useCallback(() => {
    connectionRef.current?.close();
    clearCredentials(matchId);
    dispatch({ type: "DISCONNECT" });
  }, [matchId]);

  return {
    state,
    myName: state.myName || "",
    sendMove,
    sendUpdateName,
    sendReady,
    disconnect,
  };
}
