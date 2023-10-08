import { createContext } from "react";
import { Coordinate, Move } from "./chers";

export const ChessCommandsContext = createContext<{
  stopShowingMoves(): void;
  showMoves(coordinate: Coordinate): void;
  executeMove(move: Move): void;
}>({
  stopShowingMoves() {},
  showMoves() {},
  executeMove() {},
});
