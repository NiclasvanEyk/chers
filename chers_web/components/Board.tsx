import { Cell as CellContents, Color, Coordinate, State } from "@/lib/chers";
import { ChessCommandsContext } from "@/lib/useChessCommands";
import { ReactNode, useContext } from "react";
import { TouchedPiece } from "./Chers";

interface PieceProps {
  piece: CellContents;
  coordinate: Coordinate;
}

export function Piece(props: PieceProps) {
  const { showMoves: showMoves } = useContext(ChessCommandsContext);

  const { piece } = props;
  if (!piece) return null;

  const { figure, color } = piece;
  const pieceColor = color === "White" ? "text-white" : "text-black";
  const label = figure === "Knight" ? "N" : figure[0];

  return (
    <button
      onMouseDown={() => showMoves(props.coordinate)}
      className={`${pieceColor} h-full w-full hover:bg-black/10`}
    >
      {label}
    </button>
  );
}

interface CellProps {
  color: Color;
  moveable: null | boolean;
  occupied: boolean;
  touched: boolean;
  children: undefined | ReactNode;
}

export function MoveableIndicator() {
  return <div className="absolute w-1/3 h-1/3 rounded-full bg-black/10"></div>;
}

export function Cell(props: CellProps) {
  let background = props.color === "White" ? "bg-[#EDD6B0]" : "bg-[#B88662]";
  if (props.touched) {
    background = "bg-amber-500";
  }
  const { stopShowingMoves } = useContext(ChessCommandsContext);

  return (
    <div
      onClick={() => !props.occupied && stopShowingMoves()}
      className={`${background} relative h-16 w-16 overflow-hidden flex items-center justify-center select-none font-bold text-xl`}
    >
      {props.moveable === true ? <MoveableIndicator /> : null}
      {props.children}
    </div>
  );
}

interface BoardProps {
  state: State;
  touchedPiece: null | TouchedPiece;
}

export function Board(props: BoardProps) {
  const { state, touchedPiece } = props;

  return (
    <div className="grid grid-cols-8 grid-rows-8">
      {state.board.flatMap((row, y) =>
        row.map((piece, x) => {
          let moveable: null | boolean = null;
          if (touchedPiece) {
            moveable =
              touchedPiece.moves.find(
                (move) => move.x === x && move.y === y,
              ) !== undefined;
          }

          return (
            <Cell
              key={`${x},${y}`}
              color={x % 2 == y % 2 ? "White" : "Black"}
              moveable={moveable}
              occupied={!!piece}
              touched={
                touchedPiece?.coordinate?.x === x &&
                touchedPiece?.coordinate?.y === y
              }
            >
              <Piece piece={piece} coordinate={{ x, y }} />
            </Cell>
          );
        }),
      )}
    </div>
  );
}
