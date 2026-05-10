import { useEffect, useRef, useState } from "react";
import { categoryVar, LIGHT_FILL_CATEGORIES } from "../lib/categories";
import { humanSize } from "../lib/format";
import {
  isActive,
  layoutTreemap,
  nameMatchesSearch,
  PARENT_HEADER_MIN_HEIGHT_PX,
  type TreeNode,
} from "../lib/treemap";
import type { Category, Entry } from "../lib/types";
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

const PAD_TOP = 18;

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
              dim={!isActive(n, filterCategories) || !nameMatchesSearch(n, searchQuery)}
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

interface FrameProps {
  node: TreeNode;
  radius: number;
  onSelect: () => void;
  onDrillIn: () => void;
  onHoverEnter: (e: { clientX: number; clientY: number }) => void;
  onHoverMove: (e: { clientX: number; clientY: number }) => void;
  onHoverLeave: () => void;
}

function ParentFrame({
  node,
  radius,
  onSelect,
  onDrillIn,
  onHoverEnter,
  onHoverMove,
  onHoverLeave,
}: FrameProps) {
  const w = node.x1 - node.x0;
  const h = node.y1 - node.y0;
  const cat = node.data.category;
  // Mirror the layout-side decision: cells too short don't get a header.
  const hasHeader = h >= PARENT_HEADER_MIN_HEIGHT_PX;
  const showDot = hasHeader && w > 50;
  const showLabel = hasHeader && w > 28;

  // Pixel-fit the label so it never escapes the cell. 8px left padding,
  // 16px reserved on the right when the dot is visible (4px otherwise).
  const reservedRight = showDot ? 16 : 4;
  const availPx = Math.max(0, w - 8 - reservedRight);
  const CHAR_PX = 6.5;
  const totalChars = Math.floor(availPx / CHAR_PX);
  const sizeText = humanSize(node.value ?? 0);
  // Need name + ~1 char gap + size text. Drop the size when the name budget
  // would shrink below 4 chars.
  const sharedNameBudget = totalChars - sizeText.length - 1;
  const showSize = sharedNameBudget >= 4;
  const nameDisplay = showSize
    ? trim(node.data.name, sharedNameBudget)
    : trim(node.data.name, totalChars);

  return (
    <g
      transform={`translate(${node.x0},${node.y0})`}
      className="tm-parent"
      style={{ cursor: "pointer" }}
      onClick={onSelect}
      onDoubleClick={onDrillIn}
      onMouseEnter={onHoverEnter}
      onMouseMove={onHoverMove}
      onMouseLeave={onHoverLeave}
    >
      <rect className="tm-parent-rect" width={w} height={h} rx={radius} />
      {hasHeader ? (
        <>
          <rect
            width={w}
            height={PAD_TOP}
            fill="var(--bg-2)"
            opacity={0.4}
            rx={radius}
          />
          {radius > 0 ? (
            <rect
              y={PAD_TOP / 2}
              width={w}
              height={PAD_TOP / 2}
              fill="var(--bg-2)"
              opacity={0.4}
            />
          ) : null}
        </>
      ) : null}
      {showLabel ? (
        <text className="tm-parent-label" x={8} y={12} fill="var(--fg)">
          <tspan>{nameDisplay}</tspan>
          {showSize ? (
            <tspan dx={6} opacity={0.5} fontWeight={400}>
              {sizeText}
            </tspan>
          ) : null}
        </text>
      ) : null}
      {showDot ? (
        <rect
          x={w - 12}
          y={6}
          width={6}
          height={6}
          rx={1.5}
          fill={categoryVar(cat)}
          opacity={0.7}
        />
      ) : null}
    </g>
  );
}

interface LeafProps {
  node: TreeNode;
  radius: number;
  dim: boolean;
  isSelected: boolean;
  onSelect: () => void;
  onDrillIn: () => void;
  onHoverEnter: (e: { clientX: number; clientY: number }) => void;
  onHoverMove: (e: { clientX: number; clientY: number }) => void;
  onHoverLeave: () => void;
}

function LeafCell(props: LeafProps) {
  const { node, radius, dim, isSelected, onSelect, onDrillIn, onHoverEnter, onHoverMove, onHoverLeave } =
    props;
  const w = node.x1 - node.x0;
  const h = node.y1 - node.y0;
  // Sub-pixel cells are filtered upstream in layoutTreemap, so no guard here.

  const cat: Category = node.data.category;
  const stale = (node.data.modified_days_ago ?? 0) > 180;
  const lightFill = LIGHT_FILL_CATEGORIES.has(cat);
  const showName = w > 50 && h > 22;
  const showSize = w > 90 && h > 36;
  const showSmallName = !showName && w > 30 && h > 14;

  return (
    <g
      transform={`translate(${node.x0},${node.y0})`}
      className="tm-node"
      data-dim={dim}
      data-selected={isSelected}
      style={{ cursor: node.children ? "pointer" : "default" }}
      onClick={onSelect}
      onDoubleClick={onDrillIn}
      onMouseEnter={onHoverEnter}
      onMouseMove={onHoverMove}
      onMouseLeave={onHoverLeave}
    >
      <rect
        className="tm-node-rect"
        width={w}
        height={h}
        rx={radius}
        fill={categoryVar(cat)}
        fillOpacity={node.children ? 0.85 : 0.75}
      />
      {stale ? (
        <rect
          width={w}
          height={h}
          rx={radius}
          fill="url(#stale-pattern)"
          pointerEvents="none"
        />
      ) : null}
      {showName ? (
        <>
          <text className={cn("tm-label", !lightFill && "light")} x={6} y={14}>
            <tspan className="tm-label-name">{trim(node.data.name, Math.floor(w / 6.5))}</tspan>
          </text>
          {showSize ? (
            <text className={cn("tm-label tm-label-size", !lightFill && "light")} x={6} y={28}>
              {humanSize(node.value ?? 0)}
            </text>
          ) : null}
        </>
      ) : showSmallName ? (
        <text className={cn("tm-label", !lightFill && "light")} x={4} y={11} fontSize={10}>
          {trim(node.data.name, Math.floor(w / 5.5))}
        </text>
      ) : null}
    </g>
  );
}

function nodesEqual(a: TreeNode, b: TreeNode): boolean {
  // Reference compare is not stable across re-layouts; compare by full path.
  const pathA = a.ancestors().map((n) => n.data.name).join("/");
  const pathB = b.ancestors().map((n) => n.data.name).join("/");
  return pathA === pathB;
}

function cn(...classes: Array<string | false | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

function trim(s: string, n: number): string {
  if (s.length <= n) return s;
  if (n < 4) return s.slice(0, n);
  return `${s.slice(0, n - 1)}…`;
}

// Keep the unused Entry import anchored — used purely for typing in props.
export type { Entry };
