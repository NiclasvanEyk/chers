import { Dispatch, useReducer } from "react";
import {
  Coordinate,
  State as GameState,
  Color,
  Piece,
  Figure,
  nextState,
  getMoves,
  Cell,
} from "../chers";
import { new_game } from "../chers/chers";

export type State = (
  | { type: "ERROR"; error: string }
  | { type: "SELECTING_FROM" }
  | {
      type: "SELECTING_TO";
      from: Coordinate;
      piece: Piece;
      moves: Coordinate[];
    }
  | { type: "PROMOTING"; from: Coordinate; to: Coordinate }
  | { type: "GAME_OVER"; winner: Color }
) & {
  game: GameState;
};

export type Command =
  | { type: "BEGIN"; game: GameState }
  | { type: "ABORT_SELECTION" }
  | { type: "SELECT_FROM"; from: Coordinate }
  | { type: "SELECT_TO"; to: Coordinate }
  | { type: "PROMOTE"; to: Figure };

export interface Adapter {
  nextState: typeof nextState;
  getMoves: typeof getMoves;
}

export const wasmAdapter = { nextState, getMoves };

/**
 * @private
 */
export function buildChersReducer(adapter: Adapter) {
  return (state: State, command: Command): State => {
    const { game } = state;

    switch (command.type) {
      case "BEGIN": {
        return { type: "SELECTING_FROM", game: command.game };
      }

      case "SELECT_FROM": {
        const { from } = command;
        const moves = adapter.getMoves(state.game, from);
        const piece = state.game.board[from.y][from.x];
        if (piece === undefined) {
          return { type: "ERROR", error: "No piece to move", game };
        }

        return { type: "SELECTING_TO", game, from, moves, piece };
      }

      case "ABORT_SELECTION": {
        if (!("game" in state)) {
          return state;
        }

        return { type: "SELECTING_FROM", game };
      }

      case "SELECT_TO": {
        if (state.type !== "SELECTING_TO") {
          return state;
        }

        const result = adapter.nextState(game, state.from, command.to);
        if ("error" in result) {
          if (result.error) {
            return {
              type: "PROMOTING",
              from: state.from,
              to: command.to,
              game: state.game,
            };
          }

          return { type: "ERROR", error: result.error, game };
        }

        // TODO: Check for check
        return { type: "SELECTING_FROM", game: result.next_state };
      }

      case "PROMOTE": {
        if (state.type !== "PROMOTING") {
          return state;
        }

        const result = adapter.nextState(
          game,
          state.from,
          state.to,
          command.to,
        );
        if ("error" in result) {
          return { type: "ERROR", error: result.error, game };
        }

        // TODO: Check for check
        return { type: "SELECTING_FROM", game: result.next_state };
      }
    }

    ((c: never) => {
      throw new Error(`Unknown command '${c}'`);
    })(command);
  };
}

export type Dispatcher = Dispatch<Command>;

export function canPickUp(cellContents: Cell, player: Color): boolean {
  if (cellContents === undefined) {
    return false;
  }

  return cellContents.color === player;
}

export function hasPickedUp(state: State, cell: Coordinate) {
  if (state.type !== "SELECTING_TO") {
    return false;
  }

  return state.from.x === cell.x && state.from.y === cell.y;
}

export function canMoveTo(state: State, to: Coordinate): boolean {
  if (state.type !== "SELECTING_TO") {
    return false;
  }

  return (
    state.moves.find((move) => move.x === to.x && move.y === to.y) !== undefined
  );
}

function initialChersState(): State {
  return {
    type: "SELECTING_FROM",
    game: new_game(),
  };
}

export function useChers() {
  return useReducer(
    buildChersReducer(wasmAdapter),
    {} as unknown as any,
    initialChersState,
  );
}
