import { Fragment, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { categoryMeta, categoryVar } from "../lib/categories";
import { humanSize, pct, relTime } from "../lib/format";
import type { TreeNode } from "../lib/treemap";
import "./DetailPanel.css";

interface DetailPanelProps {
  node: TreeNode;
  total: number;
  /** Path of names from data root to the current view root (rootPath in App state). */
  rootPath: readonly string[];
  rootName: string;
  onDrillIn: (node: TreeNode) => void;
  onSelect: (node: TreeNode) => void;
  onNavigateTo: (path: string[]) => void;
}

export function DetailPanel(props: DetailPanelProps) {
  const { node, total, rootPath, rootName, onDrillIn, onSelect, onNavigateTo } = props;
  const cat = node.data.category;
  const meta = categoryMeta(cat);
  const days = node.data.modified_days_ago;
  const hasChildren = !!node.children && node.children.length > 0;

  const topChildren = node.children
    ? [...node.children].sort((a, b) => (b.value ?? 0) - (a.value ?? 0)).slice(0, 10)
    : [];

  // Full breadcrumb from data root through view root to selected node.
  const inViewSegments = node
    .ancestors()
    .reverse()
    .slice(1)
    .map((a) => a.data.name);
  const fullSegments = [rootName, ...rootPath, ...inViewSegments];

  return (
    <aside className="detail" aria-label="Selection details">
      <div className="detail-head">
        <div className="detail-crumbs" aria-label="Path">
          <svg
            className="detail-crumbs-icon"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            aria-hidden="true"
          >
            <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
          </svg>
          {fullSegments.map((name, i) => {
            const isLast = i === fullSegments.length - 1;
            return (
              <Fragment key={`${i}-${name}`}>
                {i > 0 ? (
                  <span className="detail-crumb-sep" aria-hidden="true">
                    /
                  </span>
                ) : null}
                {isLast ? (
                  <span className="detail-crumb last">{name}</span>
                ) : (
                  <button
                    type="button"
                    className="detail-crumb"
                    onClick={() => onNavigateTo(fullSegments.slice(1, i + 1))}
                  >
                    {name}
                  </button>
                )}
              </Fragment>
            );
          })}
        </div>

        <div className="detail-size tabular">
          {humanSize(node.value ?? 0)}
          <span className="detail-size-pct">{pct(node.value ?? 0, total)} of root</span>
        </div>
        <div className="detail-cat-row">
          <span className="detail-cat-chip">
            <span className="detail-cat-chip-dot" style={{ background: categoryVar(cat) }} />
            {meta.label}
          </span>
        </div>
      </div>

      {hasChildren ? (
        <div className="detail-section">
          <div className="detail-section-title">Top children</div>
          <div className="detail-children">
            {topChildren.map((c, i) => {
              const cat2 = c.data.category;
              const isDir = !!c.children && c.children.length > 0;
              return (
                <button
                  type="button"
                  key={`${i}-${c.data.name}`}
                  className="detail-child"
                  title={isDir ? "Drill into this folder" : "Inspect file"}
                  onClick={() => (isDir ? onDrillIn(c) : onSelect(c))}
                >
                  <span className="detail-child-dot" style={{ background: categoryVar(cat2) }} />
                  <span className="detail-child-name">
                    <FileIcon isDir={isDir} />
                    {c.data.name}
                    {isDir ? "/" : ""}
                  </span>
                  <span className="detail-child-size mono tabular">
                    {humanSize(c.value ?? 0)}
                  </span>
                </button>
              );
            })}
          </div>
        </div>
      ) : null}

      <div className="detail-section">
        <div className="detail-section-title">Metadata</div>
        <div className="detail-meta">
          <span className="detail-meta-key">Type</span>
          <span className="detail-meta-val">{node.children ? "directory" : "file"}</span>
          <span className="detail-meta-key">Modified</span>
          <span className="detail-meta-val">{relTime(days)}</span>
          <span className="detail-meta-key">Items</span>
          <span className="detail-meta-val">
            {node.children ? (node.descendants().length - 1).toLocaleString() : "—"}
          </span>
          <span className="detail-meta-key">Depth from root</span>
          <span className="detail-meta-val">{node.depth}</span>
        </div>
      </div>

      <div className="detail-section">
        <div className="action-row">
          <RevealButton segments={[...rootPath, ...inViewSegments]} />
          <TrashButton />
        </div>
      </div>
    </aside>
  );
}

function RevealButton({ segments }: { segments: readonly string[] }) {
  const [state, setState] = useState<"idle" | "ok" | "error">("idle");

  async function reveal() {
    setState("idle");
    try {
      const res = await fetch("/reveal", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ segments }),
      });
      if (res.ok) {
        setState("ok");
        setTimeout(() => setState("idle"), 1500);
      } else {
        setState("error");
        console.error("reveal failed:", res.status, await res.text());
        setTimeout(() => setState("idle"), 2500);
      }
    } catch (err) {
      setState("error");
      console.error("reveal request failed:", err);
      setTimeout(() => setState("idle"), 2500);
    }
  }

  const label = state === "ok" ? "Opened" : state === "error" ? "Failed" : "Reveal in folder";

  return (
    <button type="button" className="btn" onClick={reveal} title="Open in your file manager">
      <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
        <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
        <path d="M5 6.5l1.5 1.5L9 5.5" />
      </svg>
      {label}
    </button>
  );
}

// Intentionally disabled. duvis never deletes anything — surfacing the
// affordance (instead of omitting it) tells users where the boundary is
// without us answering "why isn't there a delete button" repeatedly.
//
// The tooltip is rendered through a portal at document.body so it can't
// be clipped by the detail panel's overflow, and positioned via fixed
// coords from the button's bounding rect so it lifts above any treemap
// stacking context. (`title` doesn't render reliably on disabled buttons
// in Chrome/Safari, which is why we don't use it.)
function TrashButton() {
  const wrapRef = useRef<HTMLSpanElement>(null);
  const [anchor, setAnchor] = useState<{ cx: number; top: number } | null>(null);

  function show() {
    const el = wrapRef.current;
    if (!el) return;
    const r = el.getBoundingClientRect();
    setAnchor({ cx: r.left + r.width / 2, top: r.top });
  }
  function hide() {
    setAnchor(null);
  }

  return (
    <span
      ref={wrapRef}
      className="hint-wrap"
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
    >
      <button type="button" className="btn" disabled aria-disabled="true">
        <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
          <path d="M2 3.5h8" strokeLinecap="round" />
          <path d="M3 3.5V10a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V3.5" />
          <path d="M5 3.5V2.5a1 1 0 0 1 1-1h0a1 1 0 0 1 1 1v1" />
        </svg>
        Move to trash
      </button>
      {anchor
        ? createPortal(
            <div
              className="hint-tip"
              role="tooltip"
              style={{ left: anchor.cx, top: anchor.top - 8 }}
            >
              <strong>duvis is read-only by design.</strong>
              <br />
              It visualizes disk usage but never deletes anything. To clean up,
              move files to the Trash yourself via Finder, Explorer,{" "}
              <code>rm</code>, or a tool like <code>trash</code> CLI.
            </div>,
            document.body,
          )
        : null}
    </span>
  );
}

function FileIcon({ isDir }: { isDir: boolean }) {
  return isDir ? (
    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M1.5 3.5a1 1 0 0 1 1-1h2l1 1h4a1 1 0 0 1 1 1V9a1 1 0 0 1-1 1h-7a1 1 0 0 1-1-1z" />
    </svg>
  ) : (
    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
      <path d="M3 1.5h4l3 3V10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V2.5a1 1 0 0 1 1-1z" />
    </svg>
  );
}
