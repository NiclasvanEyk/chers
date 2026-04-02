"use client";

import { useState, useCallback, useEffect } from "react";

interface LobbyProps {
  inviteUrl: string;
  myName: string;
  onUpdateName: (name: string) => void;
  isReady: boolean;
  opponentName?: string;
  opponentReady: boolean;
  onToggleReady: (ready: boolean) => void;
  countdown: number | null;
}

// State machine for the current player's UI
// 1: Editing name (input + Save)
// 2: Ready to play (display name + Ready button)
// 3: Confirm abort (display name + Abort button)
type PlayerUIState = 1 | 2 | 3;

export function Lobby({
  inviteUrl,
  myName,
  onUpdateName,
  isReady,
  opponentName,
  opponentReady,
  onToggleReady,
  countdown,
}: LobbyProps) {
  const [copied, setCopied] = useState(false);
  const [editName, setEditName] = useState(myName);
  const [displayName, setDisplayName] = useState(myName); // Local display name for immediate updates
  const [nameError, setNameError] = useState<string | null>(null);
  const [uiState, setUiState] = useState<PlayerUIState>(1); // Start in editing mode

  // Update display name when prop changes (from server)
  useEffect(() => {
    setDisplayName(myName);
    setEditName(myName);
  }, [myName]);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(inviteUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  const handleNameChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const newName = e.target.value;
    setEditName(newName);

    if (newName.length === 0) {
      setNameError("Name cannot be empty");
    } else if (newName.length > 25) {
      setNameError("Name must be 25 characters or less");
    } else {
      setNameError(null);
    }
  }, []);

  const handleSaveName = useCallback(() => {
    if (editName.length === 0) {
      setNameError("Name cannot be empty");
      return;
    }
    if (editName.length > 25) {
      setNameError("Name must be 25 characters or less");
      return;
    }

    setDisplayName(editName); // Update immediately for visual feedback
    onUpdateName(editName);
    setUiState(2); // Move to ready state after saving
  }, [editName, onUpdateName]);

  const handleReadyClick = useCallback(() => {
    if (isReady) {
      // Already ready - unready and go back to state 1
      onToggleReady(false);
      setUiState(1);
    } else {
      // Not ready yet - set ready and move to state 3 (abort mode)
      onToggleReady(true);
      setUiState(3);
    }
  }, [isReady, onToggleReady]);

  const handleAbortClick = useCallback(() => {
    onToggleReady(false);
    setUiState(1); // Go back to editing
  }, [onToggleReady]);

  // Render current player section based on UI state
  const renderCurrentPlayer = () => {
    switch (uiState) {
      case 1: // Editing mode
        return (
          <div className="flex items-center justify-between gap-3">
            <div className="flex-1">
              <input
                type="text"
                value={editName}
                onChange={handleNameChange}
                placeholder="Your name"
                className="w-full px-3 py-2 border border-stone-300 dark:border-stone-600 rounded bg-stone-50 dark:bg-stone-800 text-sm"
                maxLength={25}
              />
              {nameError && <p className="text-red-500 text-xs mt-1">{nameError}</p>}
            </div>
            <button
              onClick={handleSaveName}
              disabled={!!nameError || editName.length === 0}
              className="px-4 py-2 bg-amber-600 hover:bg-amber-700 disabled:bg-stone-400 disabled:cursor-not-allowed text-white rounded transition-colors text-sm whitespace-nowrap"
            >
              Save
            </button>
          </div>
        );

      case 2: // Ready mode (not ready yet)
        return (
          <div className="flex items-center justify-between gap-2">
            <span className="font-medium">{displayName}</span>
            <div className="flex gap-2">
              <button
                onClick={() => setUiState(1)}
                disabled={countdown !== null}
                className="px-3 py-2 rounded transition-colors text-sm font-medium bg-stone-500 hover:bg-stone-600 text-white"
              >
                Change
              </button>
              <button
                onClick={handleReadyClick}
                disabled={countdown !== null}
                className={`px-4 py-2 rounded transition-colors text-sm font-medium bg-amber-600 hover:bg-amber-700 text-white ${
                  countdown !== null ? "opacity-50 cursor-not-allowed" : ""
                }`}
              >
                Ready?
              </button>
            </div>
          </div>
        );

      case 3: // Abort mode (already ready)
        return (
          <div className="flex items-center justify-between">
            <span className="font-medium">{displayName}</span>
            <button
              onClick={handleAbortClick}
              disabled={countdown !== null}
              className={`px-4 py-2 rounded transition-colors text-sm font-medium bg-red-600 hover:bg-red-700 text-white ${
                countdown !== null ? "opacity-50 cursor-not-allowed" : ""
              }`}
            >
              Abort
            </button>
          </div>
        );
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4">
      <div className="max-w-md w-full space-y-6 text-center">
        {/* Share Link */}
        <div className="space-y-3">
          <p className="text-stone-600 dark:text-stone-400">
            Share this link with a friend to start playing:
          </p>

          <div className="flex gap-2">
            <input
              type="text"
              value={inviteUrl}
              readOnly
              className="flex-1 px-3 py-2 border border-stone-300 dark:border-stone-600 rounded bg-stone-50 dark:bg-stone-800 text-sm"
            />
            <button
              onClick={handleCopy}
              className="px-4 py-2 bg-stone-600 hover:bg-stone-700 text-white rounded transition-colors"
            >
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
        </div>

        {/* Players & Ready Status */}
        <div className="bg-stone-100 dark:bg-stone-800 rounded-lg p-4 space-y-4">
          {/* Current Player */}
          <div className="border-b border-stone-200 dark:border-stone-700 pb-3">
            {renderCurrentPlayer()}
          </div>

          {/* Opponent */}
          {opponentName ? (
            <div className="flex items-center justify-between">
              <span className="font-medium">{opponentName}</span>
              <span
                className={`text-sm ${opponentReady ? "text-green-600 font-medium" : "text-stone-400"}`}
              >
                {opponentReady ? "Ready ✓" : "Not Ready"}
              </span>
            </div>
          ) : (
            <p className="text-sm text-stone-500 text-center">Waiting for opponent to join...</p>
          )}
        </div>

        {/* Countdown */}
        {countdown !== null && (
          <div className="text-center">
            <p className="text-4xl font-bold text-amber-600 animate-pulse">{countdown}</p>
            <p className="text-sm text-stone-500">Game starting...</p>
          </div>
        )}
      </div>
    </div>
  );
}
