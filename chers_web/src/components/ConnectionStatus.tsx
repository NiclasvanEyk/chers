"use client";

import type { Color } from "@/generated/chers/chers";
import type { User } from "@/lib/multiplayer/protocol";

interface ConnectionStatusProps {
  myColor: Color;
  opponent: User;
  isMyTurn: boolean;
}

export function ConnectionStatus({ myColor, opponent, isMyTurn }: ConnectionStatusProps) {
  const opponentColor = myColor === "White" ? "Black" : "White";

  return (
    <div className="flex items-center justify-between p-3 bg-gray-100 dark:bg-gray-800 rounded-lg mb-4">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full bg-green-500"></div>
          <span className="font-medium">{opponent.name}</span>
          <span className="text-sm text-gray-600 dark:text-gray-400">({opponentColor})</span>
        </div>
      </div>

      <div className="text-sm font-medium">
        {isMyTurn ? "Your turn" : "Opponent's turn"}
      </div>
    </div>
  );
}
