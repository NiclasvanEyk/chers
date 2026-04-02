"use client";

import { useEffect, useState } from "react";
import { useMatch } from "@/lib/multiplayer/useMatch";
import { Lobby } from "./Lobby";
import { MultiplayerGame } from "./MultiplayerGame";
import { GameOver } from "./GameOver";
import { ReconnectingOverlay } from "./ReconnectingOverlay";
import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { Link } from "@tanstack/react-router";
import init from "@/generated/chers/chers";

interface MatchClientProps {
  id: string;
}

export function MatchClient({ id }: MatchClientProps) {
  const [wasmReady, setWasmReady] = useState(false);
  const { state, myName, sendMove, sendUpdateName, sendReady } = useMatch(id);

  // Initialize WASM
  useEffect(() => {
    init().then(() => setWasmReady(true));
  }, []);

  // Wait for WASM to be ready
  if (!wasmReady) {
    return <ChessFigureLoadingIndicator fullscreen message="Loading game engine..." />;
  }

  // Loading state
  if (state.phase.kind === "loading") {
    return <ChessFigureLoadingIndicator fullscreen message="Loading match..." />;
  }

  // Connecting state
  if (state.phase.kind === "connecting" || state.phase.kind === "authenticating") {
    return <ChessFigureLoadingIndicator fullscreen message="Connecting to server..." />;
  }

  // Game starting - waiting for color assignment
  if (state.phase.kind === "game_starting") {
    return <ChessFigureLoadingIndicator fullscreen message="Game starting... assigning colors" />;
  }

  // Reconnecting state
  if (state.phase.kind === "reconnecting") {
    return (
      <>
        <ReconnectingOverlay
          attempt={state.phase.attempt}
          secondsRemaining={state.phase.secondsRemaining}
        />
        {/* Show game behind overlay if we have game state */}
        {state.phase.secondsRemaining < 110 && (
          <div className="blur-sm">
            <ChessFigureLoadingIndicator fullscreen message="Attempting to reconnect..." />
          </div>
        )}
      </>
    );
  }

  // Match not found error
  if (state.phase.kind === "match_not_found") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen p-4">
        <div className="text-center max-w-md">
          <h1 className="text-3xl font-bold mb-4 text-red-600">Match Not Found</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            This match doesn&apos;t exist or has already ended.
          </p>
          <Link
            to="/multiplayer"
            className="inline-block px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded transition-colors"
          >
            Create New Game
          </Link>
        </div>
      </div>
    );
  }

  // Authentication error
  if (state.phase.kind === "auth_error") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen p-4">
        <div className="text-center max-w-md">
          <h1 className="text-3xl font-bold mb-4 text-red-600">Connection Failed</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            {state.phase.reason === "MatchFull"
              ? "This match already has two players."
              : state.phase.reason === "MatchNotFound"
                ? "This match doesn't exist."
                : "Authentication failed."}
          </p>
          <Link
            to="/multiplayer"
            className="inline-block px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded transition-colors"
          >
            Create New Game
          </Link>
        </div>
      </div>
    );
  }

  // Error state
  if (state.phase.kind === "error") {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen p-4">
        <div className="text-center max-w-md">
          <h1 className="text-3xl font-bold mb-4 text-red-600">Error</h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">{state.phase.message}</p>
          <Link
            to="/"
            className="inline-block px-6 py-3 bg-gray-600 hover:bg-gray-700 text-white font-semibold rounded transition-colors"
          >
            Back to Home
          </Link>
        </div>
      </div>
    );
  }

  // Waiting for opponent
  if (state.phase.kind === "waiting") {
    return (
      <Lobby
        inviteUrl={state.phase.inviteUrl}
        myName={myName}
        onUpdateName={sendUpdateName}
        isReady={state.phase.isReady}
        opponentName={state.phase.opponentName || undefined}
        opponentReady={state.phase.opponentReady}
        onToggleReady={sendReady}
        countdown={state.phase.countdown}
      />
    );
  }

  // Game over
  if (state.phase.kind === "game_over") {
    if (!state.myColor) {
      return (
        <div className="flex flex-col items-center justify-center min-h-screen">
          <GameOver result={state.phase.result} reason={state.phase.reason} myColor="White" />
        </div>
      );
    }

    return (
      <div className="flex flex-col items-center justify-center min-h-screen">
        <GameOver result={state.phase.result} reason={state.phase.reason} myColor={state.myColor} />
      </div>
    );
  }

  // Playing
  if (state.phase.kind === "playing") {
    if (!state.myColor) {
      return <ChessFigureLoadingIndicator fullscreen message="Loading game state..." />;
    }

    return (
      <MultiplayerGame
        game={state.phase.game}
        myColor={state.myColor}
        myTurn={state.phase.myTurn}
        myName={myName}
        opponent={state.phase.opponent}
        onMove={sendMove}
      />
    );
  }

  // Should never reach here
  return (
    <div className="flex flex-col items-center justify-center min-h-screen">
      <p className="text-red-600">Unknown state: {state.phase.kind}</p>
    </div>
  );
}
