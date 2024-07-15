"use client";

import { ChessFigureLoadingIndicator } from "@/components/ChessFigureLoadingIndicator";
import { play } from "@/lib/multiplayer";
import { useEffect, useState } from "react";
import { PendingGameMessages } from "@/generated/chers-server/PendingGameMessages";

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

function useMatch(
	id: number,
	onMessage: (command: PendingGameMessages) => unknown,
) {
	const [state, setState] = useState<MatchState>({ kind: "initial" });

	useEffect(() => {
		setState({ kind: "connecting" });
		const socket = play(id);

		console.log("Opening socket...");
		socket.onmessage = (message) => {
			console.log(message);
			const { data } = message;
			const parsed = JSON.parse(data);
			onMessage(parsed);
			console.log(parsed);
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

	return { state };
}

export default function MatchPage({ params: { id } }: Props) {
	const [exists, setExists] = useState(true);
	const [messages, setMessages] = useState<string[]>([]);

	const { state } = useMatch(id, (message) => {
		switch (message.kind) {
			case "GameDoesNotExist":
				setExists(false);
				return true;

			case "PlayerJoined":
				setMessages((prev) => [...prev, "A player joined!"]);
				return true;

			case "PlayerIdentified":
				setMessages((prev) => [...prev, "The player got a name!"]);
				return true;

			case "GameStarted":
				setMessages((prev) => [...prev, "The game starts!"]);
				return true;
		}

		return false;
	});

	if (!exists) {
		<div>This game does not exist.</div>;
	}

	return (
		<ChessFigureLoadingIndicator fullscreen message={loadingMessage(state)} />
	);
}
