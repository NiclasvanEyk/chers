import {
  Coordinate as CoordinateDTO,
  Move as MoveDTO,
  available_moves,
  next_state,
} from "./chers/chers";

// This file contains abstractions and hand-written types for the WASM
// functions, since the generated ones are not that nice to use

export type Figure = "King" | "Queen" | "Rook" | "Bishop" | "Knight" | "Pawn";
export type Color = "White" | "Black";
export type Cell = undefined | Piece;
export type Row = [Cell, Cell, Cell, Cell, Cell, Cell, Cell, Cell];
export type Board = [Row, Row, Row, Row, Row, Row, Row, Row];

export interface Piece {
  color: Color;
  figure: Figure;
}

export interface Coordinate {
  x: number;
  y: number;
}

export interface Move {
  from: Coordinate;
  to: Coordinate;
  promotion: undefined | Omit<Figure, "Pawn" | "King">;
}

export interface State {
  player: Color;
  board: Board;
  en_passant_target: undefined | Coordinate;
  fullmove_number: number;
  halfmove_clock: number;
}

export interface MoveExecutionError {
  error: string;
}

export interface MoveExecutionResult {
  next_state: State;
  check: boolean;
  mate: boolean;
}

function coordToDto(coordinate: Coordinate): CoordinateDTO {
  return new CoordinateDTO(coordinate.x, coordinate.y);
}

function promotionToSerialized(
  promotion: Move["promotion"],
): number | undefined {
  if (!promotion) return undefined;

  return (
    {
      Queen: 0,
      Rook: 1,
      Bishop: 2,
      Knight: 3,
      // @ts-ignore-next-line
    }[promotion] ?? 0
  );
}

function moveToDto(move: Move): MoveDTO {
  const dto = new MoveDTO(
    coordToDto(move.from),
    coordToDto(move.to),
    promotionToSerialized(move.promotion),
  );
  console.log("DTO[promotion]", dto.promotion);

  return dto;
}

export function getMoves(state: State, from: Coordinate) {
  return available_moves(state, coordToDto(from)) as unknown as Coordinate[];
}

export function nextState(
  current: State,
  from: Coordinate,
  to: Coordinate,
  promotion: Figure | undefined = undefined,
): MoveExecutionResult | MoveExecutionError {
  return next_state(current, moveToDto({ from, to, promotion })) as unknown as
    | MoveExecutionResult
    | MoveExecutionError;
}
