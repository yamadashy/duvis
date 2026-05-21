import { describe, expect, it } from "vitest";
import { aggregate, buildHierarchy, nodeAtPath } from "./hierarchy";
import type { Entry } from "./types";

const tree: Entry = {
  name: "root",
  size: 0,
  is_dir: true,
  category: "other",
  children: [
    {
      name: "node_modules",
      size: 0,
      is_dir: true,
      category: "cache",
      children: [
        { name: "react.js", size: 5_000, is_dir: false, category: "cache" },
        { name: "vite.js", size: 3_000, is_dir: false, category: "cache" },
      ],
    },
    {
      name: "src",
      size: 0,
      is_dir: true,
      category: "other",
      children: [
        { name: "main.ts", size: 1_000, is_dir: false, category: "other" },
        { name: "old.log", size: 500, is_dir: false, category: "log", modified_days_ago: 400 },
      ],
    },
  ],
};

describe("nodeAtPath", () => {
  it("returns the root for an empty path", () => {
    expect(nodeAtPath(tree, []).name).toBe("root");
  });

  it("walks one segment at a time", () => {
    expect(nodeAtPath(tree, ["src"]).name).toBe("src");
    expect(nodeAtPath(tree, ["src", "main.ts"]).name).toBe("main.ts");
  });

  it("stops at the deepest reachable node when the path overshoots", () => {
    // main.ts has no children; ["src", "main.ts", "ghost"] should land
    // on main.ts rather than crashing.
    expect(nodeAtPath(tree, ["src", "main.ts", "ghost"]).name).toBe("main.ts");
  });

  it("stops at the last matching segment when a name is wrong", () => {
    expect(nodeAtPath(tree, ["src", "nope"]).name).toBe("src");
  });
});

describe("buildHierarchy", () => {
  it('sums leaf sizes onto each ancestor (sort="size")', () => {
    const h = buildHierarchy(tree, "size");
    expect(h.value).toBe(9_500);
    // node_modules should now sort first (8_000 > src 1_500)
    const firstChildName = h.children?.[0]?.data.name;
    expect(firstChildName).toBe("node_modules");
  });

  it('sort="name" orders children alphabetically', () => {
    const h = buildHierarchy(tree, "name");
    const names = h.children?.map((c) => c.data.name);
    expect(names).toEqual(["node_modules", "src"]);
  });

  it('sort="oldest" puts older entries first', () => {
    const h = buildHierarchy(tree, "oldest");
    const src = h.children?.find((c) => c.data.name === "src");
    // old.log (400 days) should sort before main.ts (no modified_days_ago)
    const firstLeaf = src?.children?.[0]?.data.name;
    expect(firstLeaf).toBe("old.log");
  });
});

describe("aggregate", () => {
  it("collapses each non-other dir's whole subtree under its own category", () => {
    const h = buildHierarchy(tree, "size");
    const agg = aggregate(h, 90);
    // node_modules carries `cache`, so its 8_000 should land in `cache`
    // as one bucket without re-splitting.
    expect(agg.byCategory.cache).toBe(8_000);
    expect(agg.total).toBe(9_500);
  });

  it("counts subtree files and stale bytes", () => {
    const h = buildHierarchy(tree, "size");
    const agg = aggregate(h, 90);
    // 4 leaves total: 2 in node_modules, 2 in src
    expect(agg.fileCount).toBe(4);
    // Only old.log is older than 90d (400 > 90)
    expect(agg.stale).toBe(500);
  });

  it('treats "other"-category dirs as transparent and descends into them', () => {
    const h = buildHierarchy(tree, "size");
    const agg = aggregate(h, 90);
    // src is "other", so we should see its kids' categories surfaced:
    // main.ts -> other (1000), old.log -> log (500)
    expect(agg.byCategory.other).toBe(1_000);
    expect(agg.byCategory.log).toBe(500);
  });
});
