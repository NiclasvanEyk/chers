"use client";

import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { play } from "@/lib/multiplayer";
import { useEffect, useState } from "react";

interface Props {
  params: {
    id: number;
  };
}

type MatchState =
  | { kind: "initial" }
  | { kind: "connecting" }
  | { kind: "error"; message: string }
  | { kind: "connected" };

function loadingMessage(state: MatchState) {
  switch (state.kind) {
    case "initial":
      return "setting everything up...";
    case "error":
      return `Something went wrong: ${state.message}`;
    case "connecting":
      return "Connecting to the server...";
    case "connected":
      return "Connected! Waiting for other players to join...";
  }
}

export default function MatchPage({ params: { id } }: Props) {
  const [state, setState] = useState<MatchState>({ kind: "initial" });

  useEffect(() => {
    setState({ kind: "connecting" });
    const socket = play(id);

    console.log("Opening socket...");
    socket.onmessage = (message) => {
      console.log(message);
    };

    socket.onerror = (error) => {
      console.error(error);
      setState({ kind: "error", message: String(error) });
    };

    socket.onopen = () => {
      console.log(`Successfully connected to match ${id}!`);
      setState({ kind: "connected" });
    };

    return () => {
      console.log("Closing socket...");
      socket.close();
    };
  }, [id]);

  return (
    <ChessFigureLoadingIndicator fullscreen message={loadingMessage(state)} />
  );
}
