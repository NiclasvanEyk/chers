import type {
  State,
  Coordinate,
  Move,
  MoveExecutionResult,
  MoveExecutionError,
  Color,
  Piece,
  Figure,
  PromotedFigure,
} from "@/generated/chers/chers";

import { available_moves, next_state, new_game } from "@/generated/chers/chers";

// Re-export types from the generated bindings
export type {
  State,
  Coordinate,
  Move,
  MoveExecutionResult,
  MoveExecutionError,
  Color,
  Piece,
  Figure,
  PromotedFigure,
};

// Extended types for the web UI
export type Cell = Piece | null;
export type Board = Cell[][];

export function getMoves(state: State, from: Coordinate): Coordinate[] {
  console.time("getMoves");
  const moves = available_moves(state, from) as Coordinate[];
  console.timeEnd("getMoves");
  return moves;
}

export function nextState(
  current: State,
  from: Coordinate,
  to: Coordinate,
  promotion: PromotedFigure | null = null,
): MoveExecutionResult | MoveExecutionError {
  const next = next_state(current, { from, to, promotion }) as
    | MoveExecutionResult
    | MoveExecutionError;

  if ("events" in next) {
    console.log(next.events);
  }

  return next;
}

export { new_game as newGame };
