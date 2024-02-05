import { Cell as CellContents, Color } from "@/lib/chers";
import { Ref, forwardRef } from "react";
import { Piece } from "./Piece";
import { cellLabel } from "@/lib/ui/accessibility";
import { useSettings } from "@/lib/settings";

interface CellProps {
	x: number;
	y: number;
	color: Color;
	pickable: boolean;
	moveable: boolean;
	captures: boolean;
	touched: boolean;
	onClick: () => void;
	contents: CellContents;
}

export function MoveableIndicator() {
	return (
		<div className="absolute h-1/4 w-1/4 rounded-full bg-black/30 animate-scale-in-bouncy" />
	);
}

export function CaptureableIndicator() {
	const shared =
		"absolute h-3 w-2/4 bg-black/30 transform animate-scale-in-bouncy";
	return (
		<>
			<div className={`${shared}`} />
		</>
	);
}

export const Cell = forwardRef(function Cell(
	{
		x,
		y,
		color,
		pickable,
		moveable,
		touched,
		captures,
		onClick,
		contents,
	}: CellProps,
	ref: Ref<HTMLButtonElement>,
) {
	const { highlightLegalMoves } = useSettings();
	let bgColor = color === "White" ? "bg-chess-beige" : "bg-chess-brown";

	let hoverColor = "";
	if (moveable || pickable) {
		hoverColor =
			color === "White" ? "hover:bg-[#e5c28b]" : "hover:bg-[#9e6b47]";
	}

	let cursor = "";
	if (!pickable && !moveable) {
		cursor = "cursor-default";
	}

	if (touched) {
		bgColor = "bg-amber-500";
		hoverColor = "";
	}

	let indicator = null;
	if (moveable && highlightLegalMoves) {
		indicator = captures ? <CaptureableIndicator /> : <MoveableIndicator />;
	}

	return (
		<button
			tabIndex="0"
			data-x={x}
			data-y={y}
			ref={ref}
			aria-label={cellLabel({ x, y }, captures, contents)}
			onClick={onClick}
			className={`focus:outline-none focus-visible:outline-none focus:ring-2 focus-visible:ring-2 focus:z-10 focus-visible:z-10 ring-blue-500 ${bgColor} ${hoverColor} ${cursor} relative h-[min(calc(100vh/8),calc(100vw/8))] w-[min(calc(100vh/8),calc(100vw/8))] md:h-16 md:w-16 flex items-center justify-center select-none font-bold text-xl`}
		>
			<Piece piece={contents} />
			{indicator}
			<RankLabels x={x} y={y} color={color} />
		</button>
	);
});

function RankLabels(props: { x: number; y: number; color: Color }) {
	const { displayLabels } = useSettings();
	if (!displayLabels) return null;

	const labelColor =
		props.color === "Black" ? "text-chess-beige" : "text-chess-brown";

	return (
		<>
			{props.x !== 0 ? null : (
				<span
					className={`${labelColor} absolute top-0 left-1 text-xs sm:text-sm`}
				>
					{8 - props.y}
				</span>
			)}
			{props.y !== 7 ? null : (
				<span
					className={`${labelColor} absolute bottom-0 right-1 text-xs sm:text-sm`}
				>
					{
						{
							0: "a",
							1: "b",
							2: "c",
							3: "d",
							4: "e",
							5: "f",
							6: "g",
							7: "h",
						}[props.x]
					}
				</span>
			)}
		</>
	);
}
