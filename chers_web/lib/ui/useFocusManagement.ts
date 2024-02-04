import { Dispatcher, State } from "@/lib/ui/state";
import { useEffect, useRef } from "react";

type CellsByCoordinate = Array<Array<HTMLButtonElement>>;

/**
 * Allows the user to play the game using the keyboard.
 *
 * Arrow keys navigate the currently focused cell.
 * Escape key aborts the move.
 */
export function useFocusManagement(state: State, dispatch: Dispatcher) {
	const cells = useRef([] as CellsByCoordinate);

	function listener(event: KeyboardEvent) {
		escapeKeyAbortsMove(event, dispatch);
		arrowKeysNavigateCells(event, cells.current);
	}

	useEffect(
		function () {
			document.addEventListener("keydown", listener);
			return () => {
				document.removeEventListener("keydown", listener);
			};
		},
		[state, dispatch],
	);

	return {
		registerCellRef(x: number, y: number, cell: HTMLButtonElement | null) {
			let row = cells.current[y];
			if (!Array.isArray(row)) {
				row = [];
				cells.current[y] = row;
			}

			if (cell) {
				row[x] = cell;
			} else {
				delete row[x];
			}
		},
	};
}

function escapeKeyAbortsMove(event: KeyboardEvent, dispatch: Dispatcher) {
	if (event.key !== "Escape") return;

	dispatch({ type: "ABORT_SELECTION" });
}

function arrowKeysNavigateCells(
	event: KeyboardEvent,
	cells: CellsByCoordinate,
) {
	const vector = {
		ArrowUp: [0, -1],
		ArrowDown: [0, 1],
		ArrowLeft: [-1, 0],
		ArrowRight: [1, 0],
	}[event.key];
	if (!vector) {
		return;
	}

	const activeCellCoordinates = findActiveCellCoordinates();
	if (activeCellCoordinates === null) {
		return;
	}

	const [x, y] = [
		activeCellCoordinates[0] + vector[0],
		activeCellCoordinates[1] + vector[1],
	];

	cells[y]?.[x]?.focus();
}

function findActiveCellCoordinates(): [number, number] | null {
	const focused = document.activeElement;
	if (!(focused instanceof HTMLElement)) return null;

	let [rawX, rawY] = [focused.dataset.x, focused.dataset.y];
	if (rawX === undefined) return null;
	const x = Number.parseInt(rawX);

	if (rawY === undefined) return null;
	const y = Number.parseInt(rawY);

	return [x, y];
}
