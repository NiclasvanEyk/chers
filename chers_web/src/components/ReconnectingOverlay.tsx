"use client";

interface ReconnectingOverlayProps {
  attempt: number;
  secondsRemaining: number;
}

export function ReconnectingOverlay({ attempt, secondsRemaining }: ReconnectingOverlayProps) {
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70">
      <div className="bg-white dark:bg-gray-900 rounded-lg p-8 max-w-md w-full mx-4 text-center shadow-2xl">
        <div className="mb-4">
          <div className="inline-flex items-center justify-center w-16 h-16 bg-yellow-100 dark:bg-yellow-900/30 rounded-full mb-4">
            <svg
              className="w-8 h-8 text-yellow-600 dark:text-yellow-400 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </div>
        </div>

        <h2 className="text-2xl font-bold mb-2">Reconnecting...</h2>

        <p className="text-gray-600 dark:text-gray-400 mb-4">Attempt {attempt}</p>

        <div className="bg-gray-100 dark:bg-gray-800 rounded-lg p-4 mb-4">
          <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">Time until forfeit:</p>
          <p className="text-2xl font-mono font-bold text-red-600 dark:text-red-400">
            {formatTime(secondsRemaining)}
          </p>
        </div>

        <p className="text-xs text-gray-500">
          The game will be forfeited if you don&apos;t reconnect within 2 minutes.
        </p>
      </div>
    </div>
  );
}
