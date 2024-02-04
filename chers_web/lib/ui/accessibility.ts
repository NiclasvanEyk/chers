import { Cell, Coordinate } from "../chers";

export function cellLabel(
	position: Coordinate,
	captures: boolean,
	contents: Cell,
): string {
	const origin = algebraicNotation(position);
	if (contents) {
		const { color, figure } = contents;
		return `${
			captures ? "capture" : ""
		} ${color} ${figure} at ${origin}`.trim();
	}

	return origin;
}

function algebraicNotation({ x, y }: Coordinate): string {
	const file =
		{
			1: "a",
			2: "b",
			3: "c",
			4: "d",
			5: "e",
			6: "f",
			7: "g",
			8: "h",
		}[8 - y] ?? null;
	if (file === null) {
		throw new Error(`Invalid y-coordinate: ${y}`);
	}

	const rank = 8 - x;
	if (rank < 1 || rank > 8) {
		throw new Error(`Invalid x-coordinate: ${x}`);
	}

	return `${file}${rank}`;
}
