import { Cell } from "./Cell";
import { Piece } from "./Piece";
import { ChessContext } from "@/lib/useChessCommands";
import { useContext } from "react";
import { State } from "@/lib/chers";

interface BoardProps {
  state: State;
}

export function Board(props: BoardProps) {
  const { state } = props;
  const { stopShowingMoves, executeMove, touchedPiece, showMoves, player } =
    useContext(ChessContext);

  return (
    <div className="p-24 grid grid-cols-8 grid-rows-8">
      {state.board.flatMap((row, y) =>
        row.map((piece, x) => {
          let moveable: null | boolean = null;
          if (touchedPiece) {
            moveable =
              touchedPiece.moves.find(
                (move) => move.x === x && move.y === y,
              ) !== undefined;
          }

          let pickable = false;
          if (piece) {
            pickable = piece.color === player;
          }

          const onCellClick = () => {
            if (!touchedPiece && pickable) {
              showMoves({ x, y });
              return;
            }

            if (moveable && touchedPiece) {
              executeMove({
                from: touchedPiece.coordinate,
                to: { x, y },
                promotion: undefined,
              });
              return;
            }

            if (!piece) {
              stopShowingMoves();
            }
          };

          return (
            <Cell
              key={`${x},${y}`}
              color={x % 2 == y % 2 ? "White" : "Black"}
              moveable={moveable}
              pickable={pickable}
              onClick={onCellClick}
              touched={
                touchedPiece?.coordinate?.x === x &&
                touchedPiece?.coordinate?.y === y
              }
            >
              <Piece piece={piece} />
            </Cell>
          );
        }),
      )}
    </div>
  );
}
