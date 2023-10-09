"use client";
import { useEffect, useState } from "react";
import init, {
  new_game,
  available_moves,
  next_state,
  Coordinate as CoordinateDTO,
  Move as MoveDTO,
} from "@/lib/chers/chers";
import { Board } from "./Board";
import { ChessContext } from "@/lib/useChessCommands";
import {
  Move,
  MoveExecutionResult,
  Coordinate,
  Piece,
  State,
} from "@/lib/chers";
import { LoadingIndicator } from "./LoadingIndicator";

function coordToDto(coordinate: Coordinate): CoordinateDTO {
  return new CoordinateDTO(coordinate.x, coordinate.y);
}

function moveToDto(move: Move): MoveDTO {
  const dto = new MoveDTO(
    coordToDto(move.from),
    coordToDto(move.to),
    undefined,
  );

  return dto;
}

export interface TouchedPiece {
  coordinate: Coordinate;
  piece: Piece;
  moves: Coordinate[];
}

export default function Chers() {
  const [game, setGame] = useState<State | null>(null);
  const [touchedPiece, setTouchedPiece] = useState<TouchedPiece | null>(null);

  useEffect(function () {
    init().then(() => {
      setGame(new_game());
    });
  }, []);

  if (!game) {
    return (
      <div className="h-screen w-screen flex justify-center items-center">
        <LoadingIndicator className="h-10 w-10 animate-spin" />
      </div>
    );
  }

  function stopShowingMoves() {
    setTouchedPiece(null);
  }

  const showMoves = (coordinate: Coordinate) => {
    const available = available_moves(game, coordToDto(coordinate));
    setTouchedPiece({
      coordinate,
      piece: game.board[coordinate.y][coordinate.x]!,
      moves: available,
    });
  };

  function executeMove(move: Move) {
    stopShowingMoves();

    const next = (
      next_state(game, moveToDto(move)) as unknown as MoveExecutionResult
    ).next_state;
    setGame(next as any);
  }

  return (
    <ChessContext.Provider
      value={{
        showMoves,
        executeMove,
        stopShowingMoves,
        touchedPiece,
        player: game.player,
      }}
    >
      <Board state={game} />
    </ChessContext.Provider>
  );
}
