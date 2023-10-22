import { Color, PromotedFigure } from "@/lib/chers";
import { Piece } from "./Piece";

export interface PromotionProps {
  onChoice: (figure: PromotedFigure) => void;
  color: Color;
}

export function Promotion({ color, onChoice }: PromotionProps) {
  function focusIfPlayingByKeyboard(dialog: HTMLDialogElement | null) {
    if (dialog === null) return;
    if (document.activeElement === null) return;

    dialog.focus();
  }

  const Option = ({
    figure,
    tabindex,
  }: {
    figure: PromotedFigure;
    tabindex: number;
  }) => {
    return (
      <li>
        <button
          tabIndex={tabindex}
          onClick={() => onChoice(figure)}
          className="p-1 md:p-3 hover:bg-black/25"
        >
          <Piece className="h-10 w-10" piece={{ figure, color }} />
        </button>
      </li>
    );
  };

  return (
    <dialog
      open={true}
      ref={focusIfPlayingByKeyboard}
      className="absolute z-10 inset-0 flex items-center justify-center bg-black/50"
    >
      <ol className="flex flex-row gap-2 md:gap-5 shrink-0">
        <Option tabindex={1} figure="Rook" />
        <Option tabindex={2} figure="Knight" />
        <Option tabindex={3} figure="Bishop" />
        <Option tabindex={4} figure="Queen" />
      </ol>
    </dialog>
  );
}
