import { State } from "../state";

export const INITIAL_STATE: State = {
  type: "SELECTING_FROM",
  game: {
    player: "White",
    board: [
      [
        {
          color: "Black",
          figure: "Rook",
        },
        {
          color: "Black",
          figure: "Knight",
        },
        {
          color: "Black",
          figure: "Bishop",
        },
        {
          color: "Black",
          figure: "Queen",
        },
        {
          color: "Black",
          figure: "King",
        },
        {
          color: "Black",
          figure: "Bishop",
        },
        {
          color: "Black",
          figure: "Knight",
        },
        {
          color: "Black",
          figure: "Rook",
        },
      ],
      [
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
        {
          color: "Black",
          figure: "Pawn",
        },
      ],
      [
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
      ],
      [
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
      ],
      [
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
      ],
      [
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
        undefined,
      ],
      [
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
        {
          color: "White",
          figure: "Pawn",
        },
      ],
      [
        {
          color: "White",
          figure: "Rook",
        },
        {
          color: "White",
          figure: "Knight",
        },
        {
          color: "White",
          figure: "Bishop",
        },
        {
          color: "White",
          figure: "Queen",
        },
        {
          color: "White",
          figure: "King",
        },
        {
          color: "White",
          figure: "Bishop",
        },
        {
          color: "White",
          figure: "Knight",
        },
        {
          color: "White",
          figure: "Rook",
        },
      ],
    ],
    halfmove_clock: 0,
    fullmove_number: 1,
    en_passant_target: undefined,
  },
};

export function buildState(adjust: (state: State) => State): State {
  // @ts-ignore-next-line
  return adjust(structuredClone(INITIAL_STATE));
}
