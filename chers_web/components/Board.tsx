import { canPickUp, canMoveTo, hasPickedUp, useChers } from "@/lib/ui/state";
import { Cell } from "./Cell";
import { Piece } from "./Piece";

export function Board() {
  const [state, dispatch] = useChers();
  const { board, player } = state.game;

  if (state.type === "ERROR") {
    return (
      <span className="text-red-500 font-bold text-2xl">{state.error}</span>
    );
  }

  return (
    <div className="md:p-24 grid grid-cols-8 grid-rows-8">
      {board.flatMap((row, y) =>
        row.map((piece, x) => {
          let pickable = canPickUp(piece, player);
          let moveable = canMoveTo(state, { x, y });
          let touched = hasPickedUp(state, { x, y });

          const onCellClick = () => {
            // Select a piece to move if you could pick one up
            if (state.type === "SELECTING_FROM" && pickable) {
              dispatch({ type: "SELECT_FROM", from: { x, y } });
              return;
            }

            // Select a field to move the currently picked up piece to
            if (state.type === "SELECTING_TO" && moveable) {
              dispatch({ type: "SELECT_TO", to: { x, y } });
              return;
            }

            // If we click on an empty field and have picked up a piece, put it
            // down again
            if (state.type === "SELECTING_TO" && !piece) {
              dispatch({ type: "ABORT_SELECTION" });
            }
          };

          return (
            <Cell
              key={`${x},${y}`}
              color={x % 2 == y % 2 ? "White" : "Black"}
              onClick={onCellClick}
              moveable={moveable}
              pickable={pickable}
              touched={touched}
            >
              <Piece piece={piece} />
            </Cell>
          );
        }),
      )}
    </div>
  );
}
