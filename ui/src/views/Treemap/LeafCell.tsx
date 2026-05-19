import { categoryVar, LIGHT_FILL_CATEGORIES } from "../../data/categories";
import { humanSize } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import type { Category } from "../../data/types";
import { cn, trim } from "./label";
import styles from "./Treemap.module.css";

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

const STALE_DAYS = 180;

export function LeafCell(props: LeafProps) {
  const {
    node,
    radius,
    dim,
    isSelected,
    onSelect,
    onDrillIn,
    onHoverEnter,
    onHoverMove,
    onHoverLeave,
  } = props;
  const w = node.x1 - node.x0;
  const h = node.y1 - node.y0;
  // Sub-pixel cells are filtered upstream in layoutTreemap, so no guard here.

  const cat: Category = node.data.category;
  const stale = (node.data.modified_days_ago ?? 0) > STALE_DAYS;
  const lightFill = LIGHT_FILL_CATEGORIES.has(cat);
  const showName = w > 50 && h > 22;
  const showSize = w > 90 && h > 36;
  const showSmallName = !showName && w > 30 && h > 14;

  return (
    <g
      transform={`translate(${node.x0},${node.y0})`}
      className={styles.tmNode}
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
        className={styles.tmNodeRect}
        width={w}
        height={h}
        rx={radius}
        fill={categoryVar(cat)}
        fillOpacity={node.children ? 0.85 : 0.75}
      />
      {stale ? (
        <rect width={w} height={h} rx={radius} fill="url(#stale-pattern)" pointerEvents="none" />
      ) : null}
      {showName ? (
        <>
          <text className={cn(styles.tmLabel, !lightFill && styles.light)} x={6} y={14}>
            <tspan className={styles.tmLabelName}>{trim(node.data.name, Math.floor(w / 6.5))}</tspan>
          </text>
          {showSize ? (
            <text
              className={cn(styles.tmLabel, styles.tmLabelSize, !lightFill && styles.light)}
              x={6}
              y={28}
            >
              {humanSize(node.value ?? 0)}
            </text>
          ) : null}
        </>
      ) : showSmallName ? (
        <text className={cn(styles.tmLabel, !lightFill && styles.light)} x={4} y={11} fontSize={10}>
          {trim(node.data.name, Math.floor(w / 5.5))}
        </text>
      ) : null}
    </g>
  );
}
