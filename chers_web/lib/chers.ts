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
  promotion: undefined | Omit<Figure, "Pawn">;
}

export interface State {
  player: Color;
  board: Board;
  en_passant_target: undefined | Coordinate;
  fullmove_number: number;
  halvmove_clock: number;
}

export interface MoveExecutionResult {
  next_state: State;
  check: boolean;
  mate: boolean;
}
