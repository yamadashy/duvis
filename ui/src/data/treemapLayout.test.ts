import { describe, expect, it } from "vitest";
import { buildHierarchy } from "./hierarchy";
import { layoutTreemap, PARENT_HEADER_MIN_HEIGHT_PX } from "./treemapLayout";
import type { Entry } from "./types";

const sample: Entry = {
  name: "root",
  size: 0,
  is_dir: true,
  category: "other",
  children: [
    { name: "a.txt", size: 1000, is_dir: false, category: "other" },
    { name: "b.txt", size: 500, is_dir: false, category: "other" },
    {
      name: "sub",
      size: 0,
      is_dir: true,
      category: "other",
      children: [{ name: "c.txt", size: 200, is_dir: false, category: "other" }],
    },
  ],
};

describe("layoutTreemap", () => {
  it("emits parents only for non-leaf nodes that have children", () => {
    const root = buildHierarchy(sample, "size");
    const { parents, leaves } = layoutTreemap(root, 400, 300, 18, 1, 3);
    // The "sub" directory is a parent frame; "root" itself is depth 0 so
    // it's filtered out by `depth > 0`.
    expect(parents.map((p) => p.data.name)).toEqual(["sub"]);
    // a.txt, b.txt, and c.txt sit as leaves.
    const leafNames = leaves.map((l) => l.data.name).sort();
    expect(leafNames).toEqual(["a.txt", "b.txt", "c.txt"]);
  });

  it("returns empty arrays for zero-size canvas", () => {
    const root = buildHierarchy(sample, "size");
    expect(layoutTreemap(root, 0, 100, 18, 1, 3)).toEqual({ parents: [], leaves: [] });
    expect(layoutTreemap(root, 100, 0, 18, 1, 3)).toEqual({ parents: [], leaves: [] });
  });

  it("respects maxDepth — cells below the cap are treated as leaves", () => {
    const root = buildHierarchy(sample, "size");
    // With maxDepth=1, sub's children should NOT appear as leaves; sub
    // itself becomes a depth-1 leaf (it sits at the depth cap and has
    // children below, but the layout drops anything past the cap).
    const { parents, leaves } = layoutTreemap(root, 400, 300, 18, 1, 1);
    expect(parents).toEqual([]); // no parents because depth < max means depth 0, filtered
    const leafNames = leaves.map((l) => l.data.name).sort();
    expect(leafNames).toEqual(["a.txt", "b.txt", "sub"]);
  });
});

describe("PARENT_HEADER_MIN_HEIGHT_PX", () => {
  it("is the threshold the renderer mirrors for dropping the header strip", () => {
    expect(PARENT_HEADER_MIN_HEIGHT_PX).toBeGreaterThan(0);
  });
});
