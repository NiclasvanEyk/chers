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

  useEffect(() => {
    init().then(() => setWasmReady(true));
  }, []);

  if (!wasmReady) {
    return <ChessFigureLoadingIndicator fullscreen message="Loading game engine..." />;
  }

  if (state.phase.kind === "loading") {
    return <ChessFigureLoadingIndicator fullscreen message="Loading match..." />;
  }

  if (state.phase.kind === "connecting") {
    return <ChessFigureLoadingIndicator fullscreen message="Connecting to server..." />;
  }

  if (state.phase.kind === "reconnecting") {
    return (
      <>
        <ReconnectingOverlay
          attempt={state.phase.attempt}
          secondsRemaining={state.phase.secondsRemaining}
        />
        {state.phase.secondsRemaining < 110 && (
          <div className="blur-sm">
            <ChessFigureLoadingIndicator fullscreen message="Attempting to reconnect..." />
          </div>
        )}
      </>
    );
  }

  if (state.phase.kind === "error") {
    const isMatchNotFound = state.phase.message === "authentication failed";
    return (
      <div className="flex flex-col items-center justify-center min-h-screen p-4">
        <div className="text-center max-w-md">
          <h1 className="text-3xl font-bold mb-4 text-red-600">
            {isMatchNotFound ? "Match Not Found" : "Error"}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mb-6">{state.phase.message}</p>
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

  if (state.phase.kind === "waiting") {
    return (
      <Lobby
        inviteUrl={state.phase.inviteUrl}
        myName={myName}
        onUpdateName={sendUpdateName}
        isReady={state.phase.isReady}
        opponent={state.phase.opponent}
        opponentIsReady={state.phase.opponentIsReady}
        onToggleReady={sendReady}
      />
    );
  }

  if (state.phase.kind === "post_game") {
    if (!state.myColor) {
      return <ChessFigureLoadingIndicator fullscreen message="Loading game state..." />;
    }

    return (
      <div className="flex flex-col items-center justify-center min-h-screen">
        <GameOver
          youWon={state.phase.youWon}
          reason={state.phase.reason}
          myColor={state.myColor}
        />
      </div>
    );
  }

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

  return (
    <div className="flex flex-col items-center justify-center min-h-screen">
      <p className="text-red-600">Unknown state: {state.phase.kind}</p>
    </div>
  );
}
