import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { categoryMeta, categoryVar } from "../lib/categories";
import { humanSize, pct, relTime } from "../lib/format";
import type { TreeNode } from "../lib/treemap";
import "./Tooltip.css";

interface TooltipProps {
  node: TreeNode | null;
  cursor: { clientX: number; clientY: number } | null;
  total: number;
  /** Path of names from data root down to the current view root. */
  rootPath: readonly string[];
  /** Name of the data root (the directory the user originally scanned). */
  rootName: string;
}

const PAD = 14;

export function Tooltip({ node, cursor, total, rootPath, rootName }: TooltipProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ x: number; y: number }>({ x: -9999, y: -9999 });

  useLayoutEffect(() => {
    if (!node || !cursor || !ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    let x = cursor.clientX + PAD;
    let y = cursor.clientY + PAD;
    if (x + rect.width > window.innerWidth - 8) x = cursor.clientX - rect.width - PAD;
    if (y + rect.height > window.innerHeight - 8) y = cursor.clientY - rect.height - PAD;
    setPos({ x, y });
  }, [cursor, node]);

  // Reset when hidden so we don't flash at the wrong spot on next show.
  useEffect(() => {
    if (!node) setPos({ x: -9999, y: -9999 });
  }, [node]);

  const show = !!node && !!cursor;
  const cat = node?.data.category ?? "other";
  const meta = categoryMeta(cat);
  const days = node?.data.modified_days_ago;
  const hasChildren = !!node?.children && node.children.length > 0;

  // Full path from the originally scanned root all the way down to this node.
  // node.ancestors() only walks up to the current view root, so combine.
  let path = "";
  if (node) {
    const inView = node
      .ancestors()
      .reverse()
      .slice(1)
      .map((a) => a.data.name);
    const segments = [rootName, ...rootPath, ...inView];
    path = segments.join(" / ");
  }

  return (
    <div
      className="tooltip"
      ref={ref}
      role="tooltip"
      data-show={show}
      style={{ left: pos.x, top: pos.y }}
    >
      {node ? (
        <>
          <div className="tt-head">
            <span className="tt-cat-dot" style={{ background: categoryVar(cat) }} />
            <span className="tt-name">{node.data.name}</span>
          </div>
          <div className="tt-path">
            <svg
              className="tt-path-icon"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              aria-hidden="true"
            >
              <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
            </svg>
            <span>{path}</span>
          </div>
          <div className="tt-rows">
            <span className="tt-key">Size</span>
            <span className="tt-val">{humanSize(node.value ?? 0)}</span>
            <span className="tt-key">% of root</span>
            <span className="tt-val">{pct(node.value ?? 0, total)}</span>
            <span className="tt-key">Category</span>
            <span className="tt-val cat-name">{meta.label}</span>
            <span className="tt-key">Modified</span>
            <span className="tt-val">{relTime(days)}</span>
            {hasChildren ? (
              <>
                <span className="tt-key">Items</span>
                <span className="tt-val">{node.descendants().length - 1}</span>
              </>
            ) : null}
          </div>
          <div className="tt-foot">
            <span>{hasChildren ? "double-click to drill in" : "click to inspect"}</span>
          </div>
        </>
      ) : null}
    </div>
  );
}
