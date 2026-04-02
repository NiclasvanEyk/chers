import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { startNewMatch } from "@/lib/multiplayer";
import { useEffect } from "react";

function MultiplayerPage() {
  const navigate = useNavigate();
  useEffect(() => {
    startNewMatch().then((id) => {
      navigate({ to: `/multiplayer/${id}` });
    });
  }, [navigate]);

  return <ChessFigureLoadingIndicator fullscreen message="Request a new game" />;
}

export const Route = createFileRoute("/multiplayer")({
  component: MultiplayerPage,
});
