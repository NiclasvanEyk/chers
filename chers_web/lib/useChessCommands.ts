import { createContext } from "react";
import { Coordinate, Move, Color } from "./chers";
import { TouchedPiece } from "@/components/Chers";

export const ChessContext = createContext<{
  stopShowingMoves(): void;
  showMoves(coordinate: Coordinate): void;
  executeMove(move: Move): void;
  touchedPiece: TouchedPiece | null;
  player: Color;
}>({
  stopShowingMoves() {},
  showMoves() {},
  executeMove() {},
  touchedPiece: null,
  player: "White",
});
