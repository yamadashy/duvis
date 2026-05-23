import { useEffect, useRef, useState } from "react";
import { revealInFolder } from "../../api/reveal";
import type { TreeNode } from "../../data/hierarchy";
import styles from "./DetailPanel.module.css";
import { HintWrap } from "./HintWrap";
import { buildEntryPayload, joinPath } from "./helpers";
import { useCopyButton } from "./useCopyButton";

interface ActionRowProps {
  node: TreeNode;
  scanRoot: string;
  segments: readonly string[];
  total: number;
}

/** Bottom-of-panel action row. Two visual tiers:
 *  - Row 1: clipboard ops (cheap, agent-friendly).
 *  - Row 2: filesystem ops (Reveal opens an external app; Trash is
 *    intentionally disabled — duvis is read-only). */
export function ActionRow({ node, scanRoot, segments, total }: ActionRowProps) {
  return (
    <div className={styles.actionRow}>
      <CopyPathButton scanRoot={scanRoot} segments={segments} />
      <CopyJsonButton node={node} scanRoot={scanRoot} segments={segments} total={total} />
      <RevealButton segments={segments} />
      <TrashButton />
    </div>
  );
}

function CopyPathButton({ scanRoot, segments }: { scanRoot: string; segments: readonly string[] }) {
  const { state, run } = useCopyButton();
  const fullPath = joinPath(scanRoot, segments);
  const label = state === "ok" ? "Copied" : state === "error" ? "Failed" : "Copy path";
  return (
    <button type="button" className="btn" onClick={() => run(fullPath)} title={`Copy ${fullPath}`}>
      <svg
        viewBox="0 0 12 12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        aria-hidden="true"
      >
        <rect x="3.5" y="2.5" width="6" height="7.5" rx="1" />
        <path d="M5.5 4.5h2M5.5 6.5h2M5.5 8.5h1.5" strokeLinecap="round" />
      </svg>
      {label}
    </button>
  );
}

function CopyJsonButton({
  node,
  scanRoot,
  segments,
  total,
}: {
  node: TreeNode;
  scanRoot: string;
  segments: readonly string[];
  total: number;
}) {
  const { state, run } = useCopyButton();
  // Mirror buildEntryPayload's own dir test so the tooltip's "fields
  // included" list (which lists `child_count` / `file_count` / `dir_count`
  // only for dirs) matches the actual payload, including for empty dirs.
  const isDir = node.data.is_dir;
  const text = JSON.stringify(buildEntryPayload(node, scanRoot, segments, total), null, 2);
  const label = state === "ok" ? "Copied" : state === "error" ? "Failed" : "Copy JSON";
  return (
    <HintWrap
      tip={
        <>
          <strong>Copies a single-entry JSON record.</strong>
          <br />
          Fields included:
          <ul className="hint-tip-list">
            <li>
              <code>name</code>, <code>absolute_path</code>, <code>relative_path</code>,{" "}
              <code>scan_root</code>
            </li>
            <li>
              <code>size</code>, <code>size_human</code>, <code>pct_of_root</code>
            </li>
            <li>
              <code>category</code>, <code>is_dir</code>, <code>depth</code>
            </li>
            <li>
              <code>modified_days_ago</code>{" "}
              {node.data.modified_days_ago === undefined ? "(N/A)" : null}
            </li>
            {isDir ? (
              <li>
                <code>child_count</code>, <code>file_count</code>, <code>dir_count</code>
              </li>
            ) : null}
          </ul>
          <span className="hint-tip-foot">
            Mirrors the CLI <code>--json</code> per-entry shape. The subtree (<code>children</code>)
            is omitted.
          </span>
        </>
      }
    >
      <button type="button" className="btn" onClick={() => run(text)}>
        <svg
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          aria-hidden="true"
        >
          <path d="M4.5 2.5C3 2.5 2.5 3 2.5 4.5v3C2.5 9 3 9.5 4.5 9.5" />
          <path d="M7.5 2.5C9 2.5 9.5 3 9.5 4.5v3c0 1.5-.5 2-2 2" />
        </svg>
        {label}
      </button>
    </HintWrap>
  );
}

function RevealButton({ segments }: { segments: readonly string[] }) {
  const [state, setState] = useState<"idle" | "ok" | "error">("idle");
  // Hold the pending revert-to-idle timer so we can cancel it both on
  // unmount and on rapid successive clicks. Without this the user could
  // see the label snap back to "idle" mid-flash after a second reveal,
  // and an outstanding timer could try to setState on an unmounted
  // component (React warning) when the panel switches selections.
  const timerRef = useRef<number | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current !== null) {
        window.clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    },
    [],
  );

  async function reveal() {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    setState("idle");
    try {
      await revealInFolder(segments);
      setState("ok");
      timerRef.current = window.setTimeout(() => {
        setState("idle");
        timerRef.current = null;
      }, 1500);
    } catch (err) {
      setState("error");
      console.error("reveal request failed:", err);
      timerRef.current = window.setTimeout(() => {
        setState("idle");
        timerRef.current = null;
      }, 2500);
    }
  }

  const label = state === "ok" ? "Opened" : state === "error" ? "Failed" : "Reveal in folder";

  return (
    <button type="button" className="btn" onClick={reveal} title="Open in your file manager">
      <svg
        viewBox="0 0 12 12"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        aria-hidden="true"
      >
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
function TrashButton() {
  return (
    <HintWrap
      tip={
        <>
          <strong>duvis is read-only by design.</strong>
          <br />
          It visualizes disk usage but never deletes anything. To clean up, move files to the Trash
          yourself via Finder, Explorer, <code>rm</code>, or a tool like <code>trash</code> CLI.
        </>
      }
    >
      <button type="button" className="btn" disabled aria-disabled="true">
        <svg
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          aria-hidden="true"
        >
          <path d="M2 3.5h8" strokeLinecap="round" />
          <path d="M3 3.5V10a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1V3.5" />
          <path d="M5 3.5V2.5a1 1 0 0 1 1-1h0a1 1 0 0 1 1 1v1" />
        </svg>
        Move to trash
      </button>
    </HintWrap>
  );
}
