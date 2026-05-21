import type { TreeNode } from "../../data/hierarchy";

/** Stable identifier for a TreeNode based on its full ancestry path.
 *  d3-hierarchy rebuilds node references on every re-layout (sort change,
 *  data slice change), so identity comparisons would remount every cell.
 *  The ancestry name path is stable across re-layouts. */
export function nodeKey(node: TreeNode): string {
  return node
    .ancestors()
    .map((n) => n.data.name)
    .join("/");
}

/** Treemap cells get keyed by full ancestry path, not object identity:
 *  re-layouts after a sort change rebuild the d3 hierarchy, so the
 *  React-rendered `selected` reference no longer matches the new layout's
 *  nodes. Compare by name path so the selection ring follows the file. */
export function nodesEqual(a: TreeNode, b: TreeNode): boolean {
  return nodeKey(a) === nodeKey(b);
}

/** Tiny conditional-class joiner (avoids pulling in clsx for one helper). */
export function cn(...classes: Array<string | false | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

/** Truncate `s` to `n` chars, appending `…` when truncation actually
 *  happens. Sub-4-char budgets just hard-cut — an ellipsis would eat all
 *  the available space. */
export function trim(s: string, n: number): string {
  if (s.length <= n) return s;
  if (n < 4) return s.slice(0, n);
  return `${s.slice(0, n - 1)}…`;
}

/** Pixel-fit a treemap label: figure out how much room is left after
 *  padding + the dot, then split the budget between name and size text.
 *  Dropping the size text once the name budget would shrink under 4
 *  chars keeps the result legible at small widths. */
export function fitParentLabel(name: string, size: string, width: number, showDot: boolean) {
  // 8px left padding; 16px reserved on the right for the dot when visible
  // (4px otherwise).
  const reservedRight = showDot ? 16 : 4;
  const availPx = Math.max(0, width - 8 - reservedRight);
  const CHAR_PX = 6.5;
  const totalChars = Math.floor(availPx / CHAR_PX);
  const sharedNameBudget = totalChars - size.length - 1;
  const showSize = sharedNameBudget >= 4;
  const nameDisplay = showSize ? trim(name, sharedNameBudget) : trim(name, totalChars);
  return { nameDisplay, showSize };
}
