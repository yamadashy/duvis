import { treemap } from "d3-hierarchy";
import type { Entry } from "./types";
import type { TreeNode } from "./hierarchy";

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
