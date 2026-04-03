import { expect, test } from "bun:test";
import { buildChersReducer } from "./state";
import { INITIAL_STATE } from "./testing/state";

const noop = (() => {}) as unknown as any;

test("Initial moves", () => {
  const reduce = buildChersReducer({
    nextState: noop,
    getMoves: () => [{ x: -1, y: -1 }],
  });
  const next = reduce(INITIAL_STATE, {
    type: "SELECT_FROM",
    from: { x: 1, y: 7 },
  });

  expect(next.type).toBe("SELECTING_TO");
  expect(next.moves).toEqual([{ x: -1, y: -1 }]);
});
