import { Cell } from "@/lib/chers";

interface PieceProps {
  piece: Cell;
}

export function Piece(props: PieceProps) {
  const { piece } = props;
  if (!piece) return null;

  return (
    <img
      src={`/pieces/${piece.color}_${piece.figure}.svg`}
      className="h-3/4 w-3/4"
    />
  );
}
