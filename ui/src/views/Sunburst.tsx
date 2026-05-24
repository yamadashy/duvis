import { type HierarchyRectangularNode, hierarchy, partition } from "d3-hierarchy";
import { arc as d3arc } from "d3-shape";
import { type JSX, useEffect, useMemo, useRef, useState } from "react";
import { categoryVar, LIGHT_FILL_CATEGORIES } from "../data/categories";
import { humanSize } from "../data/format";
import type { TreeNode } from "../data/hierarchy";
import { buildSubtreeMatchSet, isActive, normalizeSearchQuery } from "../data/search";
import type { Category, Entry } from "../data/types";
import styles from "./Sunburst.module.css";
import { nodeKey } from "./Treemap/label";

interface SunburstProps {
  root: TreeNode;
  selected: TreeNode | null;
  filterCategories: ReadonlySet<Category>;
  searchQuery: string;
  rootPathLength: number;
  maxDepth: number;
  onSelect: (node: TreeNode) => void;
  onDrillIn: (node: TreeNode) => void;
  onUp: () => void;
  onHover: (node: TreeNode | null, evt: { clientX: number; clientY: number } | null) => void;
}

type PartitionNode = HierarchyRectangularNode<Entry>;

function trim(s: string, n: number): string {
  if (s.length <= n) return s;
  if (n < 4) return s.slice(0, n);
  return `${s.slice(0, n - 1)}…`;
}

export function Sunburst(props: SunburstProps) {
  const {
    root,
    selected,
    filterCategories,
    searchQuery,
    rootPathLength,
    maxDepth,
    onSelect,
    onDrillIn,
    onUp,
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

  // Precompute the set of nodes whose subtree contains a name match.
  // Previously this was recomputed per-arc via a full subtree walk —
  // O(N²) on big scans and visibly laggy while typing. The walk is now
  // O(N), the per-arc check is a Set membership lookup, and the result
  // is reused while `searchQuery` is unchanged.
  // Must sit above the early `if (!size)` return so the hook count stays
  // stable across the "measuring" → "ready" transition (React #310).
  const matchSet = useMemo(
    () => buildSubtreeMatchSet(root, normalizeSearchQuery(searchQuery)),
    [root, searchQuery],
  );

  if (!size || size.w === 0 || size.h === 0) {
    return <div className="view-wrap" ref={wrapRef} />;
  }

  const cx = size.w / 2;
  const cy = size.h / 2;
  const radius = Math.min(size.w, size.h) / 2 - 24;
  const innerHole = Math.max(48, radius * 0.32);
  const ringWidth = (radius - innerHole) / maxDepth;

  // Build a fresh partition layout from the same data tree.
  const partRoot = partition<Entry>().size([2 * Math.PI, maxDepth + 1])(
    hierarchy<Entry>(root.data, (d) => d.children)
      .sum((d) => (d.children && d.children.length > 0 ? 0 : d.size))
      .sort((a, b) => (b.value ?? 0) - (a.value ?? 0)),
  ) as PartitionNode;

  const arcGen = d3arc<PartitionNode>()
    .startAngle((d) => d.x0)
    .endAngle((d) => d.x1)
    .innerRadius((d) => innerHole + (d.depth - 1) * ringWidth)
    .outerRadius((d) => innerHole + d.depth * ringWidth)
    .padAngle(0.003)
    .padRadius(radius);

  const visible = partRoot.descendants().filter((d) => d.depth >= 1 && d.depth <= maxDepth);

  // Map a partition node back to the live `root` hierarchy by following names.
  function findInRoot(p: PartitionNode): TreeNode | null {
    const path: string[] = [];
    let n: PartitionNode | null = p;
    while (n?.parent) {
      path.unshift(n.data.name);
      n = n.parent as PartitionNode | null;
    }
    let cur: TreeNode | undefined = root;
    for (const seg of path) {
      const next: TreeNode | undefined = cur.children?.find((c) => c.data.name === seg);
      if (!next) return null;
      cur = next;
    }
    return cur ?? null;
  }

  return (
    <div className="view-wrap" ref={wrapRef}>
      <svg
        className={styles.sunburstSvg}
        viewBox={`0 0 ${size.w} ${size.h}`}
        role="img"
        aria-label="Sunburst chart of file sizes by directory"
      >
        <title>Sunburst chart of file sizes by directory</title>
        <g transform={`translate(${cx},${cy})`}>
          {visible.map((d) => {
            const angleSpan = d.x1 - d.x0;
            if (angleSpan < 0.005) return null;
            const cat: Category = d.data.category;
            const live = findInRoot(d);
            // For ring arcs (which can sit above leaves), match if the
            // arc's subtree contains a hit — otherwise a deep match would
            // be hidden behind a dimmed parent ring. `matchSet === null`
            // means no search filter is active.
            const matches = matchSet === null || (live !== null && matchSet.has(live));
            const active = (live ? isActive(live, filterCategories) : true) && matches;
            const isSel = !!selected && live === selected;
            // Match Treemap LeafCell: parents (with children) slightly more
            // opaque than leaves. Avoids depth-based fade that made deep
            // sunburst rings look washed-out compared to treemap cells of
            // the same category.
            const hasChildren = !!d.children && d.children.length > 0;
            const baseOpacity = hasChildren ? 0.85 : 0.75;

            const path = arcGen(d) ?? "";

            // Label placement
            const midAngle = (d.x0 + d.x1) / 2;
            const arcLength = angleSpan * (innerHole + (d.depth - 0.5) * ringWidth);
            const showLabel = arcLength > 36 && ringWidth > 14;
            let label: JSX.Element | null = null;
            if (showLabel) {
              const r = innerHole + (d.depth - 0.5) * ringWidth;
              const x = Math.sin(midAngle) * r;
              const y = -Math.cos(midAngle) * r;
              const rotateDeg = (midAngle * 180) / Math.PI - 90;
              const flip = midAngle > Math.PI ? 180 : 0;
              const text = trim(d.data.name, Math.floor(arcLength / 6));
              const dark = LIGHT_FILL_CATEGORIES.has(cat);
              label = (
                <text
                  className={dark ? `${styles.sbLabel} ${styles.dark}` : styles.sbLabel}
                  transform={`translate(${x},${y}) rotate(${rotateDeg + flip})`}
                  dy="0.35em"
                >
                  {text}
                </text>
              );
            }

            return (
              <g key={`a-${nodeKey(d)}`}>
                <path
                  className={styles.sbArc}
                  d={path}
                  fill={categoryVar(cat)}
                  fillOpacity={baseOpacity}
                  data-dim={!active}
                  data-selected={isSel}
                  onMouseEnter={(e) => live && onHover(live, e)}
                  onMouseMove={(e) => live && onHover(live, e)}
                  onMouseLeave={() => onHover(null, null)}
                  onClick={() => {
                    if (live) onSelect(live);
                  }}
                  onDoubleClick={() => {
                    if (live?.children && live.children.length > 0) {
                      onDrillIn(live);
                    }
                  }}
                />
                {label}
              </g>
            );
          })}

          <circle
            r={innerHole - 4}
            fill="var(--bg-1)"
            stroke="var(--line)"
            strokeWidth={1}
            style={{ cursor: rootPathLength > 0 ? "pointer" : "default" }}
            onClick={() => {
              if (rootPathLength > 0) onUp();
            }}
          />

          <text className={styles.sbCenterName} dy="-0.2em">
            {trim(root.data.name + (root.data.children ? "/" : ""), 18)}
          </text>
          <text className={styles.sbCenterSize} dy="1.1em">
            {humanSize(root.value ?? 0)}
          </text>
          {rootPathLength > 0 ? (
            <text className={styles.sbCenterHint} dy="2.6em">
              ← click to go up
            </text>
          ) : null}
        </g>
      </svg>
    </div>
  );
}
