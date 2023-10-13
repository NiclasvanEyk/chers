import { Color, Figure } from "@/lib/chers";
import { Piece } from "./Piece";

export interface PromotionProps {
  onChoice: (figure: Figure) => void;
  color: Color;
}

export function Promotion({ color, onChoice }: PromotionProps) {
  const Option = ({ figure }: { figure: Figure }) => {
    return (
      <li>
        <button
          onClick={() => onChoice(figure)}
          className="p-1 md:p-3 hover:bg-black/25"
        >
          <Piece className="h-10 w-10" piece={{ figure, color }} />
        </button>
      </li>
    );
  };

  return (
    <div className="absolute z-10 inset-0 flex items-center justify-center bg-black/50">
      <ol className="flex flex-row gap-2 md:gap-5 shrink-0">
        <Option figure="Rook" />
        <Option figure="Knight" />
        <Option figure="Bishop" />
        <Option figure="Queen" />
      </ol>
    </div>
  );
}
