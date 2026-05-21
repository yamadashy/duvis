import { describe, expect, it } from "vitest";
import { humanSize, pct, relTime, truncate } from "./format";

describe("humanSize", () => {
  it("formats bytes under 1KiB with no decimals", () => {
    expect(humanSize(0)).toBe("0 B");
    expect(humanSize(512)).toBe("512 B");
    expect(humanSize(1023)).toBe("1023 B");
  });

  it("formats KiB with 1 decimal", () => {
    expect(humanSize(1024)).toBe("1.0 KB");
    expect(humanSize(1536)).toBe("1.5 KB");
  });

  it("formats MiB / GiB / TiB at expected thresholds", () => {
    expect(humanSize(1024 ** 2)).toBe("1.0 MB");
    expect(humanSize(1024 ** 3)).toBe("1.00 GB");
    expect(humanSize(1024 ** 4)).toBe("1.00 TB");
  });
});

describe("pct", () => {
  it('returns "0%" when whole is 0 to avoid division-by-zero', () => {
    expect(pct(0, 0)).toBe("0%");
    expect(pct(5, 0)).toBe("0%");
  });

  it("renders with 1 decimal", () => {
    expect(pct(50, 100)).toBe("50.0%");
    expect(pct(1, 3)).toBe("33.3%");
  });
});

describe("relTime", () => {
  it('returns "—" for undefined', () => {
    expect(relTime(undefined)).toBe("—");
  });

  it("uses words for today/yesterday", () => {
    expect(relTime(0)).toBe("today");
    expect(relTime(1)).toBe("yesterday");
  });

  it("scales unit to the magnitude", () => {
    expect(relTime(3)).toBe("3d ago");
    expect(relTime(14)).toBe("2w ago");
    expect(relTime(60)).toBe("2mo ago");
    expect(relTime(800)).toBe("2y ago");
  });
});

describe("truncate", () => {
  it("returns the string unchanged when it fits", () => {
    expect(truncate("hello", 10)).toBe("hello");
    expect(truncate("hello", 5)).toBe("hello");
  });

  it("appends ellipsis when truncating", () => {
    expect(truncate("hello world", 8)).toBe("hello w…");
  });

  it("hard-cuts without ellipsis when budget is too tight", () => {
    // 4 chars or fewer of budget can't afford the ellipsis itself.
    expect(truncate("hello", 3)).toBe("hel");
    expect(truncate("hello", 1)).toBe("h");
  });
});
