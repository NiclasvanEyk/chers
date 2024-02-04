import type { State as PartialState } from "@/generated/chers-serde/State";
import type { Piece } from "@/generated/chers-serde/Piece";
import type { Coordinate } from "@/generated/chers-serde/Coordinate";
import type { Move } from "@/generated/chers-serde/Move";
import type { MoveError } from "@/generated/chers-serde/MoveError";
import type { MoveResult } from "@/generated/chers-serde/MoveResult";
import type { PromotedFigure } from "@/generated/chers-serde/PromotedFigure";

import {
  Coordinate as CoordinateDTO,
  Move as MoveDTO,
  available_moves,
  next_state,
} from "@/generated/chers/chers";

// This file contains abstractions and hand-written types for the WASM
// functions, since the generated ones are not that nice to use

export type { Figure } from "@/generated/chers-serde/Figure";
export type { PromotedFigure } from "@/generated/chers-serde/PromotedFigure";
export type { Color } from "@/generated/chers-serde/Color";
export type { Piece } from "@/generated/chers-serde/Piece";
export type { Coordinate } from "@/generated/chers-serde/Coordinate";
export type { Move } from "@/generated/chers-serde/Move";

export type Cell = undefined | Piece;
export type Row = [Cell, Cell, Cell, Cell, Cell, Cell, Cell, Cell];
export type Board = [Row, Row, Row, Row, Row, Row, Row, Row];
export interface State extends PartialState {
  board: Board;
}

export interface MoveExecutionError {
  error: MoveError;
}

export type MoveExecutionResult = MoveResult;

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

  return dto;
}

export function getMoves(state: State, from: Coordinate) {
  console.time('getMoves');
  const moves = available_moves(state, coordToDto(from)) as unknown as Coordinate[];
  console.timeEnd('getMoves');

  return moves;
}

export function nextState(
  current: State,
  from: Coordinate,
  to: Coordinate,
  promotion: PromotedFigure | null = null,
): MoveExecutionResult | MoveExecutionError {
  const next = next_state(current, moveToDto({ from, to, promotion })) as unknown as
    | MoveExecutionResult
    | MoveExecutionError;

  if ("events" in next) {
    console.log(next.events);
  }

  return next;
}
