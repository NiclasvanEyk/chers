import { canPickUp, canMoveTo, hasPickedUp, useChers } from "@/lib/ui/state";
import { Cell } from "./Cell";
import { Promotion } from "./Promotion";
import { useFocusManagement } from "@/lib/ui/useFocusManagement";

export function Board() {
  const [state, dispatch] = useChers();
  const { board, player } = state.game;

  const { registerCellRef } = useFocusManagement(state, dispatch);

  if (state.type === "ERROR") {
    return (
      <span className="text-red-500 font-bold text-2xl">{state.error}</span>
    );
  }

  return (
    <div className="relative">
      {state.type === "PROMOTING" ? (
        <Promotion
          color={state.game.player}
          onChoice={(figure) => dispatch({ type: "PROMOTE", to: figure })}
        />
      ) : null}
      {state.type === "GAME_OVER" ? (
        <div className="absolute inset-0 z-10">
          <div className="flex flex-col h-full w-full bg-black/50 place-content-center text-center">
            <dialog
              open
              style={{
                color: state.winner,
                textShadow:
                  state.winner === "White"
                    ? ""
                    : "2px 0 #fff, -2px 0 #fff, 0 2px #fff, 0 -2px #fff, 1px 1px #fff, -1px -1px #fff, 1px -1px #fff, -1px 1px #fff",
              }}
              className="p-5 flex flex-col gap-5 bg-transparent drop-shadow"
            >
              <p className="text-3xl font-extrabold leading-tight">
                {state.winner} won
              </p>
              <button
                onClick={() => dispatch({ type: "BEGIN" })}
                className="text-white"
              >
                Start Over
              </button>
            </dialog>
          </div>
        </div>
      ) : null}
      <div
        // @ts-ignore-next-line
        inert={state.type === "PROMOTING" ? true : null}
        className="w-full h-full grid grid-cols-8 grid-rows-8"
      >
        {board.flatMap((row, y) =>
          row.map((contents, x) => {
            let pickable = canPickUp(contents, player);
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

              // Change the currently picked up piece to a different one
              if (state.type === "SELECTING_TO" && pickable) {
                dispatch({ type: "SELECT_FROM", from: { x, y } });
                return;
              }

              // If we click on an empty field and have picked up a piece, put it
              // down again
              if (state.type === "SELECTING_TO" && !contents) {
                dispatch({ type: "ABORT_SELECTION" });
              }
            };

            return (
              <Cell
                ref={(ref) => registerCellRef(x, y, ref)}
                x={x}
                y={y}
                key={`${x},${y}`}
                color={x % 2 == y % 2 ? "White" : "Black"}
                onClick={onCellClick}
                moveable={moveable}
                pickable={pickable}
                touched={touched}
                contents={contents}
              />
            );
          }),
        )}
      </div>
    </div>
  );
}
