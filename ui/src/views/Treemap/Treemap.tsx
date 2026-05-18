import { useEffect, useMemo, useRef, useState } from "react";
import type { TreeNode } from "../../data/hierarchy";
import { isActive, nameMatchesSearch, normalizeSearchQuery } from "../../data/search";
import { layoutTreemap } from "../../data/treemapLayout";
import type { Category } from "../../data/types";
import { LeafCell } from "./LeafCell";
import { nodesEqual } from "./label";
import { PAD_TOP, ParentFrame } from "./ParentFrame";
import "./Treemap.css";

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

  let parents: TreeNode[] = [];
  let leaves: TreeNode[] = [];
  if (size && size.w > 0 && size.h > 0) {
    const out = layoutTreemap(root, size.w, size.h, PAD_TOP, treemapPadding, maxDepth);
    parents = out.parents;
    leaves = out.leaves;
  }

  // Normalize the query once per render rather than per-cell. Treemap
  // can render thousands of leaves; the previous helper toLowerCase'd
  // the query inside the predicate, so the cost scaled with cell count.
  const loweredQuery = useMemo(() => normalizeSearchQuery(searchQuery), [searchQuery]);

  return (
    <div className="treemap-wrap" ref={wrapRef}>
      <svg
        className="treemap-svg"
        viewBox={size ? `0 0 ${size.w} ${size.h}` : undefined}
        preserveAspectRatio="none"
      >
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

        <g className="tm-parents">
          {parents.map((p, i) => (
            <ParentFrame
              key={`p-${i}-${p.data.name}`}
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

        <g className="tm-leaves">
          {leaves.map((n, i) => (
            <LeafCell
              key={`l-${i}-${n.data.name}`}
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
