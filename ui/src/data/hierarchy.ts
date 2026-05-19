import { type HierarchyRectangularNode, hierarchy } from "d3-hierarchy";
import type { Entry, SortMode } from "./types";

// After treemap layout, nodes carry x0/x1/y0/y1 in addition to size info.
// We treat the unlaid hierarchy as the rectangular variant up-front so
// downstream components don't have to narrow types.
export type TreeNode = HierarchyRectangularNode<Entry>;

export function buildHierarchy(data: Entry, sort: SortMode): TreeNode {
  const h = hierarchy<Entry>(data, (d) => d.children);
  h.sum((d) => (d.children && d.children.length > 0 ? 0 : d.size));
  if (sort === "size") h.sort((a, b) => (b.value ?? 0) - (a.value ?? 0));
  else if (sort === "oldest")
    h.sort((a, b) => (b.data.modified_days_ago ?? 0) - (a.data.modified_days_ago ?? 0));
  else if (sort === "newest")
    h.sort((a, b) => (a.data.modified_days_ago ?? 0) - (b.data.modified_days_ago ?? 0));
  else if (sort === "name") h.sort((a, b) => a.data.name.localeCompare(b.data.name));
  return h as TreeNode;
}

/** Walk the data tree following a path of names. Returns the deepest reachable node. */
export function nodeAtPath(root: Entry, path: readonly string[]): Entry {
  let node: Entry = root;
  for (const seg of path) {
    const child = node.children?.find((c) => c.name === seg);
    if (!child) break;
    node = child;
  }
  return node;
}

export interface CategoryAggregates {
  byCategory: Record<string, number>;
  total: number;
  stale: number;
  fileCount: number;
}

export function aggregate(root: TreeNode, staleDays: number): CategoryAggregates {
  const byCategory: Record<string, number> = {};
  let fileCount = 0;
  let stale = 0;

  // Mirror src/output/summary.rs: when a directory carries a non-Other
  // category (propagated from its children), count the whole subtree under
  // that category and stop recursing. Otherwise descend into the children.
  function visit(n: TreeNode) {
    const isLeaf = !n.children || n.children.length === 0;
    if (isLeaf) {
      byCategory[n.data.category] = (byCategory[n.data.category] ?? 0) + (n.value ?? 0);
      fileCount++;
      if ((n.data.modified_days_ago ?? 0) > staleDays) stale += n.value ?? 0;
      return;
    }
    if (n.data.category !== "other") {
      byCategory[n.data.category] = (byCategory[n.data.category] ?? 0) + (n.value ?? 0);
      // Approximate file count and stale total for the whole subtree.
      n.each((m) => {
        if (!m.children || m.children.length === 0) {
          fileCount++;
          if ((m.data.modified_days_ago ?? 0) > staleDays) stale += m.value ?? 0;
        }
      });
      return;
    }
    for (const c of n.children!) visit(c);
  }
  visit(root);

  return {
    byCategory,
    total: root.value ?? 0,
    stale,
    fileCount,
  };
}
