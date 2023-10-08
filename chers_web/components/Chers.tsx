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
import { ChessCommandsContext } from "@/lib/useChessCommands";
import { Move, MoveExecutionResult, Coordinate } from "@/lib/chers";

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
  moves: Coordinate[];
}

export default function Chers() {
  const [game, setGame] = useState(null);
  const [touchedPiece, setTouchedPiece] = useState<TouchedPiece | null>(null);

  useEffect(function () {
    init().then(() => {
      setGame(new_game());
    });
  }, []);

  if (!game) {
    return "Loading...";
  }

  function stopShowingMoves() {
    setTouchedPiece(null);
  }

  function showMoves(coordinate: Coordinate) {
    const available = available_moves(game, coordToDto(coordinate));
    setTouchedPiece({
      coordinate,
      moves: available,
    });
  }

  function executeMove(move: Move) {
    stopShowingMoves();

    const next = (
      next_state(game, moveToDto(move)) as unknown as MoveExecutionResult
    ).next_state;
    setGame(next as any);
  }

  return (
    <ChessCommandsContext.Provider
      value={{ showMoves, executeMove, stopShowingMoves }}
    >
      <Board state={game} touchedPiece={touchedPiece} />
    </ChessCommandsContext.Provider>
  );
}
