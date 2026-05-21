import { describe, expect, it } from "vitest";
import { clampColumn, COLUMN_MAX, COLUMN_MIN } from "./columnWidths";

describe("clampColumn", () => {
  it("clamps to COLUMN_MIN at the low end", () => {
    expect(clampColumn(0)).toBe(COLUMN_MIN);
    expect(clampColumn(-100)).toBe(COLUMN_MIN);
  });

  it("clamps to COLUMN_MAX at the high end", () => {
    expect(clampColumn(9_999)).toBe(COLUMN_MAX);
  });

  it("rounds fractional values", () => {
    expect(clampColumn(304.6)).toBe(305);
  });

  it("passes through valid values inside the range", () => {
    expect(clampColumn(304)).toBe(304);
    expect(clampColumn(COLUMN_MIN)).toBe(COLUMN_MIN);
    expect(clampColumn(COLUMN_MAX)).toBe(COLUMN_MAX);
  });
});
