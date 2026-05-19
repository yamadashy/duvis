import type { TreeNode } from "./hierarchy";

export function isActive(node: TreeNode, activeCategories: ReadonlySet<string>): boolean {
  return activeCategories.has(node.data.category);
}

/** Single source of truth for "the search query as the predicates expect
 *  it" — trim whitespace, then lowercase. Callers `useMemo` the result
 *  and pass it to every match helper, so trim/case semantics can't drift
 *  between Treemap, Sunburst, and ListView. */
export function normalizeSearchQuery(query: string): string {
  return query.trim().toLowerCase();
}

/** Case-insensitive substring match on the node's basename. The query
 *  is expected to be already normalized via `normalizeSearchQuery` —
 *  callers typically `useMemo` it once and reuse across thousands of
 *  cells, so the predicate doesn't pay for `trim`/`toLowerCase` per
 *  node. Empty query matches everything. */
export function nameMatchesSearch(node: TreeNode, normalizedQuery: string): boolean {
  if (!normalizedQuery) return true;
  return node.data.name.toLowerCase().includes(normalizedQuery);
}

/** Walk the tree once and return the set of nodes whose subtree (self
 *  or any descendant) contains a name match for `normalizedQuery`.
 *  Returns `null` when the query is empty so callers can short-circuit
 *  cheaply (`null` ↔ "no filter active"). The walk is post-order with
 *  an early exit per branch — once a child reports a hit, the parent
 *  stops scanning further siblings to confirm its own membership. */
export function buildSubtreeMatchSet(
  root: TreeNode,
  normalizedQuery: string,
): ReadonlySet<TreeNode> | null {
  if (!normalizedQuery) return null;
  const set = new Set<TreeNode>();
  const visit = (n: TreeNode): boolean => {
    let hit = n.data.name.toLowerCase().includes(normalizedQuery);
    if (n.children) {
      for (const c of n.children as TreeNode[]) {
        if (visit(c)) hit = true;
      }
    }
    if (hit) set.add(n);
    return hit;
  };
  visit(root);
  return set;
}
