"use client";

import type { PlayerConnectionStatus } from "@/generated/chers_server_api/PlayerConnectionStatus";
import type { PlayerInfo } from "@/generated/chers_server_api/PlayerInfo";
import type { Color } from "@/generated/chers/chers";

interface ConnectionStatusProps {
  myColor: Color;
  opponent: PlayerInfo;
  isMyTurn: boolean;
}

export function ConnectionStatus({ myColor, opponent, isMyTurn }: ConnectionStatusProps) {
  const getStatusColor = () => {
    if (opponent.connected) return "bg-green-500";
    return "bg-yellow-500";
  };

  const getStatusText = () => {
    if (opponent.connected) {
      return isMyTurn ? "Your turn" : "Opponent's turn";
    }
    return "Waiting for opponent...";
  };

  const opponentColor = myColor === "White" ? "Black" : "White";

  return (
    <div className="flex items-center justify-between p-3 bg-gray-100 dark:bg-gray-800 rounded-lg mb-4">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${getStatusColor()}`}></div>
          <span className="font-medium">{opponent.name}</span>
          <span className="text-sm text-gray-600 dark:text-gray-400">
            ({opponentColor})
          </span>
        </div>
        {!opponent.connected && (
          <span className="text-xs text-yellow-600 dark:text-yellow-400">
            (Disconnected)
          </span>
        )}
      </div>
      
      <div className="text-sm font-medium">
        {getStatusText()}
      </div>
    </div>
  );
}
