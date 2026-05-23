"use client";

import { useNavigate } from "@tanstack/react-router";
import type { Color } from "@/generated/chers/chers";
import type { GameEndReason } from "@/lib/multiplayer/protocol";

interface GameOverProps {
  youWon: boolean;
  reason: GameEndReason;
  myColor: Color;
}

export function GameOver({ youWon, reason, myColor }: GameOverProps) {
  const navigate = useNavigate();

  const getWinnerText = () => {
    return youWon ? "You Won!" : "You Lost";
  };

  const getReasonText = () => {
    switch (reason) {
      case "Checkmate":
        return "by checkmate";
      case "Stalemate":
        return "by stalemate";
      case "Resignation":
        return "by resignation";
      case "DrawAgreement":
        return "by agreement";
      case "FiftyMoveRule":
        return "by 50-move rule";
      case "InsufficientMaterial":
        return "by insufficient material";
      case "ThreefoldRepetition":
        return "by threefold repetition";
      case "Timeout":
        return "by timeout";
      case "Abandoned":
        return "by abandonment";
      default:
        return "";
    }
  };

  const handleNewGame = () => {
    navigate({ to: "/multiplayer" });
  };

  const handleHome = async () => {
    await navigate({ to: "/" });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-white dark:bg-gray-900 rounded-lg p-8 max-w-md w-full mx-4 text-center shadow-2xl">
        <h2 className="text-4xl font-bold mb-2">{getWinnerText()}</h2>

        {reason !== "Checkmate" && (
          <p className="text-lg text-gray-600 dark:text-gray-400 mb-6">{getReasonText()}</p>
        )}

        <div className="space-y-3">
          <button
            onClick={handleNewGame}
            className="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded transition-colors"
          >
            Start New Game
          </button>

          <button
            onClick={handleHome}
            className="w-full py-3 px-4 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 font-semibold rounded transition-colors"
          >
            Back to Home
          </button>
        </div>
      </div>
    </div>
  );
}
