import { hierarchy } from "d3-hierarchy";
import { describe, expect, it } from "vitest";
import type { TreeNode } from "./hierarchy";
import {
  buildSubtreeMatchSet,
  isActive,
  nameMatchesSearch,
  normalizeSearchQuery,
} from "./search";
import type { Category, Entry } from "./types";

const make = (data: Entry): TreeNode =>
  hierarchy<Entry>(data, (d) => d.children).sum(() => 0) as TreeNode;

describe("normalizeSearchQuery", () => {
  it("trims whitespace and lowercases", () => {
    expect(normalizeSearchQuery("  Foo Bar  ")).toBe("foo bar");
  });

  it("returns empty string for whitespace-only input", () => {
    expect(normalizeSearchQuery("   ")).toBe("");
  });
});

describe("nameMatchesSearch", () => {
  const node = make({ name: "Cargo.lock", size: 0, is_dir: false, category: "build" });

  it("matches everything on empty query", () => {
    expect(nameMatchesSearch(node, "")).toBe(true);
  });

  it("matches case-insensitively against the basename", () => {
    expect(nameMatchesSearch(node, "cargo")).toBe(true);
    expect(nameMatchesSearch(node, ".lock")).toBe(true);
    expect(nameMatchesSearch(node, "missing")).toBe(false);
  });
});

describe("buildSubtreeMatchSet", () => {
  it("returns null when query is empty so callers can short-circuit", () => {
    const root = make({ name: "root", size: 0, is_dir: true, category: "other" });
    expect(buildSubtreeMatchSet(root, "")).toBeNull();
  });

  it("includes every ancestor of a matching descendant", () => {
    const root = make({
      name: "root",
      size: 0,
      is_dir: true,
      category: "other",
      children: [
        {
          name: "src",
          size: 0,
          is_dir: true,
          category: "other",
          children: [{ name: "main.ts", size: 100, is_dir: false, category: "other" }],
        },
        { name: "README.md", size: 50, is_dir: false, category: "other" },
      ],
    });
    const set = buildSubtreeMatchSet(root, "main");
    if (!set) throw new Error("expected non-null match set");
    const names = Array.from(set, (n) => n.data.name).sort();
    expect(names).toEqual(["main.ts", "root", "src"]);
  });

  it("omits subtrees with no match", () => {
    const root = make({
      name: "root",
      size: 0,
      is_dir: true,
      category: "other",
      children: [
        { name: "hit.txt", size: 10, is_dir: false, category: "other" },
        {
          name: "miss-branch",
          size: 0,
          is_dir: true,
          category: "other",
          children: [{ name: "deep.txt", size: 10, is_dir: false, category: "other" }],
        },
      ],
    });
    const set = buildSubtreeMatchSet(root, "hit");
    if (!set) throw new Error("expected non-null match set");
    const names = Array.from(set, (n) => n.data.name).sort();
    // root is in (one of its descendants matched), but the miss-branch
    // subtree must not be.
    expect(names).toEqual(["hit.txt", "root"]);
  });
});

describe("isActive", () => {
  const node = make({ name: "x", size: 0, is_dir: false, category: "cache" });

  it("is true when the node's category is in the active set", () => {
    expect(isActive(node, new Set<Category>(["cache", "log"]))).toBe(true);
  });

  it("is false when the node's category is missing from the set", () => {
    expect(isActive(node, new Set<Category>(["log"]))).toBe(false);
  });
});
