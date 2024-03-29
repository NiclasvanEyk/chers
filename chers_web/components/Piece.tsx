import { Cell } from "@/lib/chers";

interface PieceProps {
	piece: Cell;
	className?: string;
}

export function Piece(props: PieceProps) {
	let { piece, className } = props;
	if (!piece) return null;

	className ??= "h-3/4 w-3/4 z-10";

	return (
		<img
			src={`/images/pieces/${piece.color}_${piece.figure}.svg`}
			alt={`${piece.color} ${piece.figure}`}
			draggable="false"
			className={className}
		/>
	);
}
