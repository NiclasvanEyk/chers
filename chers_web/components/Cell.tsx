import { Color } from "@/lib/chers";
import { ReactNode } from "react";

interface CellProps {
  color: Color;
  pickable: boolean;
  moveable: null | boolean;
  onClick: () => void;
  touched: boolean;
  children: undefined | ReactNode;
}

export function MoveableIndicator() {
  return <div className="absolute w-1/3 h-1/3 rounded-full bg-black/40"></div>;
}

export function Cell(props: CellProps) {
  let bgColor = props.color === "White" ? "bg-[#EDD6B0]" : "bg-[#B88662]";

  let hoverColor = "";
  if (props.moveable === true || props.pickable) {
    hoverColor =
      props.color === "White" ? "hover:bg-[#e5c28b]" : "hover:bg-[#9e6b47]";
  }

  if (props.touched) {
    bgColor = "bg-amber-500";
    hoverColor = "";
  }

  return (
    <div
      onClick={props.onClick}
      className={`${bgColor} ${hoverColor} relative h-[min(calc(100vh/8),calc(100vw/8))] w-[min(calc(100vh/8),calc(100vw/8))] md:h-16 md:w-16 overflow-hidden flex items-center justify-center select-none font-bold text-xl`}
    >
      {props.moveable === true ? <MoveableIndicator /> : null}
      {props.children}
    </div>
  );
}
