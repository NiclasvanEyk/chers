"use client";

import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { startNewMatch } from "@/lib/multiplayer";
import { useRouter } from "next/navigation";
import { useEffect } from "react";

export default function MultiplayerPage() {
  const { replace } = useRouter();
  useEffect(() => {
    startNewMatch().then((id) => {
      replace(`/multiplayer/${id}`);
    });
  }, [replace]);

  return (
    <ChessFigureLoadingIndicator fullscreen message="Request a new game" />
  );
}
