import { useEffect, useMemo, useRef, useState } from "react";
import type { TreeNode } from "../../data/hierarchy";
import { isActive, nameMatchesSearch, normalizeSearchQuery } from "../../data/search";
import { layoutTreemap } from "../../data/treemapLayout";
import type { Category } from "../../data/types";
import { LeafCell } from "./LeafCell";
import { PAD_TOP, ParentFrame } from "./ParentFrame";
import styles from "./Treemap.module.css";
import { nodeKey, nodesEqual } from "./label";

interface TreemapProps {
  root: TreeNode;
  selected: TreeNode | null;
  filterCategories: ReadonlySet<Category>;
  searchQuery: string;
  treemapPadding: number;
  treemapRadius: number;
  maxDepth: number;
  onSelect: (node: TreeNode) => void;
  onDrillIn: (node: TreeNode) => void;
  onHover: (node: TreeNode | null, evt: { clientX: number; clientY: number } | null) => void;
}

export function Treemap(props: TreemapProps) {
  const {
    root,
    selected,
    filterCategories,
    searchQuery,
    treemapPadding,
    treemapRadius,
    maxDepth,
    onSelect,
    onDrillIn,
    onHover,
  } = props;

  const wrapRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ w: number; h: number } | null>(null);

  useEffect(() => {
    const el = wrapRef.current;
    if (!el) return;
    const measure = () => setSize({ w: el.clientWidth, h: el.clientHeight });
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Memoize the layout pass: hover and selection changes re-render the
  // <Treemap> but they don't change any of the layout inputs, so without
  // this cache every cell hover would force the full d3 treemap squarify
  // pass over the whole tree.
  const { parents, leaves } = useMemo(() => {
    if (!size || size.w <= 0 || size.h <= 0) {
      return { parents: [] as TreeNode[], leaves: [] as TreeNode[] };
    }
    return layoutTreemap(root, size.w, size.h, PAD_TOP, treemapPadding, maxDepth);
  }, [root, size, treemapPadding, maxDepth]);

  // Normalize the query once per render rather than per-cell. Treemap
  // can render thousands of leaves; the previous helper toLowerCase'd
  // the query inside the predicate, so the cost scaled with cell count.
  const loweredQuery = useMemo(() => normalizeSearchQuery(searchQuery), [searchQuery]);

  return (
    <div className="view-wrap" ref={wrapRef}>
      <svg
        className={styles.treemapSvg}
        viewBox={size ? `0 0 ${size.w} ${size.h}` : undefined}
        preserveAspectRatio="none"
        role="img"
        aria-label="Treemap of file sizes by directory"
      >
        <title>Treemap of file sizes by directory</title>
        <defs>
          <pattern
            id="stale-pattern"
            width="6"
            height="6"
            patternUnits="userSpaceOnUse"
            patternTransform="rotate(45)"
          >
            <line x1="0" y1="0" x2="0" y2="6" stroke="rgba(0,0,0,.15)" strokeWidth="1" />
          </pattern>
        </defs>

        <g>
          {parents.map((p) => (
            <ParentFrame
              key={`p-${nodeKey(p)}`}
              node={p}
              radius={treemapRadius}
              onSelect={() => onSelect(p)}
              onDrillIn={() => onDrillIn(p)}
              onHoverEnter={(e) => onHover(p, e)}
              onHoverMove={(e) => onHover(p, e)}
              onHoverLeave={() => onHover(null, null)}
            />
          ))}
        </g>

        <g>
          {leaves.map((n) => (
            <LeafCell
              key={`l-${nodeKey(n)}`}
              node={n}
              radius={treemapRadius}
              dim={!isActive(n, filterCategories) || !nameMatchesSearch(n, loweredQuery)}
              isSelected={!!selected && nodesEqual(selected, n)}
              onSelect={() => onSelect(n)}
              onDrillIn={() => {
                if (n.children && n.children.length > 0) onDrillIn(n);
              }}
              onHoverEnter={(e) => onHover(n, e)}
              onHoverMove={(e) => onHover(n, e)}
              onHoverLeave={() => onHover(null, null)}
            />
          ))}
        </g>
      </svg>
    </div>
  );
}
