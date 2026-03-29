"use client";

import { useReducer, useCallback, useEffect } from "react";
import { Board } from "@/components/Board";
import { PlayerBar } from "./PlayerBar";
import { ChersSettingsProvider } from "@/lib/settings";
import { Settings, SettingsTrigger } from "@/components/Settings";
import { useState } from "react";
import type { State as GameState, Coordinate, PromotedFigure, Color, Piece } from "@/generated/chers/chers";
import type { PlayerInfo } from "@/generated/chers_server_api/PlayerInfo";
import { getMoves } from "@/lib/chers";
import { canPickUp } from "@/lib/ui/state";
import { useMatchCredentials } from "@/lib/multiplayer/token";

// ============================================================================
// Local Multiplayer State (for piece selection only)
// ============================================================================

type MultiplayerLocalState =
  | { type: "SELECTING_FROM"; game: GameState }
  | {
      type: "SELECTING_TO";
      game: GameState;
      from: Coordinate;
      piece: Piece;
      moves: Coordinate[];
    }
  | {
      type: "PROMOTING";
      game: GameState;
      from: Coordinate;
      to: Coordinate;
    };

type MultiplayerCommand =
  | { type: "SYNC_GAME"; game: GameState }
  | { type: "SELECT_FROM"; from: Coordinate }
  | { type: "ABORT_SELECTION" }
  | { type: "SELECT_TO"; to: Coordinate }
  | { type: "PROMOTE"; promotion: PromotedFigure };

function multiplayerReducer(
  state: MultiplayerLocalState,
  command: MultiplayerCommand,
): MultiplayerLocalState {
  switch (command.type) {
    case "SYNC_GAME": {
      // Reset to initial selection state with new game
      return { type: "SELECTING_FROM", game: command.game };
    }

    case "SELECT_FROM": {
      if (state.type !== "SELECTING_FROM") {
        // If already selecting, just switch selection
        state = { type: "SELECTING_FROM", game: state.game };
      }

      const { from } = command;
      const piece = state.game.board[from.y][from.x];
      
      if (!piece) {
        return state;
      }

      const moves = getMoves(state.game, from);
      return { type: "SELECTING_TO", game: state.game, from, piece, moves };
    }

    case "ABORT_SELECTION": {
      return { type: "SELECTING_FROM", game: state.game };
    }

    case "SELECT_TO": {
      if (state.type !== "SELECTING_TO") {
        return state;
      }

      const { to } = command;
      
      // Check if this is a promotion
      const isPromotion =
        state.piece.figure === "Pawn" && (to.y === 0 || to.y === 7);

      if (isPromotion) {
        return {
          type: "PROMOTING",
          game: state.game,
          from: state.from,
          to,
        };
      }

      // For non-promotion moves, selection resets after move is sent
      return { type: "SELECTING_FROM", game: state.game };
    }

    case "PROMOTE": {
      if (state.type !== "PROMOTING") {
        return state;
      }

      return { type: "SELECTING_FROM", game: state.game };
    }

    default:
      return state;
  }
}

// ============================================================================
// Component
// ============================================================================

interface MultiplayerGameProps {
  game: GameState;
  myColor: Color;
  myTurn: boolean;
  myName: string;
  opponent: PlayerInfo;
  onMove: (from: Coordinate, to: Coordinate, promotion: PromotedFigure | null) => void;
}

export function MultiplayerGame({
  game,
  myColor,
  myTurn,
  myName,
  opponent,
  onMove,
}: MultiplayerGameProps) {
  const [settingsOpen, setSettingsOpen] = useState(false);
  
  const [localState, dispatch] = useReducer(
    multiplayerReducer,
    { type: "SELECTING_FROM", game }
  );

  // Sync local state when game changes from server
  useEffect(() => {
    if (localState.game !== game) {
      dispatch({ type: "SYNC_GAME", game });
    }
  }, [game, localState.game]);

  // Custom dispatch that intercepts moves and sends to server
  const handleDispatch = useCallback(
    (action: { type: string; from?: Coordinate; to?: Coordinate | PromotedFigure }) => {
      if (!myTurn) return;

      switch (action.type) {
        case "SELECT_FROM":
          if (action.from) {
            // Only allow selecting own pieces
            const piece = game.board[action.from.y][action.from.x];
            if (piece && canPickUp(piece, myColor)) {
              dispatch({ type: "SELECT_FROM", from: action.from });
            }
          }
          break;

        case "ABORT_SELECTION":
          dispatch({ type: "ABORT_SELECTION" });
          break;

        case "SELECT_TO":
          if (action.to && typeof action.to === "object" && "x" in action.to && localState.type === "SELECTING_TO") {
            const to = action.to as Coordinate;
            const isValidMove = localState.moves.some(
              (m) => m.x === to.x && m.y === to.y
            );
            
            if (isValidMove) {
              // Check for promotion
              const isPromotion =
                localState.piece.figure === "Pawn" &&
                (to.y === 0 || to.y === 7);

              if (isPromotion) {
                // Enter PROMOTING state (don't send yet)
                dispatch({ type: "SELECT_TO", to });
              } else {
                // Send move immediately
                onMove(localState.from, to, null);
                dispatch({ type: "SELECT_TO", to });
              }
            }
          }
          break;

        case "PROMOTE":
          // This comes from the Promotion component
          if (localState.type === "PROMOTING") {
            const promotion = action.to as PromotedFigure;
            onMove(localState.from, localState.to, promotion);
            dispatch({ type: "PROMOTE", promotion });
          }
          break;
      }
    },
    [myTurn, game, myColor, localState, onMove]
  );

  return (
    <ChersSettingsProvider>
      <div className="min-h-screen p-4">
        <SettingsTrigger
          onClick={() => setSettingsOpen(true)}
          className="fixed top-3 left-3 z-10"
          color={myColor}
        />
        <Settings open={settingsOpen} onClose={() => setSettingsOpen(false)} />

        <div className="max-w-2xl mx-auto">
          <PlayerBar
            myColor={myColor}
            myName={myName}
            opponent={opponent}
            isMyTurn={myTurn}
          />

          <div className="relative touch-manipulation">
            {/* Block interaction when not my turn */}
            {!myTurn && (
              <div 
                className="absolute inset-0 z-10" 
                style={{ pointerEvents: "auto" }}
                aria-label="Waiting for opponent"
              />
            )}
            
            <Board 
              state={localState as any} 
              dispatch={handleDispatch as any} 
            />
          </div>

          {!myTurn && opponent.connected && (
            <div className="mt-4 text-center text-gray-600 dark:text-gray-400">
              Waiting for opponent&apos;s move...
            </div>
          )}
          
          {!opponent.connected && (
            <div className="mt-4 text-center text-yellow-600 dark:text-yellow-400 font-medium">
              Opponent disconnected - waiting for reconnection...
            </div>
          )}
        </div>
      </div>
    </ChersSettingsProvider>
  );
}
