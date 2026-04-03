"use client";

import type { PlayerInfo } from "@/generated/chers_server_api/PlayerInfo";
import type { Color } from "@/generated/chers/chers";

interface PlayerBarProps {
  myColor: Color;
  myName: string;
  opponent: PlayerInfo;
  isMyTurn: boolean;
}

export function PlayerBar({ myColor, myName, opponent, isMyTurn }: PlayerBarProps) {
  const opponentColor = myColor === "White" ? "Black" : "White";

  // Determine whose turn it is
  const currentPlayerColor = isMyTurn ? myColor : opponentColor;
  const isWhiteTurn = currentPlayerColor === "White";

  return (
    <div className="relative flex items-center justify-between gap-4 rounded-xl mb-4">
      {/* Sliding Background Highlight */}
      <div
        className="absolute top-0 bottom-0 w-[calc(50%-0.5rem)] bg-white dark:bg-stone-700 shadow-md rounded-lg transition-transform duration-150 ease-in-out -z-10"
        style={{
          transform: isWhiteTurn ? "translateX(0)" : "translateX(calc(100% + 1rem))",
        }}
      />

      {/* White Player */}
      <PlayerSlot
        name={myColor === "White" ? myName : opponent.name}
        color="White"
        isConnected={myColor === "White" ? true : opponent.connected}
        rightAligned={false}
      />

      {/* Black Player */}
      <PlayerSlot
        name={myColor === "Black" ? myName : opponent.name}
        color="Black"
        isConnected={myColor === "Black" ? true : opponent.connected}
        rightAligned={true}
      />
    </div>
  );
}

interface PlayerSlotProps {
  name: string;
  color: Color;
  isConnected: boolean;
  rightAligned: boolean;
}

function PlayerSlot({ name, color, isConnected, rightAligned }: PlayerSlotProps) {
  const isWhite = color === "White";

  // Color indicator - neutral white for white player, dark for black
  const colorIndicator = isWhite ? "bg-white border-stone-300" : "bg-stone-800 border-stone-600";

  return (
    <div
      className={`flex-1 flex items-center gap-3 p-3 rounded-lg ${rightAligned ? "flex-row-reverse" : ""}`}
    >
      {/* Color Circle - just an empty circle, no dot inside */}
      <div
        className={`w-10 h-10 rounded-full border-2 ${colorIndicator} flex items-center justify-center flex-shrink-0`}
      >
        {/* Empty - no dot inside */}
      </div>

      {/* Player Info */}
      <div className={`flex flex-col min-w-0 flex-1 ${rightAligned ? "items-end text-right" : ""}`}>
        <div className={`flex items-center gap-2 ${rightAligned ? "flex-row-reverse" : ""}`}>
          <span className="font-semibold truncate text-stone-900 dark:text-white">{name}</span>
        </div>
        <span
          className={`text-xs ${isConnected ? (isWhite ? "text-stone-500" : "text-stone-400") : "text-yellow-600 font-semibold"}`}
        >
          {isConnected ? color : "Trying to reconnect..."}
        </span>
      </div>
    </div>
  );
}
