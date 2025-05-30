import { BebopView } from 'bebop';
import * as G from './generated/gen';
import { it, expect } from 'vitest';
it("Constants are generated", () => {
    expect(G.exampleConstInt32).toEqual(-123);
    expect(G.exampleConstUint64).toEqual(0x123ffffffffn);
    expect(G.exampleConstFloat64).toEqual(123.45678e9);
    expect(G.exampleConstInf).toEqual(Number.POSITIVE_INFINITY);
    expect(G.exampleConstNegInf).toEqual(Number.NEGATIVE_INFINITY);
    expect(G.exampleConstNan).toEqual(Number.NaN);
    expect(G.exampleConstFalse).toEqual(false);
    expect(G.exampleConstTrue).toEqual(true);
    expect(G.exampleConstString).toEqual("hello \"world\"\nwith newlines");
    expect(G.exampleConstGuid === "e215a946-b26f-4567-a276-13136f0a1708").toBe(true);
});

