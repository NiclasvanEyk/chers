import { createFileRoute, Navigate } from "@tanstack/react-router";
import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { startNewMatch } from "@/lib/multiplayer";
import { useEffect, useState } from "react";

function MultiplayerPage() {
  const [matchId, setMatchId] = useState<string | null>(null);

  useEffect(() => {
    startNewMatch()
      .then((id) => setMatchId(id))
      .catch((err) => console.error("Failed to create match:", err));
  }, []);

  if (matchId) {
    return (
      <Navigate
        to="/multiplayer/$id"
        params={{ id: matchId }}
        replace={true}
      />
    );
  }

  return (
    <ChessFigureLoadingIndicator fullscreen message="Creating a new game..." />
  );
}

export const Route = createFileRoute("/multiplayer")({
  component: MultiplayerPage,
});
