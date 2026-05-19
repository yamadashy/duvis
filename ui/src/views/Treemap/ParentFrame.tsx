import { categoryVar } from "../../data/categories";
import { humanSize } from "../../data/format";
import type { TreeNode } from "../../data/hierarchy";
import { PARENT_HEADER_MIN_HEIGHT_PX } from "../../data/treemapLayout";
import styles from "./Treemap.module.css";
import { fitParentLabel } from "./label";

/** Height of the parent header strip in pixels. Must match what the
 *  layout pass reserves via paddingTop. */
export const PAD_TOP = 18;

interface FrameProps {
  node: TreeNode;
  radius: number;
  onSelect: () => void;
  onDrillIn: () => void;
  onHoverEnter: (e: { clientX: number; clientY: number }) => void;
  onHoverMove: (e: { clientX: number; clientY: number }) => void;
  onHoverLeave: () => void;
}

export function ParentFrame({
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

  const sizeText = humanSize(node.value ?? 0);
  const { nameDisplay, showSize } = fitParentLabel(node.data.name, sizeText, w, showDot);

  return (
    <g
      transform={`translate(${node.x0},${node.y0})`}
      className={styles.tmParent}
      style={{ cursor: "pointer" }}
      onClick={onSelect}
      onDoubleClick={onDrillIn}
      onMouseEnter={onHoverEnter}
      onMouseMove={onHoverMove}
      onMouseLeave={onHoverLeave}
    >
      <rect className={styles.tmParentRect} width={w} height={h} rx={radius} />
      {hasHeader ? (
        <>
          <rect width={w} height={PAD_TOP} fill="var(--bg-2)" opacity={0.4} rx={radius} />
          {radius > 0 ? (
            <rect y={PAD_TOP / 2} width={w} height={PAD_TOP / 2} fill="var(--bg-2)" opacity={0.4} />
          ) : null}
        </>
      ) : null}
      {showLabel ? (
        <text className={styles.tmParentLabel} x={8} y={12} fill="var(--fg)">
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
