import { Cell as CellContents, Color } from "@/lib/chers";
import { ReactNode, Ref, forwardRef } from "react";
import { Piece } from "./Piece";
import { cellLabel } from "@/lib/ui/accessibility";

interface CellProps {
  x: number;
  y: number;
  color: Color;
  pickable: boolean;
  moveable: boolean;
  touched: boolean;
  onClick: () => void;
  contents: CellContents;
}

export function MoveableIndicator() {
  return <div className="absolute w-1/3 h-1/3 rounded-full bg-black/40"></div>;
}

export const Cell = forwardRef(function Cell(
  { x, y, color, pickable, moveable, touched, onClick, contents }: CellProps,
  ref: Ref<HTMLButtonElement>,
) {
  let bgColor = color === "White" ? "bg-chess-beige" : "bg-chess-brown";

  let hoverColor = "";
  if (moveable || pickable) {
    hoverColor =
      color === "White" ? "hover:bg-[#e5c28b]" : "hover:bg-[#9e6b47]";
  }

  let cursor = "";
  if (!pickable && !moveable) {
    cursor = "cursor-initial";
  }

  if (touched) {
    bgColor = "bg-amber-500";
    hoverColor = "";
  }

  return (
    <button
      data-x={x}
      data-y={y}
      ref={ref}
      aria-label={cellLabel({ x, y }, contents)}
      onClick={onClick}
      className={`${bgColor} ${hoverColor} ${cursor} relative h-[min(calc(100vh/8),calc(100vw/8))] w-[min(calc(100vh/8),calc(100vw/8))] md:h-16 md:w-16 overflow-hidden flex items-center justify-center select-none font-bold text-xl`}
    >
      {moveable ? <MoveableIndicator /> : null}
      <Piece piece={contents} />
    </button>
  );
});
