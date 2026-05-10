import { hierarchy, type HierarchyRectangularNode, treemap } from "d3-hierarchy";
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

export function isActive(node: TreeNode, activeCategories: ReadonlySet<string>): boolean {
  return activeCategories.has(node.data.category);
}

/** Case-insensitive substring match on the node's basename. Empty query
 *  matches everything (so callers can wire the same predicate regardless
 *  of whether the user has typed in the search box). */
export function nameMatchesSearch(node: TreeNode, query: string): boolean {
  if (!query) return true;
  return node.data.name.toLowerCase().includes(query.toLowerCase());
}

/** True if the node itself or any descendant matches the search query.
 *  Used to keep parent cells "lit" when a child matches — otherwise the
 *  match would be hidden inside a dimmed ancestor. Empty query → true. */
export function subtreeMatchesSearch(node: TreeNode, query: string): boolean {
  if (!query) return true;
  const q = query.toLowerCase();
  let hit = false;
  node.each((n) => {
    if (!hit && n.data.name.toLowerCase().includes(q)) hit = true;
  });
  return hit;
}

export interface LaidOutTree {
  parents: TreeNode[];
  leaves: TreeNode[];
}

/** Cells smaller than this on either axis are not rendered: they're invisible
 *  to the eye, useless as hover/click targets, and just bloat the React tree. */
const MIN_CELL_PX = 2;
/** Hard cap on visible leaves. Above this we keep the largest by area and
 *  silently drop the rest — empirically the browser stays smooth with a few
 *  thousand SVG <g> nodes; 5000 leaves headroom for parents. */
const MAX_RENDER_LEAVES = 5000;
/** Cells shorter than this drop their header strip. At deep depth the headers
 *  otherwise stack up and crowd out the actual leaves. Exported so the
 *  renderer can mirror the same threshold when deciding what to draw. */
export const PARENT_HEADER_MIN_HEIGHT_PX = 50;

export function layoutTreemap(
  root: TreeNode,
  width: number,
  height: number,
  paddingTop: number,
  paddingInner: number,
  maxDepth: number,
): LaidOutTree {
  // round(true) snaps to integer pixels, which would erase any sub-pixel
  // padding we set. Round only when the gap is at least 1px.
  // paddingTop is adaptive: cells too short to host a header + meaningful
  // children just give all their height to the children. d3 calls this after
  // the node's own y0/y1 are set, so we can use them.
  treemap<Entry>()
    .size([width, height])
    .paddingOuter(0)
    .paddingTop((node) => {
      const n = node as TreeNode;
      return n.y1 - n.y0 >= PARENT_HEADER_MIN_HEIGHT_PX ? paddingTop : 0;
    })
    .paddingInner(paddingInner)
    .round(paddingInner >= 1)(root);

  // 1: top-level only; 2: + immediate children; 3: + grandchildren; ...
  const max = maxDepth;
  const visible = root.descendants().filter((n) => n.depth > 0 && n.depth <= max);

  const isRenderable = (n: TreeNode): boolean =>
    n.x1 - n.x0 >= MIN_CELL_PX && n.y1 - n.y0 >= MIN_CELL_PX;

  // A node is a "parent frame" (drawn as bordered header) when it sits above
  // the depth limit AND has its own children to host. Otherwise it is a leaf
  // cell. Sort parents shallow-first so SVG paint order layers them correctly.
  const parents = visible
    .filter(
      (n): n is TreeNode => n.depth < max && !!n.children && n.children.length > 0,
    )
    .filter(isRenderable)
    .sort((a, b) => a.depth - b.depth);

  let leaves = visible
    .filter((n) => {
      if (n.depth === max) return true;
      if (n.depth < max && (!n.children || n.children.length === 0)) return true;
      return false;
    })
    .filter(isRenderable);

  if (leaves.length > MAX_RENDER_LEAVES) {
    // Keep the biggest cells; the viewer can drill in to see smaller ones.
    leaves = [...leaves]
      .sort((a, b) => (b.x1 - b.x0) * (b.y1 - b.y0) - (a.x1 - a.x0) * (a.y1 - a.y0))
      .slice(0, MAX_RENDER_LEAVES);
  }

  return { parents, leaves };
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
